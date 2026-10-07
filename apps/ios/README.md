# Spotiurge for iPhone (developmental)

A native SwiftUI candidate for Spotiurge on iPhone. It follows the desktop
interface contract in `DESIGN.md` and the phone brief in
`.impeccable/surfaces/ios-interface.md`. The forest/mist palette, quiet bundled
mountain scene and flat mint ridge mark match the desktop and T3 Pretty. The
background stays static; Liquid Glass belongs to native navigation and controls.
Artwork no longer causes a second request for ambient lighting. Native hard
scroll-edge treatment keeps content beneath the tab bar and mini-player
blurred, and the compact accessory caps Dynamic Type at XXX Large. Full-width
blocks have equal 16-point margins; Settings hints, grouped surfaces and disabled
controls use the shared forest/mist tokens.

**Status: developmental, not for distribution.** The production iOS
architecture is still gated by `docs/reviews/spotiurge-ios/playback-gate.md`.
This app does not use the official Spotify iOS SDK and is not a remote control
for the Spotify app. It has not been installed on a physical iPhone. No
TestFlight build or App Store Connect record exists.

## What works

- **Home (For you).** It has the same workflow and state rules as the desktop
  discovery desk (`src/ui/discovery.rs`, `src/app.rs`):
  - Familiar / Balanced / Adventurous exploration.
  - Automatic picks on the desktop schedule, from your saved taste and ratings.
  - Refresh, a taste editor, and more-like-this / less-like-this feedback with
    44-point targets and swipe actions.
  - Saved mixes and the newest ten AI history entries.
  - An honest status line: picks ready, not found on Spotify, or not checked,
    with the reason.
  - Unmatched picks stay collapsed, with a Search Spotify shortcut.
- **Private cloud.** Pairing stores a token in the Keychain, bound to the HTTPS
  origin. Sync is automatic after edits, on launch and on return to the
  foreground, and can also be run by hand. Recommendations come from
  `services/private-cloud`. The details are below.
- **Library and Search.** Liked songs, playlists and catalogue search use the
  Spotify Web API. The app uses the desktop's shared app and asks only for read
  scopes.
- **Now Playing.** The floating mini-player sits in the tab-bar accessory. The
  Now Playing sheet shows the scenery, a read-only position, the
  transport and feedback.
- **Settings.** Spotify sign-in, playback status, pairing and sync, discovery
  options, what leaves the phone, the demo-data switch and attribution.
- **Demo mode.** It has synthetic data, the same titles and reasons as the
  desktop demo, and generated covers. It is labelled on every screen. It never
  saves, syncs, plays or sends anything. Start it with
  `-SpotiurgeDemo home|error|onboarding` or the Settings switch. Real and demo
  state never share a model:
  - The real session is created once per process and kept across demo round
    trips, so there is only one replica writer.
  - While demo data shows, the real session is suspended. Automatic picks,
    the sync debounce and requests in flight are cancelled, and no new
    private-cloud, AI or Spotify Web API request starts. A request the server
    had already received cannot be unsent; its answer is dropped. Real data,
    settings and playback are left as they are.
  - The demo session has its own player, which never touches the playback
    engine or the playback credential. Settings shows "Demo: no audio".

## Playback: development engine only

`build.sh` links the playback probe's Rust engine
(`probes/ios-playback/engine`) as an explicit, development-only import. The
probe source is not modified. The app owns `AVAudioSession` (`.playback`,
`.longFormAudio`, background audio), the `AVAudioSourceNode` render path,
interruptions, route changes, Now Playing and remote commands, ported from
the probe.

Without the engine (`build.sh --no-engine`), playback says it is not in this
build. With the engine, playback says it is not signed in until you sign in.
Nothing falls back to controlling another app.

Limits:

- **Stock login is refused.** The engine is pinned to stock `crmne/librespot`
  `23fc42c`, which announces an iPhone identity. Spotify refuses that login
  (builds 1 and 2 in the gate record). Device playback only worked with the
  `config::OS = "macos"` experiment in an ignored local copy. That change
  needs a maintainer-owned librespot fork pinned to a commit and an upstream
  proposal. This tree does not reference any ignored path.
- **One track or context per load.** The probe ABI loads one context URI.
  Albums and playlists play as contexts. A list of picks or a saved mix starts
  with its first track, and the app says so. Track-list loading, queueing and
  seeking need the production engine ABI.
- **Simulator audio.** The engine also builds for `aarch64-apple-ios-sim`. A
  simulator does not satisfy the playback gate.

## Network access and personal data

