# Sync diagnostic and sign-in identity fixes

Recorded October 7, 2026, after review of `e547786`.

The discovery diagnostic now reconciles an acknowledged upload through
`Replica::merge_synced`, using its dispatched snapshot before saving. This
clears acknowledged pending ratings whose stamps were raised above a remote
retention floor. The regression reproduces that cloned upload and reopens the
actual saved file, preserving the rating without stale pending stamps.

Spotify's browser completion pages now say Spotiurge on successful and failed
sign-in. This changes visible identity only; grants, callbacks, scopes and
credential storage are unchanged. A regression checks both generated pages.

The full CONTRIBUTING checks pass on Apple Silicon macOS: launchers, format,
both locked all-target Clippy configurations, default/all-feature tests
(1005/1028 library tests plus other targets, including the new diagnostic
regression), doctests, rustdoc, gettext and Jekyll. The two focused regressions
also pass. No dependency or Nix vendor hash changes are needed in this round.
Exact-head Windows/Linux/Nix results remain separate CI evidence.

The private r7 universal development DMG was built from `e547786`, before these
two corrections. Its Apple Silicon native launch restored the Spotify session,
library, saved discovery mix and cached picks. Retrying catalogue matching
without another AI call ended with "Spotify took too long to answer", leaving
0 playable and 12 unchecked suggestions. Playback stayed paused at zero app
volume with the Mac muted. This does not prove new-package audio or recommendation
quality. Intel payload validation is not an Intel runtime test. The physical
iPhone remained untouched, and the r7 package remains unnotarized.
