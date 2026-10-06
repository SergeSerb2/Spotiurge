//! The fork preview must reject updater flags before any legacy package
//! discovery, install or profile mutation can run.

use std::process::Command;

/// There is no fork update contract until package identities are migrated.
#[test]
fn update_relaunch_flags_are_rejected_while_fork_updates_are_disabled() {
    const { assert!(!spotifast::updates::ENABLED) };
    for flags in [
        [
            "--update-receipt",
            "/missing/.spotifast-update-0000000000000000/handoff.json",
        ],
        [
            "--update-error",
            "The update could not start. The previous version has been restored.",
        ],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_spotifast"))
            .args(flags)
            .arg("--version")
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2), "{flags:?}: {output:?}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("unexpected argument"));
        assert!(output.stdout.is_empty());
    }
}

/// A helper request must fail at the parser, without touching a profile or job.
#[test]
fn apply_update_is_rejected_before_any_helper_or_app_work() {
    const { assert!(!spotifast::updates::ENABLED) };
    let scratch = std::env::temp_dir().join(format!(
        "spotifast-apply-update-{:016x}",
        rand::random::<u64>()
    ));
    std::fs::create_dir(&scratch).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_spotifast"))
        .arg("--apply-update")
        .arg(scratch.join("missing-job.json"))
        .env("HOME", &scratch)
        .env("XDG_CONFIG_HOME", scratch.join("config"))
        .env("XDG_STATE_HOME", scratch.join("state"))
        .env("XDG_CACHE_HOME", scratch.join("cache"))
        .env("XDG_DATA_HOME", scratch.join("data"))
        .env("APPDATA", scratch.join("appdata"))
        .env("LOCALAPPDATA", scratch.join("localappdata"))
        .output()
        .unwrap();
    let left = std::fs::read_dir(&scratch).unwrap().count();
    std::fs::remove_dir_all(&scratch).unwrap();
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("unexpected argument"));
    assert_eq!(left, 0, "disabled updates must not touch the profile");
}
