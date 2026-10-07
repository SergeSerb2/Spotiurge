---
title: Spotiurge sign-in listener readiness correction
description: Wait for the loopback callback listener before opening Spotify sign-in.
nav_exclude: true
---

# PR #2: wait for the callback listener

Baseline: `d06c7ed48f020254c2932287db9db083365dced6`. The probe now opens
Spotify's system authentication sheet only after `NWListener` reports ready.
Asynchronous failed or waiting states close the listener and complete sign-in
with an actionable port error. A port already in use therefore cannot leave a
browser sheet waiting for a callback that this attempt cannot receive.

Listener ownership is assigned before startup. Ready is delivered once, and
every queued state or connection callback checks the exact listener identity.
Canceling or replacing an attempt invalidates that identity before canceling
the socket. Browser and request callbacks also check their sign-in state, so
an old callback cannot close or complete a replacement attempt. No Spotify
grant, verifier, callback URL or network error body is logged.

## Verification

Apple Silicon macOS: all CONTRIBUTING checks passed, including both locked
all-target Clippy configurations with warnings denied, default/all-feature
tests (1009/1032 library tests passed plus other targets), doctests, rustdoc,
launchers, gettext and Jekyll. The existing three explicitly ignored library
tests remain documented; credential-storage code did not change in this round.

The host Swift regression exercises the production listener using actual
Network.framework sockets on ephemeral loopback ports. It proves no ready
callback runs synchronously before binding, a ready listener accepts a local
connection, an occupied port fails without opening authentication, and a
canceled listener cannot affect a replacement. These tests never open a
browser or contact Spotify. Existing status, secure-verifier, callback,
audio-graph and disposable native Keychain regressions also pass. All probe
Swift sources typecheck against iPhoneOS. The host audio-graph check retains
the previously documented macOS 27 connect-API deprecation.

The separate native iOS candidate receives the same listener correction and
state guards. Its existing ten-minute sign-in deadline now includes listener
startup. All 19 hosted Simulator tests pass, including actual loopback sockets;
the unsigned no-engine generic iPhoneOS build and engine-enabled Swift source
typecheck pass. Core source is unchanged from its prior 45-test run. These are
compilation and local regression checks, not real Spotify browser sign-in.

The physical phone was not accessed. The approved scenery appearance is
unchanged. No new playback, cloud deployment, DMG, TestFlight build or physical
device result is claimed. Fresh CI and reviews are required before merge.
