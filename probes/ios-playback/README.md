# iPhone independent-playback probe

An isolated feasibility probe for the iPhone playback gate in
[the architecture reference](../../docs/_reference/spotiurge-architecture.md#ios-architecture-gate-and-signing-preparation).
It answers one question: can Spotiurge's own pinned librespot engine stream
and decode Spotify Premium music on a real iPhone, in the background, with
the probe owning its audio session and the Spotify app not running?

The probe does not choose the production iOS architecture or interface. Its
UIKit screen is a deliberately plain laboratory. It is not a Spotify App
Remote/SDK controller and must never be replaced by one.

## Layout

- `engine/`: Rust staticlib. The desktop's pinned `crmne/librespot` revision
  (`23fc42c`), the same hyper-proxy2 patch, and a lockfile seeded from the root
  `Cargo.lock`. Every crate matches the root lock except `webpki-roots 0.26.11`,
  the compatibility re-export of the root's `webpki-roots 1.0.9`. The engine
  exposes a C ABI (`engine/include/probe_engine.h`). Its librespot `Sink` hands
  44.1 kHz stereo f32 PCM to a bounded half-second queue. The queue blocks the
  decoder thread and never blocks the render thread.
- `app/`: a hand-written Xcode project (`PlaybackProbe.xcodeproj`) with three
  Swift files:
  - `Probe.swift`: owns the `AVAudioSession` (`.playback`, `.longFormAudio`)
    and an `AVAudioEngine` with an `AVAudioSourceNode` that drains the engine's
    queue. It handles interruptions, route changes, engine configuration
    changes, lock-screen Now Playing and remote commands. It also writes the
    evidence log.
  - `SignIn.swift`: the desktop's streaming-scope PKCE sign-in (`src/auth.rs`):
    Spotify's own client ID with its registered loopback redirect
    `http://127.0.0.1:8898/login`. The probe presents the page with
    `ASWebAuthenticationSession`, which is the system sign-in sheet and not an
    embedded browser. A `Network.framework` listener on 127.0.0.1 answers the
    redirect. Only the reusable librespot credential is kept, in the Keychain
    (`AfterFirstUnlockThisDeviceOnly`).
  - `App.swift`: the scene-based laboratory screen with 44-point controls.
- `Info.plist`: `UIBackgroundModes = audio` and the scene manifest. Background
  audio needs no separate entitlement.
- `build.sh`: builds the engine, then the signed app. All output goes to
  `target/ios-probe/` in the checkout.

Current source opens the system sign-in sheet only after the loopback listener
reports ready. It binds exclusively, with local endpoint reuse disabled, so a
competing bind fails before authentication opens, including when the other
socket opts into address/port reuse. Asynchronous listener failures or waiting
states close that attempt with a retryable sign-in-port error. Canceled or replaced listeners
cannot open a sheet or complete a later attempt; browser and request callbacks
also belong to their exact sign-in state. Host regressions use ephemeral local
ports, including occupied and reusable-socket tests, without opening Spotify.
These source fixes have not been installed or tested on the physical phone.

A synchronous connection rejection also revokes its callback gate, clears live
playback status and displays a credential recovery error. No asynchronous
engine event is expected when the C ABI refuses an empty or malformed blob.
The saved credential stays intact for explicit replacement or Forget; foreground
restoration cannot keep retrying a revoked connection.

## Why webpki roots instead of the desktop's native roots

`rustls-native-certs` 0.8 only reads the system trust store on macOS and
Windows (`#[cfg(target_os = "macos")]`). On iOS it falls back to the Unix
certificate-file lookup, finds no files, and every TLS connection to Spotify
would fail. The probe therefore enables librespot's `rustls-tls-webpki-roots`
feature, which bundles Mozilla's roots. A production iOS target needs the same
decision, or a platform verifier.

## Required librespot change (not yet in a pinned fork)

Unpatched librespot identifies itself to Spotify as an iPhone client, and
Spotify refuses that login. The failures are recorded in
[the gate record](../../docs/reviews/spotiurge-ios/playback-gate.md). The probe
only plays with
[`librespot-ios-os.diff`](../../docs/reviews/spotiurge-ios/librespot-ios-os.diff)
applied: `config::OS` reports `"macos"` on iOS, which is the desktop identity.
That patch belongs in a maintainer-owned librespot fork pinned to a commit.
Until then, device builds use a local, ignored experiment script that applies
the patch to a copy under `target/ios-probe/` through `cargo --config`. The
manifest and lockfile here keep the stock pin, so `build.sh` produces an app
that cannot log in.

To reproduce the device builds (builds 5 to 9) until the fork exists:

1. Copy the cargo checkout of `crmne/librespot@23fc42c` to
   `target/ios-probe/librespot-ios-platform`, and apply the diff there.
2. Write a cargo config file that patches
   `'https://github.com/crmne/librespot'` for all seven `librespot-*` crates
   to `path` entries in that copy.
3. Save a copy of `engine/Cargo.lock`. Run
   `cargo build --release --target aarch64-apple-ios --config <file>` in
   `engine/`, then put the saved lockfile back.
4. Run the `xcodebuild` line from `build.sh`.

When the fork is pinned, change the five `rev` lines here and drop the
patch.

## Build

```sh
rustup target add aarch64-apple-ios --toolchain 1.98.0   # once
probes/ios-playback/build.sh CODE_SIGNING_ALLOWED=NO     # compile check
probes/ios-playback/build.sh                             # signed, team 78A5P57U23
```

Signing uses Xcode automatic signing with `-allowProvisioningUpdates` and the
probe's own bundle ID, `com.sergeserbinenko.spotiurge.playbackprobe`. Xcode needs
a signed-in account for team `78A5P57U23` (Xcode Settings > Accounts) to create
that development profile. No profile from another app is reused.

Engine checks: run these in `engine/` with
`CARGO_TARGET_DIR=../../../target/ios-probe`:

- `cargo test --locked`: tests the PCM queue order, silence padding, RMS and
  credential JSON serialization. This host test does not exercise iOS Keychain;
  reusable Keychain restoration was observed separately on the device.
- `cargo clippy --locked --all-targets -- -D warnings`
- `cargo fmt --check`

## Install and run on the iPhone

```sh
D=<connected-device-identifier>   # substitute the CoreDevice identifier
xcrun devicectl device info lockState --device $D      # expect passcodeRequired false
xcrun devicectl device install app --device $D target/ios-probe/xcode/build/Release-iphoneos/PlaybackProbe.app
xcrun devicectl device process launch --device $D com.sergeserbinenko.spotiurge.playbackprobe
# Optional, starts playback over the cable once connected:
#   ... process launch --device $D com.sergeserbinenko.spotiurge.playbackprobe -- -autoplay spotify:album:…
```

At the first launch, the phone may ask to trust the developer profile in
Settings > General > VPN & Device Management.

## Gate procedure (the human steps are on the phone)

1. Terminate the Spotify app. Record the proof:
   `xcrun devicectl device info processes --device $D | grep -i spotify`
   must print nothing.
2. In the probe, tap **Sign in to Spotify (streaming)** and approve. The status
   line shows `Connected to Spotify`.
3. Enter an album or playlist URI with several short tracks. Tap **Play context
   on this iPhone**. The music plays from the phone speaker.
4. Lock the phone for at least 10 minutes and at least 3 track changes. The
   lock screen shows the probe's Now Playing item.
5. Interruption: start and end a call or Siri request, then confirm that
   playback resumes. Route change: connect and then disconnect Bluetooth or
   wired headphones; playback pauses on disconnect.
6. Connect handoff, both directions:
   1. Use another Spotify client to move playback to "Spotiurge Probe
      (iPhone)".
   2. Move it back to that client. The probe records `stopped` or `paused` and
      a `controller` event.
   3. Tap **Take over from active Connect device**.
   Keep the Mac muted.
7. Copy the evidence log off the phone:
   `xcrun devicectl device copy from --device $D --domain-type appDataContainer --domain-identifier com.sergeserbinenko.spotiurge.playbackprobe --source Documents/evidence.jsonl --destination .qa/ios-playback/evidence.jsonl`

The log at `Documents/evidence.jsonl` holds these entries:

- Track events: URI, title, artists and duration.
- Play, pause, stop and position.
- Counters: decoded, rendered and silent frames, and the RMS of the last
  rendered block.
- A 15-second heartbeat: app state, `protected_data_available` (false while the
  passcode lock is engaged), engine state, audio category, other-audio-playing
  and output port types.
- Interruptions, route changes and Connect controller changes.

The log never contains audio, tokens, credentials or authorization responses.

Pass criteria:

- `decoded_frames` and `rendered_frames` rise in step while
  `protected_data_available` is false, `app_state` is `background`, and the
  category is `AVAudioSessionCategoryPlayback`.
- The RMS is non-zero.
- At least 3 `track` events occur within the locked window.
- The Spotify process is absent.

A synthetic tone, the simulator, or the Spotify app's own audio does not pass.

## Known probe limits (not production decisions)

- The sign-in and Keychain code is research scaffolding. Before production,
  fail closed if secure-random generation fails, surface Keychain errors and
  update an existing credential without deleting it first. Bound the loopback
  listener lifetime and request size and require the exact redirect path.
  These changes are not part of the recorded device builds.
- The output stops after 5 idle minutes. iOS then suspends the app, and a
  suspended app cannot answer Connect. When the app returns to the foreground,
  it replaces the stale session using the stored credential.
- If Spotify drops the session during playback, the probe reconnects with
  backoff and restores the exact queue through `Spirc::restore_playback`.
- The render path uses `try_lock` on a mutex. A lock-free SPSC ring buffer is
  the upgrade if silent blocks appear in the counters.
- No audio cache, normalisation or gapless tuning: the engine uses librespot's
  defaults.

## Sign-in regressions

Current source treats a callback with missing, duplicate or different `state`
as stray traffic and keeps listening. A denial bound to the current state still
fails sign-in. Media-services resets pause the Rust player, flush PCM, replace
the audio engine and source node, restore the Playback category, and bind the
configuration observer to the new engine. Audio stays stopped until a screen or
lock-screen playback request. These changes have host regressions and iPhoneOS
type checking, not a new physical-device reset test.

`probes/ios-playback/test-signin.sh` runs host-only callback and credential
regressions. It checks every TCP request-line split, malformed/oversized requests,
replacement failures, and a disposable native Mac Keychain round trip. The
probe accumulates a complete bounded request line before parsing. Credential
replacement updates in place and reports its status; a failed write preserves
the prior item and never records `credential_stored`. This check never accesses
the iPhone or its real credential.
