//! Bounded, authenticated personal-cloud requests, called only on the backend.

use crate::discovery::{
    Document, Exploration, MAX_BYTES, RecommendationError, RecommendationErrorKind, Replica,
    Suggestion,
};
use keyring_core::api::CredentialStoreApi;
use serde::Deserialize;

pub const DEFAULT_URL: &str = "https://private-cloud-production.up.railway.app";

pub fn endpoint() -> String {
    std::env::var("SPOTIURGE_CLOUD_URL").unwrap_or_else(|_| DEFAULT_URL.into())
}

fn validate_endpoint(endpoint: &str) -> Result<reqwest::Url, String> {
    let url = reqwest::Url::parse(endpoint)
        .map_err(|_| "Configure the Spotiurge private cloud endpoint.")?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
    {
        return Err(
            "The private cloud must use an HTTPS origin without credentials or a path.".into(),
        );
    }
    Ok(url)
}

// Native calls live on a separate bounded worker: timing out must not leave
// a Tokio blocking task that prevents runtime shutdown. Secrets have no Debug.
type TokenJob = (
    String,
    Option<String>,
    tokio::sync::oneshot::Sender<Result<String, String>>,
);
static TOKEN_JOBS: std::sync::OnceLock<std::sync::mpsc::SyncSender<TokenJob>> =
    std::sync::OnceLock::new();
static BOOTSTRAP_TOKEN: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();

/// Consume pairing input before any helper can inherit it. Native persistence
/// still runs on the credential worker, away from the UI and playback threads.
///
/// # Safety
/// Call at single-threaded process startup, before libraries or workers can
/// access the process environment. Removing environment variables on Unix is
/// unsafe once other threads may be reading them.
pub unsafe fn capture_bootstrap() {
    BOOTSTRAP_TOKEN.get_or_init(|| {
        let token = std::env::var("SPOTIURGE_CLOUD_TOKEN").ok();
        // SAFETY: the caller guarantees single-threaded process startup.
        unsafe { std::env::remove_var("SPOTIURGE_CLOUD_TOKEN") };
        token
    });
}

fn protected_token(endpoint: String, bootstrap: Option<String>) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    let store = apple_native_keyring_store::keychain::Store::new();
    #[cfg(windows)]
    let store = windows_native_keyring_store::Store::new();
    #[cfg(target_os = "linux")]
    let store = zbus_secret_service_keyring_store::Store::new();
    let store = store.map_err(|_| "Unlock your system credential store to pair Spotiurge.")?;
    let entry = store
        .build(
            "com.sergeserbinenko.spotiurge",
            &format!("cloud:{endpoint}"),
            None,
        )
        .map_err(|_| "Cannot open the Spotiurge credential entry.")?;
    if let Some(secret) = bootstrap {
        if secret.len() < 32 || secret.len() > 256 || !secret.is_ascii() {
            return Err("Invalid Spotiurge pairing token.".to_string());
        }
        entry
            .set_secret(secret.as_bytes())
            .map_err(|_| "Cannot protect the Spotiurge pairing token.")?;
        if entry
            .get_secret()
            .map_err(|_| "Cannot verify the protected pairing token.")?
            != secret.as_bytes()
        {
            return Err("The credential store did not retain the pairing token.".into());
        }
    }
    String::from_utf8(
        entry
            .get_secret()
            .map_err(|_| "Pair this device with Spotiurge using SPOTIURGE_CLOUD_TOKEN.")?,
    )
    .map_err(|_| "Invalid protected Spotiurge pairing token.".into())
}

async fn token(endpoint: &str) -> Result<String, String> {
    let endpoint = validate_endpoint(endpoint)?.origin().ascii_serialization();
    let bootstrap = BOOTSTRAP_TOKEN.get().cloned().flatten();
    let jobs = TOKEN_JOBS.get_or_init(|| {
        let (sender, receiver) = std::sync::mpsc::sync_channel::<TokenJob>(4);
        let _ = std::thread::Builder::new()
            .name("spotiurge-cloud-credentials".into())
            .spawn(move || {
                for (endpoint, bootstrap, reply) in receiver {
                    if reply.is_closed() {
                        continue;
                    }
                    let _ = reply.send(protected_token(endpoint, bootstrap));
                }
            });
        sender
    });
    let (reply, pending) = tokio::sync::oneshot::channel();
    jobs.try_send((endpoint, bootstrap, reply))
        .map_err(|_| "The credential store is busy or unavailable.")?;
    tokio::time::timeout(std::time::Duration::from_secs(10), pending)
        .await
        .map_err(|_| "The credential store did not respond. Try again after unlocking it.")?
        .map_err(|_| "The credential store is unavailable.")?
}

