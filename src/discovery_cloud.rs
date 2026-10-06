//! Bounded, authenticated personal-cloud requests, called only on the backend.

use crate::discovery::{Document, MAX_BYTES, Replica, Suggestion};
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
    let bootstrap = std::env::var("SPOTIURGE_CLOUD_TOKEN").ok();
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
        replica.document.merge(&remote.document)?;
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

pub async fn recommend(
    client: &reqwest::Client,
    endpoint: &str,
    document: &Document,
) -> Result<Vec<Suggestion>, String> {
    document.validate()?;
    let url = validate_endpoint(endpoint)?
        .join("v1/recommendations")
        .map_err(|_| "Invalid cloud URL.")?;
    let token = token(endpoint).await?;
    let response = client
        .post(url)
        .bearer_auth(token)
        .json(&serde_json::json!({"taste": document.taste(), "feedback": document.feedback()}))
        .timeout(std::time::Duration::from_secs(85))
        .send()
        .await
        .map_err(|_| "Recommendations are unavailable. Your previous discoveries are kept.")?;
    status(&response)?;
    #[derive(Deserialize)]
    struct Answer {
        suggestions: Vec<Suggestion>,
    }
    let answer: Answer = response_json(response).await?;
    if !crate::discovery::valid_suggestions(&answer.suggestions) {
        return Err("The AI returned invalid suggestions. Try again.".into());
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
}