| Destination | Sent | When |
|-------------|------|------|
| Private cloud (default `https://private-cloud-production.up.railway.app`) | Bearer device token. The discovery document: taste text, ratings (track URI, title, artist, rating), saved mixes, AI history. | Sync, after pairing |
| Private cloud `/v1/recommendations` | Taste text, plus title, artist and rating of up to 100 ratings, and the exploration value. No URIs, account identity or Spotify grants. The server asks `gpt-6-luna` only. | Refresh or automatic picks |
| `accounts.spotify.com` | PKCE sign-in in the system sheet, and code or refresh exchange | Sign-in, and token refresh |
| `api.spotify.com` | Web API reads: profile, liked songs, playlists, search, discovery matching | Library, search, matching |
| Spotify access points (librespot) | Streaming session | Engine builds after playback sign-in |
| Cover image hosts | Image requests for real covers | Showing covers |

There is no telemetry. Spotify grants never go to the private cloud or the AI.
The cloud token, the Web API refresh grant and the playback credential are
separate Keychain items. They use `AfterFirstUnlockThisDeviceOnly`.

## Mix slots and feedback retention

Saved mixes can be removed. New saves reuse removed slots first, then the oldest
slot once 100 exist, matching desktop; imported legacy slots remain readable
and removable. Manual recommendation requests honor the AI retry deadline.
Attempt and pending-refresh checkpoints, failure counts, cooldowns and pairing
suspension persist per installation, outside the cloud document. Restarting
keeps the same limits, and the local writer finishes before AI access.
The loopback sign-in parser waits for a complete bounded request line, including
when TCP splits the state or CRLF across callbacks.

The shared version-one document retains at most 500 ratings and clears, with a
logical retention cutoff. A bounded local `pending_feedback` stamp map persists
fresh unsent feedback across restarts. Sync imports the remote clock and
re-expresses pending ratings pruned by its cutoff before uploading, while
acknowledged old replicas stay forgotten. Acknowledgments clear only dispatched
stamps; edits made during a sync remain pending. Removed recommendation inputs
invalidate in-flight AI answers. The pending map never reaches the cloud or AI.

## Safeguards (beyond the probe scaffold)

- **Listener startup.** The system sign-in sheet opens only after its loopback
  listener is ready. An asynchronous bind failure or waiting state ends the
  attempt with a retryable port error. Canceled listener, browser and request
  callbacks cannot affect a replacement sign-in. The ten-minute deadline also
  covers listener startup.
- **Randomness.** PKCE verifier and state generation fail closed if
  `SecRandomCopyBytes` fails.
- **Keychain.**
  - A read distinguishes "missing" from "locked or failing", so a locked store
    never looks like a signed-out one. A locked pairing token shows "Keychain
    unavailable", not "Not paired", and a locked Spotify grant is retried when
    the app returns to the foreground.
  - A write updates in place, or adds the item, and then reads it back. An
    existing credential is never deleted before a successful replacement.
  - A failed delete is reported, including on Spotify sign-out.
- **Spotify Web grant.**
  - Being offline, a Spotify server error or a locked Keychain never removes
    the grant. Only a missing grant, or one Spotify rejects, asks for a new
    sign-in.
  - A refresh or library answer that arrives after sign-out is dropped, so it
    cannot store a rotated grant or refill the library.
