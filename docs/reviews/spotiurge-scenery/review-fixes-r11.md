---
title: Spotiurge pending intent and service failure corrections
description: Preserve offline records, honor superseded request throttles and surface credential rejection.
nav_exclude: true
---

# PR #2: preserve intent and report service failures

Baseline: `366793a8b049f5f554513a19244607b803b6b57f`.

- **Offline records.** The local `pending_edits` journal now records every
  edited record, including taste, mix/history changes and tombstones. It is
  bounded by the retained record count. A cloud clock import captures that
  intent before merging, then re-expresses overwritten pending values above
  the imported clock. Acknowledgments clear only dispatched stamps; newer
  local edits remain pending. A cloud import without a dispatched snapshot
  acknowledges nothing. Earlier `pending_feedback` entries migrate on read.
  The cloud document, retention limits and AI payload stay unchanged; the
  journal is persisted locally and is never sent to either service.
- **Superseded AI requests.** Obsolete picks and input-specific errors still
  cannot update cached picks, history or refresh time. The active request's
  Busy, RateLimited and Pairing failures install and persist their service
  retry deadline or automatic suspension even when inputs changed. Duplicate
  old completions cannot throttle or release newer work. Explicit manual
  recovery from a pairing error retains its existing behavior.
- **Synchronous connection refusal.** A nonzero `probe_connect` result closes
  the callback gate, clears live playback status and displays how to sign in
  again, Forget or restart. The saved credential is preserved for explicit
  recovery. Foreground restoration cannot retry a revoked connection, and
  queued callbacks cannot store its rejected credential.

## Verification

Apple Silicon macOS: all CONTRIBUTING checks passed, including default and
all-feature locked Clippy with warnings denied, all-target tests (1012/1035
library tests passed plus other targets), doctests, rustdoc, launchers, gettext
and Jekyll. The existing three explicitly ignored library tests are unchanged.
The live-sync example was updated for the renamed journal field. Fifteen
translation catalogs have source-reference/header updates only; their messages,
translations and flags were compared with the baseline and are unchanged.

The new Rust sync regression writes and reloads real local replica files,
imports higher cloud clocks, simulates a CAS conflict, verifies acknowledgment
of pre-dispatch intent and then accepts a subsequent cloud edit. It covers
taste replacement/clearing, mix replacement/removal and history removal.
Additional checks cover legacy field migration and imports without a dispatched
snapshot. Existing in-flight edit, retention and clock-collision tests pass.

The application regression changes taste or exploration during an active AI
request, receives each of the three service throttles, checks its persisted
checkpoint and blocked retries, and proves an old duplicate cannot affect a
new request. No AI request is sent by these scripted backend tests.

Host Swift checks pass for synchronous rejection statuses, callback revocation
across a fresh connection and the existing disposable native Keychain round
trip. Probe Swift sources typecheck against iPhoneOS. The host audio-graph
check retains the previously documented macOS 27 connect-API deprecation.

The separate native iOS candidate receives all three fixes. Its 47 Swift core
tests and 21 hosted Simulator tests pass. The hosted failure test drives its
actual result handler without audio, preserves the dummy saved credential and
rejects stale writes. Held local URLProtocol requests prove stale service
refusals block automatic/manual retries. Unsigned no-engine generic iPhoneOS
compilation and engine-enabled Swift source typechecking pass.

The physical phone was not accessed, and the Mac remained muted. No new live
Spotify, AI, cloud sync/deployment, DMG or TestFlight result is claimed. The
approved scenery appearance is unchanged. Fresh CI and review must pass before
merge.
