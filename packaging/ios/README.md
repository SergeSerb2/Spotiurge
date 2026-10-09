# iPhone playback gate and TestFlight preparation

Serge resumed iOS on October 6, 2026 after connecting his real iPhone; desktop
delivery continues in parallel. There is no production iOS target yet. Choosing
one before real independent background playback is proven would violate the
product brief. The official
Spotify iOS SDK controls the Spotify app and cannot satisfy that requirement.
See the [probe acceptance plan](../../docs/_reference/spotiurge-architecture.md#ios-architecture-gate-and-signing-preparation).

Xcode 27.0 and an Apple Development identity are available for team `78A5P57U23`.
The recorded device runs used an iPhone 17 Pro Max on iOS 27.2, connected by
cable with Developer Mode enabled. The signed playback probe used its own
bundle ID and a team-managed development profile. Build 8 proved 17 minutes
22 seconds of independent locked playback with four natural track changes
and automatic recovery from a session disconnect. The complete gate remains
open for interruption, route changes and Connect handoff in both directions.
Phone checks stopped at Serge's request after build 9 was installed. See the
[redacted gate record](../../docs/reviews/spotiurge-ios/playback-gate.md).
Distribution signing, a Spotiurge App Store Connect record and upload access
are unverified.
A simulator does not satisfy the gate.

The isolated playback probe informs the production architecture decision.
The developmental interface now follows the desktop's forest/mist palette,
mountain scenery, flat ridge mark and finite selection motion. Opus 5.5's
current role is design review only. A controller or synthetic tone can diagnose
a component, but cannot pass independent Spotify background playback. The
probe must not replace unrelated installed apps or reuse their provisioning
identities.

`ExportOptions.plist.example` prepares automatic App Store Connect upload for
that team. Confirm team membership and reserve the fork-owned bundle ID before
using it. After the playback gate passes and the chosen iOS target exists:

1. Register `com.sergeserbinenko.spotiurge` and create its App Store Connect app.
2. Enable only required capabilities, including justified background audio and
   protected credential storage. Complete privacy and export-compliance entries.
3. Create a distribution identity/profile through Xcode automatic signing or
   configure the existing protected signing infrastructure. Upload credentials
   belong in protected CI secrets; never in this repository.
4. Archive the release scheme for `generic/platform=iOS` into a checkout-local
   QA directory. Validate the archive, inspect bundle ID/team/background modes
   and increment the build number.
5. Copy the export-options example to the QA directory and export/upload the
   archive with `xcodebuild -exportArchive`. An accepted upload is not a pass.
6. Wait for Apple processing, complete compliance, assign the internal testing
   group, install that exact build via TestFlight and repeat the real-phone gate.

No app record, distribution profile, archive or TestFlight build is claimed by
this preparation. Do not upload a remote-controller substitute as an independent
player. No signing secrets or provisioning files are committed.

A developmental SwiftUI interface candidate lives in [`apps/ios`](../../apps/ios/README.md).
It uses its own development bundle ID, `com.sergeserbinenko.spotiurge.dev`,
and keeps `com.sergeserbinenko.spotiurge` reserved for the chosen production
target. It has been built and captured only on the iOS Simulator; no signed
device build, archive or upload has been made from it. Its playback engine is
the probe's stock-pinned librespot, which Spotify refuses on iOS until the
identity fix is in a pinned fork, so it is not a TestFlight candidate.
