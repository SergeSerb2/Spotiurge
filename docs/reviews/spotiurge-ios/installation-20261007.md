# Private native iPhone installation

On October 7, Serge explicitly requested installation of the native iOS app.
This authorized installation and an attempted launch; the earlier instruction
to stop phone playback testing was preserved.

- Native app: `Spotiurge`, distinct from `Spotiurge Probe`.
- Development bundle: `com.sergeserbinenko.spotiurge.dev`, build `20261008`.
- Apple signing team: `78A5P57U23`.
- Automatic profile: `iOS Team Provisioning Profile: *`, including the connected
  iPhone. The production bundle ID remains reserved for later distribution.
- Signed engine-enabled iPhoneOS build and strict code-signature verification:
  passed. The Info.plist retains background audio.
- `devicectl` installation on the physical iPhone: **success**, verified in its
  structured installation receipt. An initial remote launch was denied because
  the phone was locked. No unlock, sign-in or playback check was requested.

## Engine provenance

This private build explicitly imported a copy of the successful playback probe's
existing experimental static library, SHA-256
`5dd15e62fd90c6584c8f42593b899c76d71594280e8090ef017028e7f9db05a2`
(84,022,480 bytes), into this worktree's owned build directory. It has the same
probe ABI and the documented iOS `config::OS = "macos"` dependency experiment.
No dependency source or ignored-path reference was added to the repository.
The normal build script still uses the stock pinned dependency.

The install enabled `SPOTIURGE_ENGINE` and
`SPOTIURGE_EXPERIMENTAL_IOS_IDENTITY`. Settings explicitly labels the experiment
and the remaining maintained-dependency/distribution work. The app uses its own
audio session; no remote-controller fallback was installed.

The source was the reviewed native candidate at `a400557`, plus the Settings
description flag. Earlier checks remain 47 Swift core tests, 22 hosted Simulator
tests, an unsigned device compilation and engine-enabled source typechecking.
This installation adds no evidence for audible native-app playback, catalogue
matching, real sync, recommendations or performance. The probe's earlier locked
playback record remains separate. TestFlight upload is still outstanding.

The owner received sign-in and private-cloud pairing instructions by private
email. No credentials or provisioning file are committed here. Raw build,
signature and installation receipts stay in `target/ios-installed-preview/`.
