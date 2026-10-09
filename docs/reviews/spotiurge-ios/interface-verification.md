# Native iPhone interface: development verification

Verified October 7, 2026. This is the native SwiftUI development candidate in
`apps/ios`, designed and implemented chiefly by Opus 5.5 with high reasoning.
The desktop interface in `DESIGN.md` is its design reference. It is separate
from the laboratory playback probe and is not ready for TestFlight.

## What changed

The iPhone interface carries the desktop's Inter typography, amber Surge
mark, cover-lit room and neutral selection controls. System Liquid Glass
holds the tab bar, mini-player and controls. Taste and options sheets use an
opaque surface. Home leads with cached discoveries, exploration choices,
automatic recommendations, matching status and listening feedback. Library,
Search, Now Playing and Settings use native navigation and touch targets.
Large text stacks the exploration choices and makes Now Playing scroll.
Reduced Motion disables selection and room transitions.

The Swift core ports the desktop discovery document, validation, merge,
automatic schedule, catalogue matcher and cloud protocol. It adds no package
dependencies. The app links the existing probe's unmodified Rust engine only
for development. It does not fall back to a Spotify remote controller.

Recommendations use the existing private cloud and CLIProxyAPI service,
which accepts only `gpt-6-luna`. Taste, exploration and rated titles/artists
are sent for recommendations; Spotify grants are kept in the phone Keychain.
The [app README](../../../apps/ios/README.md) lists every network destination,
credential boundary, offline behavior and conflict rule.

## Checks actually run

| Check | Result and scope |
|---|---|
| `swift test` in `apps/ios/SpotiurgeCore` | 31 tests passed on macOS arm64, including desktop wire format, per-record merge, bounded history, scheduling, matching, scripted cloud conflicts and Python server validation. |
| Hosted `SpotiurgeTests` on Simulator, engine linked | 13 tests passed on iPhone 18 Pro, iOS 27.0. The tests exercise the actual account, discovery and session models with fake grants and a stub network. |
| Native Keychain round trip | Passed in that hosted suite. Stores, replaces, reads and deletes a disposable UUID dummy item. No real sign-in or pairing item is read. |
| Hosted tests without the engine | The earlier 11-test suite passed. The final 13-test suite was run with the engine linked. |
| Engine-linked Simulator app | Debug arm64 build and hosted tests passed with the stock-pinned Rust static library. Demo audio never starts. |
| Generic device-target build | `apps/ios/build.sh device CODE_SIGNING_ALLOWED=NO` passed after the final fixes. This compiles iphoneos arm64; it does not install, sign or exercise a real iPhone. |
| Project and script syntax | `plutil -lint` on the Xcode project and Info.plist, and `sh -n apps/ios/build.sh` passed. |
| Visual evidence | 26 native Simulator captures: 12 surfaces in light/dark, plus Home and Now Playing at accessibility-large with Reduce Motion. Captures show synthetic demo data or real unsigned/unpaired empty states. |

Logs and Xcode result bundles stay in ignored `.qa/` and
`target/ios-interface/`. The new `iOS core` GitHub workflow runs the package
tests and desktop/server wire checks. Hosted model tests and unsigned app
builds were verified locally, not through that workflow.

The [visual viewer](interface-review.html) shows each surface and theme.
[Capture metadata](interface-captures.json) records source file hashes,
installed Simulator executable hashes and every PNG hash. There is no earlier
Spotiurge iPhone interface with matching content; its Before control is
disabled. The probe's different laboratory screen is not used as a baseline.

## Review and regressions

An independent Opus 5.5 review first found seven issues. The implementation
fixed them, and a fresh second review approved the development source with no
material blocker:

- Offline or failed Spotify refresh keeps the grant; only rejection signs out.
- A single real session survives demo round trips. Demo suspends new cloud,
  AI and Web API requests, cancels active client work and drops late answers.
  A request already received by a server cannot be unsent.
- Demo playback and Settings cannot read or delete the real playback item.
- Sign-out invalidates late refresh and library results. A failed Keychain
  deletion stays visible and retryable.
- An interruption resumes only audio that was playing before it began.
- Cover-light results are discarded when the track changes or the task ends.
- A locked Keychain is distinguished from a missing grant or unpaired phone.

Three low follow-ups were also fixed: successful foreground sync clears the
old locked-Keychain notice, leaving demo retries a refused credential restore,
and an ended playback session gives accurate recovery instructions. The last
13-test run includes the two model regressions and the native store check.
The second review was a source review; it did not inspect the final binary or
run device operations. Simulator captures were refreshed by the parent agent
after those fixes.

## Remaining limits

The earlier probe proved **17 minutes 22 seconds of independent locked
background playback** on Serge's iPhone, with four natural track changes and
automatic session recovery. That result required a local librespot identity
experiment. The [gate record](playback-gate.md) retains its evidence and the
unfinished interruption, route-change and bidirectional Connect criteria.
No more physical-phone operations were performed after Serge asked to stop.

This candidate uses stock `crmne/librespot` at `23fc42c`. Spotify refused its
iPhone identity during the probe, so this interface cannot claim working
independent device playback. No ignored patch or dependency source is shipped.
A production engine needs a maintainer-owned pinned fork, an upstream proposal
and an acceptable distribution path before an architecture commitment.

The development ABI loads one track or context. A mix starts its first track
and discloses that limit; there is no URI-list queue or seek implementation.
Library currently loads the first 50 liked tracks and first playlist page.
There is no offline Spotify audio download.

Not verified in this interface: signed-in real catalogue/library, live Luna
recommendations, real cross-device sync, VoiceOver, frame pacing, energy use,
iPad/landscape or device animations. Static Reduce Motion captures do not prove
animation smoothness. The desktop/cloud live checks are separate evidence.
No signed candidate build, App Store Connect record, distribution archive,
upload or TestFlight build was produced. The
[signing preparation](../../../packaging/ios/README.md) records that path and
its blockers.
