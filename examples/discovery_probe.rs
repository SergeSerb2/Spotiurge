//! Diagnostic for the same native credential, merge and HTTP paths as the UI.
//! Pair via environment once; the token is never printed or written to JSON.

use std::path::Path;

use spotifast::discovery::{Document, Replica, Value};

fn save_synced(
    replica: &mut Replica,
    remote: &Document,
    snapshot: &Document,
    path: &Path,
) -> Result<(), String> {
    replica.merge_synced(remote, Some(snapshot))?;
    replica.save(path)
}

fn main() -> Result<(), String> {
    // SAFETY: consume pairing input before the runtime starts any worker.
    unsafe { spotifast::discovery_cloud::capture_bootstrap() };
    tokio::runtime::Runtime::new()
        .map_err(|_| "Cannot start diagnostic runtime.")?
        .block_on(run())
}

async fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let path = std::path::PathBuf::from(args.get(1).ok_or("Supply a private replica JSON path.")?);
    let mut replica = Replica::load(&path)?;
    if let Some(key) = args.get(2).filter(|key| !key.starts_with("--")) {
        replica.edit(
            format!("mix:{key}"),
            if args.iter().any(|a| a == "--delete") {
                None
            } else {
                Some(Value::Mix {
                    title: format!("Cross-device QA mix ({})", &replica.device[..6]),
                    uris: vec![],
                })
            },
        )?;
        replica.save(&path)?;
    }
    if args.iter().any(|a| a == "--offline") {
        println!("local_saved=true device={}", replica.device);
        return Ok(());
    }
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "Cannot build HTTP client.")?;
    // Public, unauthenticated health preflight keeps network failures separate
    // from protected credential failures without printing an auth response.
    let health = client
        .get(format!("{}/health", spotifast::discovery_cloud::endpoint()))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|error| format!("Cloud health request failed: {:?}", error.without_url()))?;
    println!("cloud_health={}", health.status().as_u16());
    let snapshot = replica.document.clone();
    let remote = spotifast::discovery_cloud::sync(
        &client,
        &spotifast::discovery_cloud::endpoint(),
        replica.clone(),
    )
    .await?;
    save_synced(&mut replica, &remote, &snapshot, &path)?;
    println!(
        "{}",
        serde_json::json!({"device":replica.device, "keys":replica.document.records.keys().collect::<Vec<_>>(), "synced":true})
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use spotifast::discovery::{Rating, Record, Stamp};

    #[test]
    fn synced_rating_below_remote_floor_reopens_with_acknowledged_stamp() {
        let mut local = Replica::default();
        let uri = "spotify:track:0123456789ABCDEFGHIJKL";
        let key = format!("feedback:{uri}");
        local
            .edit(
                key.clone(),
                Some(Value::Feedback {
                    uri: uri.into(),
                    title: "Song".into(),
                    artist: "Artist".into(),
                    rating: Rating::Love,
                }),
            )
            .unwrap();
        let snapshot = local.document.clone();
        let mut remote = Document::default();
        remote.records.insert(
            "feedback:retention".into(),
            Record {
                stamp: Stamp {
                    counter: 100,
                    device: "b".repeat(32),
                },
                value: None,
            },
        );
        // Reproduce the HTTP sync path's cloned upload, with a fresh rating
        // re-stamped above another installation's retention floor.
        let mut uploaded = local.clone();
        uploaded.merge_for_sync(&remote).unwrap();
        assert_eq!(uploaded.document.records[&key].stamp.counter, 101);
        let path = Path::new("target").join(format!(
            "discovery-probe-sync-{:032x}.json",
            rand::random::<u128>()
        ));
        save_synced(&mut local, &uploaded.document, &snapshot, &path).unwrap();
        let reopened = Replica::load(&path);
        std::fs::remove_file(&path).unwrap();
        let reopened = reopened.unwrap();
        assert_eq!(reopened.document, uploaded.document);
        assert!(reopened.pending_feedback.is_empty());
        assert_eq!(reopened.document.rating(uri), Some(Rating::Love));
    }
}
