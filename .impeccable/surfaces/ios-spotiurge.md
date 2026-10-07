---
version: 1
slug: "ios-spotiurge"
primary_target: "candidate native iOS architecture (production gate open)"
related_targets: ["src/theme.rs","src/ui/material.rs","src/ui/motion.rs","assets/brand/spotiurge-mark.svg",".impeccable/surfaces/src-ui-mod-rs.md","probes/ios-playback/"]
---

# Spotiurge for iPhone: transferable design brief

Status: design brief plus a candidate architecture. Root requested the
candidate on October 7, 2026, after the locked-playback criterion passed on the
real iPhone (build 8: 17 min 22 s locked, 4 track changes, Spotify app absent).
The interruption, route-change and bidirectional Connect criteria are not yet
instrumented, so the gate in `docs/reviews/spotiurge-ios/playback-gate.md` is
still open. The candidate below is a recommendation, not a final selection. Serge authorized the full visual scope and asked for a working
native interface first, so no separate concept-image round is planned. The
desktop contract in `src-ui-mod-rs.md` is the visual authority. This brief
translates that contract to a phone. It does not invent a second identity.

## Direction contract (carried from desktop)

The thesis is the studio control room. Music lives in the lit live room behind
the glass, and Spotiurge is the console in front of it. Spotiurge avoids the
category default of a flat black Spotify clone with green accents and card
grids.

The phone's own world:

- Smoked glass controls float over a dim room, which is washed by light derived
  from the playing cover.
- Each pane has a lit top rim and a shaded bottom rim, with a faint specular
  sheen.
- One VU-amber lamp marks live state only: playing, the primary play control
  and the current selection. Inactive items stay neutral.
- The type is Inter, and rank is carried by weight.
- The mark is the amber surge S on a smoked glass tile
  (`assets/brand/spotiurge-mark.svg`).

The story: one-handed, Serge sees what plays now and what to play next. Moving
between Home, Library and Search feels like a fader moving between channels.
Every control answers immediately, even when the cloud or AI is unavailable.

## Tokens to reuse verbatim (source of truth: desktop code)

| Role | Dark | Light | Source |
|------|------|-------|--------|
| window/room | #0B0D12 | #E9EBF1 | `theme.rs` |
| panel | #14171F | #FCFCFE | `theme.rs` |
| surface | #1C202A | #EEF0F5 | `theme.rs` |
| text | #F3F2EF | #15171C | `theme.rs` |
| secondary | #A7ADBB | #4D5463 | `theme.rs` |
| dim | #707787 | #868D9C | `theme.rs` |
| accent (VU amber) | #FFB547 | #A95C06 | `theme.rs` |
| on accent | #1F1303 | (palette) | `theme.rs` |
| danger | #FF6F73 | #C83344 | `theme.rs` |
| mark amber gradient | #FFD680 to #FF9228 | same | `spotiurge-mark.svg` |
| mark glass | #262C4A to #0A0C16 | same | `spotiurge-mark.svg` |

Glass recipes are in `material.rs::glass`. Each recipe gives
(opacity, sheen, rim top, rim bottom):

| Glass | Dark | Light |
|-------|------|-------|
| Pane | 0.72, 0.045, 0.11, 0.42 | 0.58, 0.55, 0.95, 0.10 |
| Console | 0.80, 0.06, 0.14, 0.45 | 0.72, 0.60, 1.0, 0.12 |

Popover glass is fully opaque in both themes. Well glass is a quiet inset.
The final desktop contract in `DESIGN.md` supersedes this preliminary brief:
small amber text uses `#653706` in light mode, choice controls stay neutral,
and the optional taste editor is capped at 560 points with 10-point field
corners. On/off switches are the existing amber exception.

Radii: the pane radius is 14, the console 18, popovers 12 and the gap 8. The
mark tile radius is 27/128 of its size. Phone layouts use a 4-point grid.

Motion (`motion.rs`):

- Feedback is 120 ms. State and selection changes take 220 ms. A page settles
  in 220 ms and rises 6 points. The ambient cover light crosses over in 600 ms.
- Motion uses an exponential ease-out, is finite and interruptible, and
  retargets from its current value.
- Reduce Motion makes every transition immediate.

## Phone translation

