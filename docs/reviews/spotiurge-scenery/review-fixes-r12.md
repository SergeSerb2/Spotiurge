---
title: Spotiurge stale-failure backoff and exclusive OAuth bind
description: Keep normal failure backoff after input changes and disable callback port sharing.
nav_exclude: true
---

# PR #2: failure backoff and exclusive callback binding

Baseline: `eade0fd24317bad5ba5e1866c9616cf783c6a517`.

- **Failure backoff.** Every completed active recommendation failure now
  passes through the existing failure handler, even after taste, feedback or
  exploration changes invalidate its generation. Availability and malformed
  responses receive the normal ten-minute initial backoff. Busy and pairing
  retain their short retry and suspension behavior. Successful obsolete picks
  still cannot replace cached content, history or refresh time; duplicate old
  completions cannot affect newer work.
- **Exclusive OAuth listener.** Both the probe and native iOS candidate
  explicitly disable local endpoint reuse. A competing listener must produce
  a retryable bind error before authentication opens. The rationale follows
  [Apple's endpoint-reuse documentation](https://developer.apple.com/documentation/network/nwparameters/allowlocalendpointreuse).
  Listener readiness and canceled/replaced callback guards are retained.

## Verification

Apple Silicon macOS: all CONTRIBUTING checks passed, including both locked
all-target Clippy configurations with warnings denied, default/all-feature
tests (1012/1035 library tests plus other targets), doctests, rustdoc, launchers,
gettext and Jekyll. The three explicitly ignored library tests are unchanged.
The 15 translation catalogs contain source-reference/header changes only;
messages, translations and flags match the baseline.

The active-failure regression covers all five error kinds for both taste and
exploration changes. It checks persisted retry eligibility, blocked manual
refreshes and duplicate completions. The existing input-mutation regression
retains all four mutations and both successful/failed completions, preserving
cached picks, history, refresh time and worker isolation. Its old expectation
that availability errors disappear was replaced by explicit backoff assertions.
No test was removed or skipped.

Host Swift checks bind a real loopback TCP socket with both `SO_REUSEADDR` and
`SO_REUSEPORT`, then verify that the probe listener fails without opening
authentication. Readiness, ordinary occupied-port, canceled-replacement and
disposable Keychain tests still pass. The pre-fix macOS run also refused this
socket, so this is an exclusive-bind acceptance check, not a reproduced macOS
callback-sharing claim. Probe Swift sources typecheck against iPhoneOS; the
existing host audio-graph deprecation remains documented.

The separate native candidate passes 47 Swift core tests and 22 hosted
Simulator tests. Its actual recommendation model receives held 503 and malformed
successful responses after input changes and persists the normal backoff.
Its real reusable-socket test also refuses the competing bind. The result bundle
reports 22 passed, zero failed or skipped. Unsigned no-engine generic iPhoneOS
compilation and engine-enabled Swift source typechecking pass.

The Mac remained muted. The physical phone was not accessed, and no browser
OAuth, live AI/cloud round, cloud deployment, new DMG, audio or TestFlight result
is claimed. The approved scenery appearance is unchanged. Fresh CI and review
must pass before merge.