async fn response_json<T: serde::de::DeserializeOwned>(
    mut response: reqwest::Response,
) -> Result<T, String> {
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| "Private cloud response interrupted.")?
    {
        if bytes.len() + chunk.len() > MAX_BYTES + 200 {
            return Err("Private cloud response is too large.".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes)
        .map_err(|_| "Invalid private cloud response. Local state is preserved.".into())
}

fn status(response: &reqwest::Response) -> Result<(), String> {
    match response.status().as_u16() {
        200..=299 => Ok(()),
        401 | 403 => Err("Pair this device again with the Spotiurge private cloud.".into()),
        429 => Err("Requests are rate limited. Keep listening and try again later.".into()),
        _ => Err("Private cloud or AI is unavailable. Keep listening and try again later.".into()),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    revision: u64,
    document: Document,
}

pub async fn sync(
    client: &reqwest::Client,
    endpoint: &str,
    mut replica: Replica,
) -> Result<Document, String> {
    let url = validate_endpoint(endpoint)?
        .join("v1/state")
        .map_err(|_| "Invalid cloud URL.")?;
    let token = token(endpoint).await?;
    for _ in 0..3 {
        let response = client
            .get(url.clone())
            .bearer_auth(&token)
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await
            .map_err(|_| "Sync is offline. Your local edits are kept.")?;
        status(&response)?;
        let remote: Snapshot = response_json(response).await?;
        replica.merge_for_sync(&remote.document)?;
        let response = client
            .put(url.clone())
            .bearer_auth(&token)
            .header("If-Match", remote.revision)
            .json(&replica.document)
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await
            .map_err(|_| "Sync was interrupted. Your local edits are kept.")?;
        if response.status() == reqwest::StatusCode::CONFLICT {
            continue;
        }
        status(&response)?;
        return Ok(replica.document);
    }
    Err("Sync met concurrent edits. Retry to merge the latest state.".into())
}

/// The server reports why a request was refused in a bounded `code`; only
/// known codes change the outcome, and no response text reaches logs or UI.
fn refused(status: u16, code: Option<&str>) -> RecommendationError {
    let (kind, message) = match (status, code) {
        (401 | 403, _) => (
            RecommendationErrorKind::Pairing,
            "Pair this device again with the Spotiurge private cloud.",
        ),
        (429, Some("busy")) => (
            RecommendationErrorKind::Busy,
            "Recommendations are already being prepared. Try again in a moment.",
        ),
        (429, _) => (
            RecommendationErrorKind::RateLimited,
            "The AI is rate limited. Keep listening and try again later.",
        ),
        (400, _) => (
            RecommendationErrorKind::Unavailable,
            "Add a taste or rate a few tracks, then try again.",
        ),
        _ => (
            RecommendationErrorKind::Unavailable,
            "Private cloud or AI is unavailable. Keep listening and try again later.",
        ),
    };
    RecommendationError {
        kind,
        message: message.into(),
    }
}

pub async fn recommend(
    client: &reqwest::Client,
    endpoint: &str,
    document: &Document,
    exploration: Exploration,
) -> Result<Vec<Suggestion>, RecommendationError> {
    let pairing = |message: String| RecommendationError {
        kind: RecommendationErrorKind::Pairing,
        message,
    };
    let invalid = |message: String| RecommendationError {
        kind: RecommendationErrorKind::InvalidResponse,
        message,
    };
    document
        .validate()
        .map_err(RecommendationError::unavailable)?;
    let url = validate_endpoint(endpoint)
        .map_err(pairing)?
        .join("v1/recommendations")
        .map_err(|_| pairing("Invalid cloud URL.".into()))?;
    let token = token(endpoint).await.map_err(pairing)?;
    let response = client
        .post(url)
        .bearer_auth(token)
        .json(&serde_json::json!({
            "taste": document.taste(),
            "feedback": document.feedback(),
            "exploration": exploration,
        }))
        .timeout(std::time::Duration::from_secs(85))
        .send()
        .await
        .map_err(|_| {
            RecommendationError::unavailable(
                "Recommendations are unavailable. Your previous discoveries are kept.",
            )
        })?;
    let status = response.status().as_u16();
    if !(200..=299).contains(&status) {
        #[derive(Deserialize)]
        struct Refusal {
            code: Option<String>,
        }
        let code = response_json::<Refusal>(response)
            .await
            .ok()
            .and_then(|refusal| refusal.code);
        return Err(refused(status, code.as_deref()));
    }
    #[derive(Deserialize)]
    struct Answer {
        suggestions: Vec<Suggestion>,
    }
    let answer: Answer = response_json(response).await.map_err(invalid)?;
    if !crate::discovery::valid_suggestions(&answer.suggestions) {
        return Err(invalid(
            "The AI returned invalid suggestions. Try again.".into(),
        ));
    }
    Ok(answer.suggestions)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authenticated_endpoints_reject_plaintext_credentials_paths_and_queries() {
        for bad in [
            "http://example.com",
            "https://user:secret@example.com",
            "https://example.com/api",
            "https://example.com?token=secret",
            "https://example.com/#secret",
        ] {
            assert!(validate_endpoint(bad).is_err());
        }
        assert!(validate_endpoint("https://example.com").is_ok());
    }

    #[test]
    fn server_busy_and_model_rate_limits_are_distinct() {
        assert_eq!(
            refused(429, Some("busy")).kind,
            RecommendationErrorKind::Busy
        );
        for code in [Some("rate_limited"), Some("unknown"), None] {
            assert_eq!(
                refused(429, code).kind,
                RecommendationErrorKind::RateLimited
            );
        }
        assert_eq!(
            refused(401, Some("busy")).kind,
            RecommendationErrorKind::Pairing
        );
        assert_eq!(
            refused(503, Some("busy")).kind,
            RecommendationErrorKind::Unavailable
        );
    }
}