- **Glass as the control layer only.** Content (track rows, covers, lyrics)
  sits on the room. Glass carries navigation, the mini-player console, the
  transport, sheets and menus.
  - On iOS 26 and later, use system Liquid Glass with
    `glassEffect(_:in:)`, `.buttonStyle(.glass)` and `GlassEffectContainer`
    (one container per control cluster, per Apple's performance guidance).
    Tint the glass with VU amber only for the live or primary control.
  - Below iOS 26, or if Liquid Glass is unavailable, fall back to the desktop
    recipe: a translucent fill, sheen and two-tone 1-pixel rim over the cached
    cover light. Use `.ultraThinMaterial` only where text contrast stays at or
    above WCAG AA. No per-frame CPU blur, and no endless shimmer or decorative
    repaint.
- **Cover-lit room.** Derive 2 or 3 soft colour fields from the current
  artwork once per track, off the main thread, and cache them. Cross-fade them
  in 600 ms, or instantly with Reduce Motion. Clamp the fields so text over
  them keeps contrast in both themes.
- **Navigation.** Use a system tab bar (Liquid Glass) with Home (the For you
  radio desk), Library, Search and Settings. The live lamp marks the selected
  tab with the 220 ms glide. A floating mini-player console sits above the tab
  bar, inside the safe area. It shows cover, title/artist, play/pause (amber
  when playing), next and a Connect device glyph. Tapping it opens the Now
  Playing sheet with a large cover, scrubber and transport. In that sheet,
  Connect, queue and lyrics are glass buttons.
- **Home radio desk (same workflow as desktop).**
  - Header: a 26-point "For you" heading, then play and refresh.
  - Exploration strip: a segmented Familiar / Balanced / Adventurous control
    with a gliding lamp.
  - Below the strip: a one-line honest status, then 56-point rows. Each row
    has the cover, title, artist, duration and a bounded reason, plus
    more-like-this and less-like-this feedback. Each feedback control has a
    44-point hit target and a swipe-action equivalent.
  - The taste editor opens as a sheet on request.
  - Unmatched suggestions and AI history start collapsed.
  - Cached picks and the player stay usable when the GPT-6 Luna broker or the
    sync service fails.
- **Controls.** Every hit target is at least 44 by 44 points. Respect all
  safe areas, including the Dynamic Island. Play/pause and the selection
  changes give light haptics. Support Dynamic Type up to accessibility sizes:
  rows grow and never truncate controls. Use tabular figures for time, so
  state changes never shift geometry.
- **Lock screen and system.** Now Playing uses the same title, artist and
  artwork. Remote commands mirror the in-app transport. Connect
  device names follow the desktop pattern ("Spotiurge (iPhone)").

## Behavioral constraints carried into the eventual app

- Playback changes are optimistic. A stale backend response must never undo or
  flicker away the user's action.
- Recommendation and AI work stays off the main thread and the audio render
  thread. Recommendations always use GPT-6 Luna through the existing
  CLIProxyAPI subscription broker, with no model fallback.
- Local state is an offline, atomically replaced snapshot with per-record
  Lamport clocks. It preserves in-flight edits, and its history is bounded.
- Device credentials for the private cloud are origin-bound and stored in the
  Keychain. Spotify grants never go to the private cloud.
- No embedded browser. Sign-in uses `ASWebAuthenticationSession` (the system
  sheet), as the probe does.

## Candidate architecture: Rust engine plus SwiftUI (recommended, not final)

The structure is taken from what the probe proved on the device:

- **Rust core (static library, `aarch64-apple-ios`).**
  - Contains librespot session, player, Connect (Spirc) and metadata, at the
    desktop's pinned revisions.
  - Exposes a narrow C ABI that grows out of
    `probes/ios-playback/engine/include/probe_engine.h`:
    - lifecycle: start, connect (token or stored credential), disconnect;
    - playback: load, command;
    - PCM: render pull, stats;
    - events: a JSON line callback carrying metadata only;
    - credentials: one-shot callback.
  - Domain code that is shareable later (recommendation request shaping,
    catalogue matching, sync merge with Lamport records) can move into the
    same crate. Each move needs its own tests; do not port it wholesale.
- **Audio.**
  - The app owns `AVAudioSession` (`.playback`, `.longFormAudio`) and
    `UIBackgroundModes` with `audio`.
  - An `AVAudioEngine` with an `AVAudioSourceNode` pulls 44.1 kHz stereo f32
    PCM from a bounded queue (0.5 s). The decoder blocks; the render thread
    never blocks.
  - Production should replace the probe's `try_lock` with an SPSC ring buffer.
  - Interruptions, route changes, engine configuration changes, Now Playing
    and remote commands are handled in Swift, and they call into the engine.
  - Visualizer taps attach after EQ and before volume, per AGENTS.md.
- **Recovery.**
  - When Spotify drops a session during playback, reconnect with the reusable
    credential and use `Spirc::restore_playback`. The probe verified the
    position-exact restore on the device.
  - Stop the output after an idle timeout.
  - On the first foreground after a real background period, replace the stale
    session. Never start a second session at launch.
- **UI.**
  - SwiftUI on iOS 26 and later with Liquid Glass, with the fallback described
    above. UIKit only where SwiftUI lacks an API.
  - Swift 6 language mode. Render and audio callbacks are `nonisolated` and
    must never inherit main-actor isolation.
- **Credentials.**
  - The reusable Spotify credential is stored in the Keychain with
    `AfterFirstUnlockThisDeviceOnly`. The private-cloud device token gets a
    separate Keychain item.
  - Sign-in uses the probe's `ASWebAuthenticationSession` plus loopback PKCE
    flow.

## Blockers before production code

- **librespot identity patch.** Stock librespot announces an iPhone client, and
  Spotify rejects that login. The one-line fix
  (`docs/reviews/spotiurge-ios/librespot-ios-os.diff`, `config::OS = "macos"`
  on iOS) needs:
  - a maintainer-owned librespot fork pinned to a commit, with all librespot
    crates from that one fork;
  - a generic upstream proposal.

  Until then, device builds depend on an ignored local copy, and
  `probes/ios-playback/build.sh` produces an app that cannot log in.
- **TLS roots.** The iOS build must use librespot's `rustls-tls-webpki-roots`
  feature or a platform verifier. Native roots are empty on iOS.
- **Recovery side effect.** During a drop, an audio-key timeout makes stock
  librespot decode undecrypted data for about 1 s. Before shipping, prefer
  skipping or retrying the key over decoding without decryption, through the
  same fork.
- **Gate criteria still open.** Interruption and resume, route change, and
  Connect handoff in both directions need instrumented runs on build 9 or a
  later build.
- **Background Connect.** An idle app in the background is suspended by iOS,
  so it cannot receive a Connect transfer until it is opened.
- **Distribution.** Whether a librespot-based engine is acceptable for
  TestFlight or App Store distribution is unverified.
