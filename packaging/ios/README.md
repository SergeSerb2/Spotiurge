# iPhone playback gate and TestFlight preparation

Serge deferred iOS while remote on October 6, 2026; macOS is the current priority.
There is no production iOS target yet. Choosing one before real independent
background playback is proven would violate the product brief. The official
Spotify iOS SDK controls the Spotify app and cannot satisfy that requirement.
See the [probe acceptance plan](../../docs/_reference/spotiurge-architecture.md#ios-architecture-gate-and-signing-preparation).

Local Xcode and an Apple Development identity are available. Its certificate
OU is team `78A5P57U23`. Only a development identity was found; distribution
signing, a Spotiurge App Store Connect record and upload access are unverified.
The real iPhone is currently unavailable to `devicectl`. A simulator does not
satisfy the gate.

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
