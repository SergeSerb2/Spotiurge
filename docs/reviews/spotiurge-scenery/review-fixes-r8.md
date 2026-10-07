---
title: Spotiurge review fixes, round eight
nav_exclude: true
---

# PR #2: pairing and playback credential lifecycle

Baseline: `c3b8da6004831c1fbfc1d9deab420d1c863cf8bd`. This follow-up fixes
four review findings and preserves the approved scenery appearance.

- The cloud credential worker moves the bootstrap pairing token out of its
  global slot, writes and reads it back, then drops the bootstrap value. Failed
  writes or verification keep that single input for retry. Later requests read
  protected storage and cannot overwrite a newer pairing with the old input.
- The probe aborts OAuth before opening a listener or browser when secure
  verifier generation fails. No predictable fallback is used.
- Forget cancels pending browser/token work and engine authentication/reconnect,
  stops output, clears Now Playing and deletes the Keychain item. A failed
  deletion is reported while playback remains stopped. Runtime callbacks are
  serialized against cancellation; queued Swift deliveries carry a generation
  ticket, so a new sign-in cannot revive an old credential or playing event.
- [How It Connects](/how-it-connects/) now names the private Railway service,
  documents synchronized records, bearer authentication, Luna-only AI data,
  offline conflict handling, plaintext server access and retention limits.
  It no longer describes the disabled fork updater as an active channel.

## Verification

Apple Silicon macOS: all CONTRIBUTING checks passed, including both locked
all-target Clippy configurations with warnings denied and default/all-feature
tests (1009/1032 library tests passed, with three documented ignored tests per
configuration), doctests, denied-warning Rustdoc, launchers, gettext and Jekyll.
Both explicit native credential-store tests passed with disposable dummy items.
CI now runs the cloud bootstrap round trip on macOS and Windows as well.

New desktop tests inject write failure, read failure, mismatched readback and
invalid input. They verify retry preservation, bootstrap consumption and no
later rewrite of a replaced or unavailable protected token. This verifies
ownership and persistence behavior, not allocator-level secret erasure.

All seven Rust probe tests and denied-warning Clippy passed. The new regression
holds a host callback open while cancellation competes for its generation
lock, then verifies that old delivery is rejected. Host Swift tests passed for
failed RNG, callback revocation across a new connection, failed deletion and a
disposable Keychain deletion that remains absent after stale deliveries. All
probe Swift sources typecheck against iPhoneOS. The host audio-graph test still
reports the macOS 27 deprecation of the existing connect API.

The separate native iOS development candidate received the same callback and
Forget guards, plus cancellable playback sign-in. Its 45 core tests and 17
hosted Simulator model tests passed; the new model test uses injected dummy
secrets and verifies deletion, stale callback rejection, fresh callback
acceptance and failed-deletion behavior. These tests are not browser OAuth or
audible playback evidence.

The physical iPhone was not read, installed, launched or otherwise changed.
No new Windows/Linux runtime, Spotify/AI request, cloud deployment or DMG is
claimed in this review round. Fresh CI and reviews on the pushed commit remain
required before merge; packaging follows the final accepted revision.
