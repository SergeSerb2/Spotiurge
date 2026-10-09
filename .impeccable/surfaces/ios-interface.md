---
version: 1
slug: "ios-interface"
primary_target: "apps/ios/Spotiurge"
related_targets: ["DESIGN.md",".impeccable/surfaces/ios-spotiurge.md",".impeccable/surfaces/src-ui-mod-rs.md","src/theme.rs","src/ui/material.rs","src/ui/motion.rs","src/ui/discovery.rs","assets/brand/spotiurge-mark.svg"]
---

# Spotiurge for iPhone: developmental interface

Status: a working native candidate, built on October 6 and 7, 2026, to the
whole-app visual scope Serge approved. It was built code first, with no
concept images. Evidence is Simulator only. The 26-state scenery before/after viewer is
`docs/reviews/spotiurge-ios/scenery/index.html`, with exact image/build hashes in
`scenery/captures.json`. Historical state differences are marked explicitly. The production architecture stays gated by
`docs/reviews/spotiurge-ios/playback-gate.md`. Physical-phone checks are
stopped at Serge's request.

## Contract carried from DESIGN.md

- **Scenery.** One original bundled mountain/lake photograph uses the desktop's
  centred aspect-fill crop and flat contrast wash. The forest room is
  #0E1110 / #F4F6F4; image opacity is 0.613402 / 0.53125 and the black/white
  wash is 0.612 / 0.68. It stays static, with no cover-light request or animation.
- **Glass.** Native Liquid Glass carries navigation and controls. Content sits
  directly on the scenery; empty-state content uses a quiet flat plate
  (#2C3A32 at 85% in dark, white at 68% in light).
- **Live controls.** Moss (#98D2AC / #27633F) marks primary play, the current
  tab and playing text/meter. Playing rows have no decorative leading bar.
  Play controls keep the established 44 / 36 / 72-point sizes. Disabled play
  uses a neutral surface with readable disabled ink, without a faded accent.
  Choices remain neutral inverted pills; the selected Love and boolean
  switches retain the existing recorded behaviour.
- **Choices.** The exploration fader selects with the neutral inverted pill
  (`text` fill, `room` label). It glides with `matchedGeometryEffect` over
  220 ms with exponential ease-out, plus a selection haptic. At accessibility
  text sizes the three channels stack vertically instead of truncating.
- **Opaque overlays.**
  - The taste editor and Discovery options sheets use `overlay`
    (#202A25 / #FFFFFF).
  - Now Playing uses the room.
  - The For you options are a sheet, not a translucent menu.
  - The demo banner and notices are opaque.
- **Type.** Inter Variable, ranked by weight:
  - display 26 bold, title 20 semibold, section 17 semibold;
  - row title 15 semibold, body 15, detail 13, caption 12 medium.
  - Every size is relative to a Dynamic Type style.
  - Times use tabular figures.
  - Navigation bar titles use Inter through `UIFontMetrics`.
- **Mark.** A flat mint ridge cut into three meter columns, from the shared
  SVG geometry. The header uses the bare glyph; the app icon uses the forest
  tile. The desktop assets are staged at build time, without duplicate sources.


## Phone structure

- **Tabs.** A system tab bar: Home (the For you desk), Library, Settings and
  a Search role tab. The mini-player is the bottom accessory.
- **Home.**
  - Header: the mark, "For you", a subtitle, refresh, options and the lamp
    Play.
  - Below the header: the exploration fader, then the status lines.
  - Pick rows are 56 points or taller, with a reason and Love/Less buttons.
    Each row also has leading and trailing swipe actions.
  - "Couldn't play" and "Your AI history" start collapsed, followed by saved
    mixes.
- **Now Playing.** A large cover, title and artist with feedback, a read-only
  position (the engine has no seek yet), and the transport. The device line
  reads "This iPhone" or "Demo: no audio".
- **Controls.**
  - Every control has a 44-point hit target.
  - Icon buttons have accessibility labels.
  - Rows combine into one VoiceOver element. The current row has the
    selected trait and the value "Current track", whether it plays or is
    paused.

## Honest states

- **Playback.** "Not in this build", "Not signed in", "Connecting" and
  "Unavailable" each come with a plain reason. In demo, Settings shows only
  "Demo: no audio". Playback never falls back to controlling the Spotify app.
- **Keychain.** A locked pairing token reads "Keychain unavailable" with an
  unlock instruction, never "Not paired" or "Pair again".
- **Picks.**
  - The status line counts picks ready to play, not found on Spotify, and not
    checked yet, and gives the catalogue reason.
  - An unpaired phone says so.
  - Errors are neutral notices.
- **Demo.** Demo data is labelled on every screen, is a separate session, and
  disables feedback, refresh and cloud requests. The one real session is
  suspended while it shows.

## Finish corrections

- Native hard bottom scroll-edge treatment blurs content below the mini-player and tab bar; the accessory caps Dynamic Type at XXX Large.
- Full-width blocks use symmetric 16-point margins and no stray dividers.
- Settings hints and disabled controls use explicit readable ink; disabled CTAs use neutral opaque pills.
- Dark grouped Settings/options rows use forest surfaces; light uses white.

## Open items

- **Real data.** Signed-in data, paired sync, a live Luna answer and audio
  have not been exercised in this app.
- **Motion and accessibility.** Motion quality is unmeasured. There has been
  no VoiceOver walkthrough, and no landscape or iPad layout.
- **System chrome.** The system search keyboard, system sheets and the tab bar
  follow iOS styling. That is intended native convention.
