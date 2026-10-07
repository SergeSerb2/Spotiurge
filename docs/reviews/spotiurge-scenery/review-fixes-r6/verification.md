# Startup and connection review fixes

Recorded October 7, 2026, after review of `7aaed52` on PR #2.

Home disables exploration until local discovery has loaded successfully. The
action handler also rejects edits before that point. The regression checks
opening, corrupt, oversized and unreadable files, unchanged settings and replica,
and enabled controls after a successful load. Original failed files remain intact.
Local discovery loading now bypasses proxy credential restoration; an actual
worker regression receives its loaded state while restoration is still pending.

The Rust iOS probe tracks and cancels its whole connection attempt, including
authentication and reconnect backoff. Replacements wait for old session cleanup.
Generation checks reject obsolete publication and player events. Cancellation
also releases a decoder waiting for PCM queue space before its player drops.
All six probe Rust host tests and Clippy pass. This is host lifecycle evidence,
not a new Spotify session, audio run or physical-device test.

All CONTRIBUTING checks pass on Apple Silicon macOS: launchers, format, both
locked all-target Clippy configurations, default/all-feature all-target tests
(1006/1029 library tests plus other targets), doctests, rustdoc, gettext and
Jekyll. Translation updates only refresh source locations and the POT date.
Dependencies and the Nix vendor hash are unchanged. Exact-head Windows/Linux/Nix
checks remain CI evidence.

[Comparison](index.html) contains four matched native before/after pairs: light
and dark themes at 1440x900 and 900x760 logical sizes. Data and failed-load state
are synthetic and isolated from real grants, files, playback and network access.
The baseline is the recorded r3 executable built from `ce7a42e` with its recorded
working changes; this is not presented as a fresh `7aaed52` baseline build.
The candidate includes the current startup fixes. Exact source inputs, binary
and native/derivative image hashes are in [captures.json](captures.json).
Only exploration's disabled state changes; the scenery design remains the same.

The previously emailed r7 DMG predates these fixes. No new package or TestFlight
distribution is claimed by these checks. The Mac stayed muted and physical-phone
operations remain stopped at Serge's request.
