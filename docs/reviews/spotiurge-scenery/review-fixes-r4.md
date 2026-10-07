# Restart, catalogue and probe review fixes

PR #2's follow-up review of `abd7e1c` identified five behavioral or fork-routing
issues. This round preserves the approved scenery interface and existing visual
captures; it changes no layout, palette, animation or logo.

- Recommendation attempts, pending changes, failure counts, retry deadlines and
  pairing suspension persist locally outside the synchronized document. The
  runtime restores bounded monotonic deadlines from wall-clock checkpoints.
  Legacy successful refresh times provide the missing attempt checkpoint.
  The serialized discovery writer completes the attempt save before AI access.
- Catalogue matching uses ICU full case folding before NFC. Regression cases
  cover sharp S, final sigma, ligatures, dotted I and decomposed accents; accents
  and explicit live/remix versions still distinguish tracks. `icu_casemap` adds
  its compiled mapping data and shares the existing ICU provider. The lockfile
  and Nix vendor hash were refreshed together.
- A probe PlayerEvent pause clears queued PCM before notifying Swift. The host
  regression fills the complete half-second queue, forwards a real pause event,
  verifies silence, then verifies fresh samples render in order.
- Probe interruption recovery requires playback to have been active at the
  beginning and the system's resume option at the end. Host tests cover idle,
  deliberately paused, non-resuming and duplicate-ended notifications.
- The gettext wrapper and all 15 catalog headers direct translation reports to
  SergeSerb2/Spotiurge. Existing translations and the inherited catalog domain
  remain intact.

Verification on macOS arm64: all CONTRIBUTING checks passed, including both
Clippy configurations, 1004 default-feature and 1027 all-feature library tests
(two explicitly ignored in each), binary/integration/example targets, doctests,
Rustdoc with denied warnings, gettext freshness, launchers and Jekyll. The probe
passed three Rust host tests, Clippy, formatting, Swift host regressions with a
disposable native Keychain round trip, and iPhoneOS SDK typechecking.

Nix is unavailable locally. The vendor NAR calculation reproduced the previous
hash before calculating the new hash from verified cached crate tarballs and
exact pinned Git trees; the actual package build remains a required CI check.
No new Windows/Linux application runtime, recommendation listening-quality
result, cloud synchronization round or physical iPhone test is claimed. The
production iOS gate remains partial and the phone was not accessed.
