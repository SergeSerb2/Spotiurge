//! Diagnostic for the same native credential, merge and HTTP paths as the UI.
//! Pair via environment once; the token is never printed or written to JSON.

use spotifast::discovery::{Replica, Value};

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
    let remote = spotifast::discovery_cloud::sync(
        &client,
        &spotifast::discovery_cloud::endpoint(),
        replica.clone(),
    )
    .await?;
    replica.document.merge(&remote)?;
    replica.save(&path)?;
    println!(
        "{}",
        serde_json::json!({"device":replica.device, "keys":replica.document.records.keys().collect::<Vec<_>>(), "synced":true})
    );
    Ok(())
}
