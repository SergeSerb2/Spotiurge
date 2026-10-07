# Final review fixes

Manual AI requests now observe the automatic scheduler's retry deadline. Home
disables refresh during that deadline and schedules one redraw for its expiry.
Only pairing failures can be explicitly rearmed. Playback and cached picks remain
available during the cooldown.

Saved mixes have a remove control that writes an immediate durable tombstone.
A stale sync cannot restore the removed mix. New saves reuse removed keys first,
then the oldest key once 100 mix slots exist, without creating another key on
every save. Existing imported slots remain readable and removable.

The playback probe accumulates a complete bounded callback request line before
parsing it. It updates reusable credentials in place, checks the result, and
records storage success only after Security.framework returns success.

## Actually checked

- Every CONTRIBUTING check passed: launchers, formatting, both locked all-target
  Clippy configurations with warnings denied, default/all-feature tests
  (1003/1026 library tests passed, two ignored in each), doctests, rustdoc,
  gettext and Jekyll. Other binary/integration/example targets also passed.
- Regressions cover repeated manual refresh after a 429, the retry deadline,
  2500 mix saves, slot reuse, removal and stale sync, plus accessible remove
  controls in the paginated list. All 14 locales include the new label.
- `probes/ios-playback/test-signin.sh` passed on macOS: every byte split, bounds,
  exact callback/state validation, failed Keychain replacement and a disposable
  native store round trip. It never reads the real playback credential.
- All probe Swift sources typechecked against the iPhoneOS SDK with the engine
  header. No probe was installed, launched or read on the physical phone.
- [Native comparison](index.html): four matched before/after pairs, dark/light
  at 1440x900 and 900x760 logical sizes. Both binaries use the same labelled
  `discovery-remote,discovery-empty` demo state. The new remove control is
  disabled in demo; the functional regressions above test removal.
- Eight native-resolution JPEG derivatives (quality 92) retain exact original
  PNG hashes, dimensions, flags and build inputs in [captures.json](captures.json).
  No artistic editing was done. Root inspected the native captures.

The full scenery design's earlier captures remain unchanged. Its before binary
is the frozen r4 candidate already identified in the parent manifest; the after
binary contains these final uncommitted fixes with exact input hashes recorded.
No fresh Windows/Linux runtime, animation measurement or physical iPhone
coverage is claimed. The production iOS playback gate stays partial.
