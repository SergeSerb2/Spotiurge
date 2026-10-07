# iPhone independent-playback gate: probe evidence

Recorded on October 6 and 7, 2026 (UTC). The probe source and procedure are in
[`probes/ios-playback/`](../../../probes/ios-playback/README.md). This record
does not select a production iOS architecture.

## Verdict

**Gate status: NOT PASSED (partial). Device testing is stopped at Serge's
request.**

- **Passed: locked background playback.** Build 8 played for more than 17
  continuous minutes while locked. That run had 4 natural track changes, the
  Spotify app was not running, and one Spotify session drop was recovered
  automatically.
- **Still required:**
  - a deliberate interruption and resume,
  - a deliberate route change,
  - Connect handoff in both directions.

  The interruption seen during the run is not valid evidence (see below).

Device checks stopped at Serge's request on October 7, after build 9 was
installed. He reported informally that "its all good". That report is not
instrumented evidence for the remaining criteria.

## Capability facts established from source and device

- **The Spotify iOS SDK / App Remote is a controller.** It drives the Spotify
  app and is not an engine, so it is out of scope for this gate.
- **The pinned librespot fork has no iOS audio backend.** The probe implements
  librespot's public `Sink` trait and feeds `AVAudioSourceNode`. The app owns
  `AVAudioSession` (`.playback`, `.longFormAudio`, `UIBackgroundModes` =
  `audio`; no separate entitlement).
- **The pinned librespot fork compiles for `aarch64-apple-ios`** with Rust
  1.98.0 and the root lockfile's versions.
- **TLS roots:** `rustls-native-certs 0.8.4` has no iOS system store and would
  leave the trust store empty. The probe uses librespot's
  `rustls-tls-webpki-roots` feature.
- **Spotify identity on iOS needs a dependency change.** Unpatched librespot
  announces itself as an iPhone client, and every failure in the table below
  came from that identity. The patch in
  [`librespot-ios-os.diff`](librespot-ios-os.diff) makes `config::OS` report
  `"macos"` on iOS. With it, the session behaves like the desktop session that
  uses the same keymaster grant. The patch follows the precedent of
  librespot's own Android-as-Linux spoof in `handshake.rs`. It was applied only
  in an ignored local copy, through `cargo --config patch`. The probe manifest
  and lockfile still pin `crmne/librespot@23fc42c`.

| Build | librespot identity | Observed on device |
|-------|--------------------|--------------------|
| 1, 2 | stock (iPhone platform) | The OAuth token was issued, then the access point refused login 6 times with `TryAnotherAP`. |
| 3 | handshake mapped to Linux, client ID overridden to keymaster | Access-point login succeeded; login5 answered `BAD_REQUEST`. |
| 4 | handshake mapped to Linux, iOS client ID | Access-point login succeeded; login5 answered `INVALID_CREDENTIALS`. |
| 5 and later | `config::OS = "macos"` on iOS | Connected from the stored Keychain credential, with no new sign-in. |

## Device run (build 6), 2026-10-06 UTC

Environment:

- iPhone 17 Pro Max (iPhone18,2), iOS 27.2 (24B5089g), connected by cable.
- Xcode 27.0 (27A266a).
- Signing: Apple Development, team `78A5P57U23`, Xcode-managed team profile.
  The application identifier is
  `78A5P57U23.com.sergeserbinenko.spotiurge.playbackprobe`.

Timeline:

- **23:17:05.** The probe connected to Spotify Connect from its stored
  credential. Playback of Daft Punk's *Discovery* was started over the cable
  with a launch argument.
- **23:17:06.** "One More Time" started. Serge confirmed that he heard it
  ("its playing now").
- **23:20:42.** The phone locked. **23:20:44:** the app went to the
  background. From the locked heartbeats:
  - `protected_data_available` was false,
  - the category was `AVAudioSessionCategoryPlayback`,
  - the engine was running,
  - the output was `Speaker`,
  - `other_audio_playing` was false.
- **Track changes while locked:** "Aerodynamic" at 23:22:26 and "Digital Love"
  at 23:25:58.
- **Decoding:** 10,520,384 frames had been decoded at 23:21:04 and 34,552,252
  at the end of the run. There were no silent render frames until the session
  ended. The RMS of locked render blocks ranged from 0.0073 to 0.35.
- **Unlocks:** Serge briefly unlocked the phone at 23:27:38 for about 20 s and
  relocked it at 23:27:58.
