# Whole-app glass preview verification

Recorded October 7, 2026 UTC. This extends the
[first-slice integration record](../spotiurge-discovery/verification.md), rather
than treating those earlier results as tests of this package.

## Native visual evidence

The [comparison](index.html) contains 52 candidate captures from macOS arm64:
light/dark, 1440 x 900 and representative 900 x 760 windows, all at 1x. Home,
onboarding, AI errors, unresolved catalogue suggestions, discovery options,
search, collections, library shelves, queue, settings, Connect, shortcuts,
sign-in and loading are included. Data is synthetic and demo-labelled. No
Spotify or AI requests ran during captures.

Before is commit `176364f229583678b3ab7a19ffe89fb031de1bab`, binary SHA-256
`00ff96bdd1e19ae340cca86816b495a84719cdcefc64dbf7b3bebb6157e8418a`.
The frozen candidate is `0b3e670` plus the source hashes in
[after-captures.json](after-captures.json), binary SHA-256
`fa1b2ab01f6e7ce36b429b17a69af0065dec21c2a6a53135cf1bbaecbdd52480`.
Later history-retention and profile-path fixes changed `app.rs` and
`entrypoint.rs`; every frozen design file remained identical. Captures identify
the actual dirty source closure, rather than claiming they came from a clean
commit or from the release executable. See [the finish verdict](design-review.md).

## Required local checks

On Apple Silicon macOS, after the behavior fixes:

- `cargo fmt --all --check`: passed.
- Both locked all-target Clippy checks, including all features and warnings
  denied: passed.
- `cargo test --locked --all-targets`: passed, 987 library tests, two ignored,
  plus integration/example/binary targets.
- `cargo test --locked --all-targets --all-features`: passed, 1010 library
  tests, two ignored, plus the other targets.
- All-feature doctests and rustdoc with warnings denied: passed.
- Launcher tests, GNU gettext regeneration/check and Jekyll: passed.

The new regressions exercise bounded history across 60 refreshes, near-limit
legacy history, atomic failure, offline convergence without resurrection,
preserved picks after a storage error, fork window paths, paged saved mixes,
choice hit testing, text-field typing/focus, light text contrast, opaque
overlays, narrow chrome, finite transitions and reduced-motion behavior.
There is no lint suppression or removed acceptance test. Test coordinates and
shape selection were updated for the added console margins and glass layers.

## Exact delivered Mac artifact

`Spotiurge-glass-development-universal-20261007-r2.dmg`, 30,092,870 bytes.
SHA-256: `d7851332c1366560a2b85bdcd677142d086f54db6ef0c3615287e84aaf576b49`.
Both release architectures built with the lockfile, without demo or MilkDrop.
The universal executable's SHA-256 is
`e9fa31e237fc42636478ef0085612c0824308dd255159b9d3b710958f9a9b3e5`.

`lipo` confirmed arm64 and x86_64; dynamic dependencies were system libraries.
The bundle uses `com.sergeserbinenko.spotiurge`, its own ICNS and hardened
runtime, signed by an Apple Development identity under team `78A5P57U23`.
Strict/deep signature validation and DMG integrity verification passed. The
read-only mounted contents were the app, Applications shortcut, installation
notes and MIT license, without preferences, grants or pairing credentials.

Gatekeeper assessment **rejected** the unnotarized development app. No
Developer ID private key, notary credentials or stapled ticket was available.
No security setting or quarantine attribute was changed. This is a private
development preview, not a production Mac distribution release.

The exact mounted payload was copied into a fresh QA bundle and launched.
Native interaction verified:

- Spotify session, library, cached discoveries and the saved two-song mix
  restored without another sign-in.
- The native Keychain cloud token restored without an environment bootstrap.
- **Sync my devices** completed with “Synced with your private cloud.”
- **Find new picks** returned twelve fresh suggestions through the deployed
  Luna-only broker. Its shared twenty-second Spotify catalogue deadline left
  all twelve unchecked. A catalogue-only **Check again** retained them and
  timed out again. Earlier library requests reported shared Spotify rate
  limiting. No new playable-catalogue or quality claim follows from this run.
- App volume was zero and the Mac's speaker mute was on. Playback remained
  paused. Local playback and Connect transfer were not repeated in this exact
  package, to avoid disturbing the user's ongoing phone session.

An isolated native demo also exercised navigation from Home to the album
library and album detail, and back through Settings with the app's Reduce
motion control enabled. This establishes control operation, not measured
animation smoothness or the macOS system preference changing live.

The private installer was uploaded without changing its owner-only Drive
permissions, downloaded again through the connector and hashed byte-for-byte
against the local DMG. Its link, checksum, development-signing limitation and
pairing instructions were emailed to Serge. The existing token remains valid;
it is not included in the package or this record.

## Performance and platform limits

Two short packaged-app observations with paused music measured 15.54% of one
CPU core on Home over 20 seconds, and 12.78% on Settings over 10 seconds after
a two-second settle. Resident memory was about 296 MiB; `sample` reported a
331 MiB footprint. This was an authenticated, observed session with Spotify
requests still subject to rate limiting, not a controlled baseline comparison.
These observations do not establish low idle power. Frame pacing, sustained
memory, startup timing and smoothness were not measured. The finite-animation
tests establish that the helpers stop requesting frames after settling.

| Platform | This glass candidate | Earlier evidence and remaining limits |
| --- | --- | --- |
| Apple Silicon Mac, macOS 27.2 | Native captures, controls, full local checks, release launch, auth restore, Luna recommendations, private sync, package checks | Earlier discovery preview proved decoded local playback and playlist Connect both ways; this DMG has no new playback/handoff proof. Shared catalogue rate limits remain. |
| Intel Mac | Release target and universal package validation | No real Intel launch, audio or native UI test. |
| Windows | Branding/resource source updated; existing matrix retained | Earlier physical Windows build and live Mac/Windows sync are documented in the first-slice record. No new Windows UI or audio test; pushed-head CI is separate. |
| Linux | Target-specific paths and reduced-motion fallback retained | No local UI, Secret Service or Flatpak-runtime acceptance run. Pushed-head CI is separate. |
| Real iPhone | Independent probe's locked playback passed for 17 min 22 s with four natural transitions and reconnect | [Full gate remains partial](../spotiurge-ios/playback-gate.md). Device checks stopped at Serge's request. The identity experiment is not a production dependency. |
| iOS Simulator / TestFlight | Separate native interface development is in progress | Not part of desktop capture or package acceptance. No TestFlight upload or three-platform application sync claim. |

Recommendation parity still needs real listening feedback. The spoken DJ,
export/deletion controls, backup scheduling, a distributable iOS dependency,
production signing and full three-platform application acceptance remain
separate milestones.