- **Loopback redirect.**
  - The listener is bound to 127.0.0.1 and lives for at most 10 minutes (the
    desktop's `LOGIN_TIMEOUT`).
  - It answers at most 8 requests.
  - It accepts only `GET /login` with exactly one matching `state`.
  - Every other path gets a 404 and does not end the flow.
- **No secrets in errors.** Errors shown to the user are fixed messages.
  System error text, URLs and token response bodies are never logged or
  displayed.
- **Private-cloud requests.**
  - The origin is validated: HTTPS only, no path, query or credentials.
  - Redirects are refused, so the bearer token cannot leave the origin.
  - Responses are bounded to 1 MiB + 200 bytes.
- **Local state.**
  - Unknown fields and schemas fail closed, matching serde
    `deny_unknown_fields`.
  - The replica file is replaced atomically, with complete file protection
    until first unlock. Saves are written by one writer, and the latest
    snapshot wins.
  - Each load gets a fresh writer identity.
  - Exploration and replica writes require a successful load. A failed load
    cannot be replaced by the default replica, even when pairing is changed.
  - Merges use per-record Lamport clocks. An acknowledged sync never undoes a
    newer local edit.

## Layout

- `SpotiurgeCore/`: a Swift package with no UI and no Keychain.
  - It ports `src/discovery.rs` (document, validation, merge, history bounds,
    automatic schedule), `src/discovery_cloud.rs` (sync and recommendations)
    and the catalogue-matching helpers from `src/backend.rs`.
  - It also holds a small Spotify Web API client.
- `Spotiurge/`: the app target, Swift 6 with main-actor default isolation.
  The project uses file-system synchronized groups, so new files need no
  project edits.
- `SpotiurgeTests/`: the hosted app-model tests.
- `build.sh`: stages ignored copies of Inter (from the desktop's pinned
  `fastframe-fonts` checkout) and the app icon (from `assets/brand`). It
  builds the engine, then the app, into `target/ios-interface/`.

## Build and check

```sh
rustup target add aarch64-apple-ios-sim aarch64-apple-ios --toolchain 1.98.0   # once
apps/ios/build.sh                                  # Simulator, with engine
apps/ios/build.sh simulator --no-engine            # Simulator, no engine
apps/ios/build.sh device CODE_SIGNING_ALLOWED=NO   # iphoneos compile check
(cd apps/ios/SpotiurgeCore && swift test --scratch-path ../../../target/ios-interface/core)
```

The app-model tests in `SpotiurgeTests/` are a hosted unit-test bundle. They
cover the Spotify grant across offline launches and sign-out, the single real
session across demo round trips, and pairing with a locked Keychain. They use
in-memory dummy secrets and a stub `URLProtocol`, so they never read real
sign-in or pairing items or reach the network. A separate native-store test
stores, replaces and deletes a disposable dummy Keychain item under a UUID
account. Under tests the host app starts in demo mode and creates no real
session. Build them, then run them on a Simulator:

```sh
xcodebuild -project apps/ios/Spotiurge.xcodeproj -scheme Spotiurge \
    -destination 'generic/platform=iOS Simulator' \
    -derivedDataPath target/ios-interface/tests ARCHS=arm64 build-for-testing
xcodebuild test-without-building -xctestrun target/ios-interface/tests/Build/Products/*.xctestrun \
    -destination 'platform=iOS Simulator,name=<simulator>'
```

The package tests do four things:

- port the desktop merge, history, schedule and matching tests;
- check the wire format against desktop JSON;
- run a stubbed private-cloud conflict round;
- run `services/private-cloud/server.py`'s own `valid_document` on a document
  written by this package.

The `iOS core` workflow runs those package tests and wire checks on GitHub.
Hosted app-model tests and app builds are checked locally. Results and the
26 native Simulator captures are recorded in the current
[`scenery/verification.md`](../../docs/reviews/spotiurge-ios/scenery/verification.md).
The earlier amber interface's results remain in
[`interface-verification.md`](../../docs/reviews/spotiurge-ios/interface-verification.md).

A signed device build needs `DEVELOPMENT_TEAM=78A5P57U23` and a registered
development bundle ID (`com.sergeserbinenko.spotiurge.dev`). This tree has not
performed one.

## Before this could ship

- **librespot identity.** Pin a librespot fork for the iOS identity, and the
  audio-key retry fix (see the gate record), and propose both upstream.
- **Production engine ABI.** Grow it out of the probe: track lists, queue,
  seek, an SPSC ring buffer, and events with artwork.
- **Playback gate.** Close the remaining criteria on a real iPhone:
  interruption, route change, and Connect handoff in both directions.
- **Distribution.** Decide whether a librespot-based engine is acceptable on
  TestFlight or the App Store. Create the App Store Connect record, signing,
  privacy and export compliance (`packaging/ios/README.md`).
- **Real-data checks.** Signed-in library, search and matching, paired sync
  against the deployed service, a live Luna answer, a VoiceOver pass, and
  motion and performance on device.

Latest review fixes preserve pending love, less and clear actions against an
existing higher-stamped cloud rating, use a restart-safe 15-second retry for
AI-slot contention, and ignore stale/unbound sign-in callbacks. Current-state
Spotify denials still fail. These core changes pass 45 Swift tests, 16 hosted
Simulator tests and an unsigned device compile without the playback engine.
No physical phone was accessed and these checks add no audio proof.

Playback Forget now cancels browser/token work and reconnect, stops audio and
clears Now Playing before deleting the Keychain item. Failed deletion is
reported while playback stays stopped. Generation tickets reject queued
credential/event callbacks even after a new sign-in. This follow-up passes
45 core tests and 17 hosted Simulator tests with dummy secrets, an unsigned
generic iPhoneOS build without the engine, and iPhoneOS typechecking of all
Swift sources with `SPOTIURGE_ENGINE` enabled. The latter checks the bridge
source; it does not link or run a production engine. No phone was accessed.

Reconnect status now clears active playback on connection loss, failed retries
and terminal connection errors, and resumes only on fresh engine events. The
actual event handler passes a scripted hosted model regression; all 18 hosted
Simulator tests pass without audio. The unsigned no-engine iPhoneOS build and
engine-enabled source typecheck also pass. The Swift core is unchanged from
the prior 45-test run. No new device playback is claimed.