- **23:30:05.** The access point closed the session ("Connection to server
  closed"). Spirc reported an unexpected shutdown, and the music stopped at
  23:30:09. Build 6 had no reconnect.
- **Spotify app:** absent in all 14 device process snapshots taken between
  23:21 and 23:33.

Result: 9 min 25 s of continuous background playback, including 6 min 56 s of
uninterrupted locked playback, with 2 locked track transitions. This does not
meet the 10-minute continuous criterion.

Unattended events after the run were not part of a test, and Serge's actions
are unknown: a route change to `BluetoothA2DPOutput` at 23:40:01, a return to
`Speaker` and an interruption at 00:34:30.

## Locked run (build 8), 2026-10-07 UTC

Serge reported "locked 10:32" in Vancouver time; the device log has the exact
edges:

- **05:32:26.** The probe launched and connected from the stored credential.
- **05:32:27.** Serge tapped Play.
- **05:32:28.** "One More Time" started.
- **05:32:32.** The phone locked. It was unlocked at 05:32:39 and relocked at
  **05:32:49**. No unlock follows that last lock in the log.

From 05:32:49 to at least 05:50:11 (17 min 22 s), heartbeats every minute
showed:

- `app_state` was background and `protected_data_available` was false,
- the engine was running and the category was `AVAudioSessionCategoryPlayback`,
- the output was `Speaker`.

Decoding and rendering:

- Decoded frames rose from 586,048 to 46,776,000 (about 17.5 min of 44.1 kHz
  audio), and rendered frames kept pace with them.
- Silent render frames totalled 59,989 (1.36 s). All of them occurred during
  the automatic reconnect.

Track changes while locked:

| Time | Track |
|------|-------|
| 05:37:49 | Aerodynamic |
| 05:41:24 | Digital Love |
| 05:46:25 | Harder, Better, Faster, Stronger |
| 05:50:11 | Crescendolls |

Session drop and restore:

- **05:40:52.** Spotify closed the playing session.
- **05:40:55.** Build 8 reconnected from the stored credential.
- **05:40:56.** It restored the exact queue with `restored: true`.
- **05:40:57.** "Aerodynamic" continued from where it had stopped; it ended at
  05:41:24, matching its 3:32 length.
- **Side effect:** an audio-key request timed out during the drop. Stock
  librespot then "continues without decryption", which produced about 1 s of
  junk-decode warnings and possibly a short audible glitch.

Spotify app: absent in all 18 device process snapshots from 05:33:22 to
05:49:15. Every snapshot reported `passcodeRequired: true`.

**Probe defect found in this run:** at launch, both the app delegate and the
scene's first foreground started a session, so two Connect sessions ran. When
an interruption arrived at 05:35:36 (cause unknown, not a scripted test), the
probe's pause went to the idle session ("SpircCommand::Pause will be ignored
while Not Active"). The engine restarted after a configuration change, and
playback continued. Therefore this run is not interruption evidence. Build 9
starts the foreground refresh only after a real return from the background.
Its launch log confirms a single session.

## Builds attempted

| Build | Result |
|-------|--------|
| Host `cargo test`, `clippy -D warnings`, `fmt --check` | pass (2 tests) |
| `aarch64-apple-ios` release staticlib | succeeded |
| Unsigned app build | succeeded |
| First signed attempt | failed: Xcode account login rejected |
| Signed builds 1 to 8 (after Serge signed in to Xcode again) | succeeded; installed with `devicectl` |
| Signed build 9 (fixes the duplicate session at launch) | succeeded. Installed at 05:52 UTC and launched at 05:52:58 with a single connect request; connected from the stored credential at 05:53:00. |

Changes in builds 7 and 8:

- After a lost session, the probe reconnects with backoff (1 to 32 s) and
  restores the exact queue through the fork's `Spirc::restore_playback`.
- The audio output stops after 5 idle minutes.
- When the app returns to the foreground while idle, it replaces the stale
  Connect session left by iOS suspension.
- The last error appears on screen.

## Remaining steps (paused by Serge; build 9 is already installed)

When Serge resumes device testing, run these with the Spotify app closed:

1. Unlock the phone, open the probe and tap **Play context on this iPhone**.
2. Ask Siri something short while music plays, and confirm that playback
   resumes afterwards.
3. Connect Bluetooth headphones, then disconnect them. Playback should pause
   on disconnect. Tap **Play** to resume.
4. Connect handoff, with the Mac muted:
   1. In another Spotify client, choose "Spotiurge Probe (iPhone)" (to the
      phone).
   2. Choose that client again (away from the phone).
   3. In the probe, tap **Take over from active Connect device**.

## Known limits relevant to architecture

- An idle app in the background is suspended by iOS, so it cannot receive a
  Connect transfer until it is opened. Observed: no events between 04:46 and
  05:12 UTC on October 7.
- The macOS identity patch must live in a maintainer-owned librespot fork
  pinned to a commit, with a generic upstream proposal, before any product
  build. It is not committed in this repository.
- Distributing librespot-based playback through TestFlight or the App Store is
  a separate policy question that this probe does not answer.

Raw logs (`.qa/ios-playback/`) stay local and ignored. They hold no audio,
tokens, credentials or authorization responses.
