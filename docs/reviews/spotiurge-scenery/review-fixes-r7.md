---
title: Spotiurge review fixes, round seven
nav_exclude: true
---

# PR #2: conflict feedback and service recovery

Baseline: `08b07948079090cd42caf7e5c4bee93097a4f814`. This follow-up addresses
five review findings without changing the approved scenery appearance.

- Unsent love, less and clear edits survive an existing higher-stamped cloud
  rating. Sync imports the clock and restamps any pending record that was
  replaced, not only records removed by retention. Acknowledged edits still
  yield to later remote changes.
- AI-slot contention waits 15 seconds, persists that deadline across restart,
  and leaves the model failure count unchanged. Rate-limit backoff remains
  exponential and requests remain Luna-only through CLIProxyAPI.
- The probe ignores callbacks with a missing, duplicate or different state;
  a Spotify denial with the current state still ends sign-in.
- The probe rebuilds its engine and source node after media-services resets,
  restores its session category, flushes PCM, and replaces the engine observer.
  It waits for a new screen or lock-screen playback request before resuming,
  following [Apple's reset guidance](https://developer.apple.com/documentation/avfaudio/avaudiosession/mediaserviceswereresetnotification).
- The private service closes proxy HTTP error streams, including 429, before
  propagating the result. This round does not change the request model or schema.

## Verification

On macOS, every CONTRIBUTING check passed: launchers, formatting, both Clippy
runs with denied warnings, default and all-feature all-target tests (1008 and
1031 library tests passed, with two documented ignored tests per run), doctests,
denied-warning Rustdoc, translation checks and Jekyll. The all-target suite
includes the disposable native credential-store round trip.

New Rust regressions exercise love, less and clear actions against a higher
remote stamp after serialization, repeated conflict reads, upload acknowledgment
and later remote changes. Busy scheduling tests cover repeated contention,
restart restoration, automatic scheduling and unchanged rate-limit escalation.

All 18 private-cloud tests passed using local disposable SQLite databases and
stub proxy responses. HTTP 429, 401 and 500 streams are closed before the result
is observed. The original 429 fixture now owns a real in-memory response stream,
matching urllib's response lifecycle. No subscription request was made.

`probes/ios-playback/test-signin.sh` passed on this Mac: fresh distinct stopped
AVAudioEngine/source objects with attached nodes, callback fragmentation and
state filtering, interruption intent, failure preservation and a disposable
native Keychain round trip. All probe Swift source type checks against the
iPhoneOS SDK. The host graph check does not simulate an iOS media-server crash
or verify post-reset audible playback. The macOS 27 SDK reports the existing
AVAudioEngine connect API as deprecated; the probe still targets iOS 26.

Your physical iPhone was not read, installed, launched or otherwise changed.
The earlier recorded locked-playback result is unchanged. These are no new
physical interruption, route-change, Connect or media-reset measurements.
No new Windows/Linux runtime evidence was collected; fresh CI on this candidate
is required. The private-cloud patch has not yet been deployed in this round.
