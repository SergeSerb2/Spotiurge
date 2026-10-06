//! Fork updates are disabled until every artifact identity and installation
//! destination is migrated and verified. This includes startup helper flags.

pub use fastframe_update::{
    CHECK_INTERVAL, DownloadState, Installation, Kind, Prepared, Release, Source, Unsupported,
    Updater,
};
use fastframe_update::{MacConfig, ReqwestTransport, UpdateConfig};

/// Release workflows still carry upstream package identities. Fail closed
/// until every fork artifact and installer destination has been migrated.
pub const ENABLED: bool = false;

pub fn launch() -> fastframe_update::Launch {
    if ENABLED {
        fastframe_update::intercept(&CONFIG)
    } else {
        fastframe_update::Launch {
            arguments: std::env::args_os().collect(),
            receipt: None,
            error: None,
        }
    }
}

pub const CONFIG: UpdateConfig = UpdateConfig {
    macos: MacConfig {
        bundle_ids: &["rocks.spotifast.Spotifast"],
        executable_names: &["Spotifast"],
        legacy_bundle_names: &[],
    },
    // Releases are verified against checksums.txt alone until they are
    // signed. Only a version shipped after the first signed release may
    // carry the key: from then on an unsigned release is refused.
    publisher_key: None,
    ..UpdateConfig::new(
        "SergeSerb2/Spotiurge",
        "Spotifast",
        "spotifast",
        env!("CARGO_PKG_VERSION"),
    )
};

/// Reject updates before creating a transport while the release gate is closed.
pub fn updater(proxy: &crate::settings::ProxyConfig) -> anyhow::Result<Updater> {
    anyhow::ensure!(
        ENABLED,
        "Spotiurge updates are disabled until fork packages are ready."
    );
    let builder = crate::http::blocking_builder(proxy).map_err(anyhow::Error::msg)?;
    Ok(Updater::new(CONFIG, ReqwestTransport::new(builder)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_config_is_valid() {
        CONFIG.validate().unwrap();
        assert_eq!(CONFIG.current_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(CONFIG.repository, "SergeSerb2/Spotiurge");
    }

    #[test]
    fn fork_updates_are_disabled_before_creating_a_transport() {
        assert!(updater(&crate::settings::ProxyConfig::default()).is_err());
    }
}
