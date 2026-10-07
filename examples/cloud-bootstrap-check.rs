//! Regression probe: pairing input must not be inherited by a spawned helper.
//! Run with a dummy SPOTIURGE_CLOUD_TOKEN; never touches the native credential store.
use std::{error::Error, process::Command};

fn main() -> Result<(), Box<dyn Error>> {
    if std::env::args().nth(1).as_deref() == Some("--child") {
        if std::env::var_os("SPOTIURGE_CLOUD_TOKEN").is_some() {
            return Err("A helper inherited pairing input".into());
        }
        return Ok(());
    }
    if std::env::var_os("SPOTIURGE_CLOUD_TOKEN").is_none() {
        return Err("Run the probe with dummy pairing input".into());
    }
    // SAFETY: no workers, runtime, or platform initialization has started.
    unsafe { spotifast::discovery_cloud::capture_bootstrap() };
    if std::env::var_os("SPOTIURGE_CLOUD_TOKEN").is_some() {
        return Err("Pairing input remained in the process environment".into());
    }
    if !Command::new(std::env::current_exe()?)
        .arg("--child")
        .status()?
        .success()
    {
        return Err("Pairing input isolation failed".into());
    }
    println!("Pairing input consumed; spawned helper inherited no token.");
    Ok(())
}
