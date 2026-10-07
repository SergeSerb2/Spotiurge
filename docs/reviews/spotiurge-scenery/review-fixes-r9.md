---
title: Spotiurge reconnect status correction
description: Reconnect status regression checks and limits of prior device evidence.
nav_exclude: true
---

# PR #2: clear stale playback status during reconnect

Baseline: `80406b68bb604a6fa432ca0e152e5bba75aeaae5`. The probe now clears
its live connection and playback flags on `reconnecting`, `reconnect_failed`,
`session_ended` and terminal `connect` errors. Explicit replacement connects
also clear the prior live state. Queue metadata and position remain available
for recovery. A new connected event restores connection status; playback status
returns only after a new playing event.

This corrects the existing screen, lock-screen rate and heartbeat fields. The
audio counter is labelled rendered audio rather than claiming it was heard.
The first idle timestamp survives repeated failures, so the existing five-minute
idle-output stop can still run. Connection loss also discards interruption
resume intent. Command errors leave an otherwise live session intact.

## Verification

Apple Silicon macOS: all CONTRIBUTING checks passed, including both locked
all-target Clippy configurations with warnings denied, default/all-feature
tests (1009/1032 library tests passed plus other targets), doctests, rustdoc,
launchers, gettext and Jekyll. The Rust engine and cloud behavior are unchanged.

The host Swift regression drives the production status reducer through active
playback, connection loss, six failed retries, terminal failure and recovery.
It checks both flags and the retained idle deadline, proves that a connected
event alone does not claim resumed playback, and also checks direct failure,
paused-session and command-error paths. The existing disposable native Keychain
regressions pass. All probe Swift sources typecheck against iPhoneOS. The host
audio-graph check reports the existing macOS 27 connect-API deprecation.

The separate native iOS candidate receives the same correction through its
actual event handler, which is exercised by a hosted Simulator model test.
All 18 hosted tests pass, with audio omitted. The unsigned no-engine generic
iPhoneOS build and engine-enabled Swift source typecheck pass. These are
compilation and scripted-state checks, not live Spotify or audible recovery.

The physical phone remained untouched. Older probe `connected`/`playing`
booleans could stay true after a lost session and are insufficient playback
evidence by themselves. The recorded locked-run result rests on timestamped
decoded/rendered frame progress, track changes, route/lock state and Spotify's
absence; its 1.36-second recovery silence remains explicitly recorded. No
additional device run, route/interruption measurement, cloud deployment or
new DMG is claimed. Fresh CI and reviews are required before merge.
