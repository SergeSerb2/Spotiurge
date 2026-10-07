---
version: 1
slug: "ios-interface"
primary_target: "apps/ios/Spotiurge"
related_targets: ["DESIGN.md",".impeccable/surfaces/ios-spotiurge.md",".impeccable/surfaces/src-ui-mod-rs.md","src/theme.rs","src/ui/material.rs","src/ui/motion.rs","src/ui/discovery.rs","assets/brand/spotiurge-mark.svg"]
---

# Spotiurge for iPhone: developmental interface

Status: a working native candidate, built on October 6 and 7, 2026, to the
whole-app visual scope Serge approved. It was built code first, with no
concept images. Evidence is simulator only:
`docs/reviews/spotiurge-ios/interface-review.html` and
`interface-captures.json`. The production architecture stays gated by
`docs/reviews/spotiurge-ios/playback-gate.md`. Physical-phone checks are
stopped at Serge's request.

## Contract carried from DESIGN.md

- **Room.**
  - The room is `room` (#0B0D12 / #E9EBF1).
  - The key light comes from the cover, from the upper left.
  - The counter-light is the same colour turned a third of the way round the
    hue wheel, from the lower right.
  - With no cover, the key light is lamp amber, so the room reads amber and
    teal.
  - Light strength is 0.24 in dark and 0.20 in light.
  - The room crosses over in 600 ms, immediately with Reduce Motion.
- **Glass.**
  - System Liquid Glass (`glassEffect`) carries only controls: the header keys,
    the exploration track, empty-state panes, the tab bar and the mini-player
    accessory.
  - Rows, covers and text sit on the room.
- **One lamp.** Amber marks primary play discs only:
  - 44 points on the desk, 36 in the console accessory, 72 in Now Playing;
  - the selected tab;
  - on the playing row: a 3-point bar at 46% of the row height, an EQ glyph
    and the title in lamp text (#FFB547 / #653706).

  DESIGN.md's recorded exceptions also apply: a selected Love, and switches.
  - The lamp is unlit (desaturated, at 35%) when there is nothing to play.
  - Everything else is neutral. Content tint is `text`.
- **Choices.** The exploration fader selects with the neutral inverted pill
  (`text` fill, `room` label). It glides with `matchedGeometryEffect` over
  220 ms with exponential ease-out, plus a selection haptic. At accessibility
  text sizes the three channels stack vertically instead of truncating.
- **Opaque overlays.**
  - The taste editor and Discovery options sheets use `overlay`
    (#1C202B / #FFFFFF).
  - Now Playing uses the room.
  - The For you options are a sheet, not a translucent menu.
  - The demo banner and notices are opaque.
- **Type.** Inter Variable, ranked by weight:
  - display 26 bold, title 20 semibold, section 17 semibold;
  - row title 15 semibold, body 15, detail 13, caption 12 medium.
  - Every size is relative to a Dynamic Type style.
  - Times use tabular figures.
  - Navigation bar titles use Inter through `UIFontMetrics`.
- **Mark.** The surge S is drawn from the SVG geometry: two 17-unit arcs,
  skewed by -6.84 degrees, on the smoked tile with a 27/128 corner radius.

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

## Open items

- **Real data.** Signed-in data, paired sync, a live Luna answer and audio
  have not been exercised in this app.
- **Motion and accessibility.** Motion quality is unmeasured. There has been
  no VoiceOver walkthrough, and no landscape or iPad layout.
- **System chrome.** The system search keyboard, system sheets and the tab bar
  follow iOS styling. That is intended native convention.
