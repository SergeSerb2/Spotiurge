---
name: Spotiurge
description: A studio control room for Spotify, smoked glass panes over a room lit by the playing cover, with one VU-amber lamp for whatever is live.
colors:
  dark-window: "#0b0d12"
  dark-panel: "#14171f"
  dark-surface: "#1c202a"
  dark-surface-hover: "#252a37"
  dark-surface-active: "#2e3444"
  dark-outline: "#2a2f3c"
  dark-rim: "#373b47"
  dark-text: "#f3f2ef"
  dark-secondary: "#a7adbb"
  dark-dim: "#707787"
  dark-lamp: "#ffb547"
  dark-lamp-hover: "#ffc770"
  dark-lamp-text: "#ffb547"
  dark-on-lamp: "#1f1303"
  dark-danger: "#ff6f73"
  dark-warning: "#f6d365"
  dark-overlay: "#1c202b"
  dark-shadow: "rgba(0, 0, 0, 0.59)"
  dark-pane-glass: "rgba(20, 23, 31, 0.72)"
  dark-console-glass: "rgba(20, 23, 31, 0.80)"
  dark-well-glass: "rgba(28, 32, 42, 0.55)"
  dark-hover-fill: "rgba(255, 255, 255, 0.06)"
  dark-selected-fill: "rgba(255, 255, 255, 0.095)"
  light-window: "#e9ebf1"
  light-panel: "#fcfcfe"
  light-surface: "#eef0f5"
  light-surface-hover: "#e3e6ee"
  light-surface-active: "#d6dae4"
  light-outline: "#d5d9e3"
  light-text: "#15171c"
  light-secondary: "#4d5463"
  light-dim: "#868d9c"
  light-lamp: "#a95c06"
  light-lamp-hover: "#914e05"
  light-lamp-text: "#653706"
  light-on-lamp: "#ffffff"
  light-danger: "#c83344"
  light-warning: "#856400"
  light-overlay: "#ffffff"
  light-shadow: "rgba(0, 0, 0, 0.19)"
  light-pane-glass: "rgba(252, 252, 254, 0.58)"
  light-console-glass: "rgba(252, 252, 254, 0.72)"
  light-well-glass: "rgba(238, 240, 245, 0.75)"
  light-hover-fill: "rgba(0, 0, 0, 0.045)"
  light-selected-fill: "rgba(0, 0, 0, 0.07)"
  liked-tint: "#5038c8"
typography:
  display:
    fontFamily: "Inter"
    fontSize: "32px"
    fontWeight: 700
  headline:
    fontFamily: "Inter"
    fontSize: "28px"
    fontWeight: 700
  desk-title:
    fontFamily: "Inter"
    fontSize: "26px"
    fontWeight: 700
  title:
    fontFamily: "Inter"
    fontSize: "20px"
    fontWeight: 700
  section:
    fontFamily: "Inter"
    fontSize: "18px"
    fontWeight: 700
  wordmark:
    fontFamily: "Inter"
    fontSize: "16px"
    fontWeight: 700
  prompt:
    fontFamily: "Inter"
    fontSize: "15px"
    fontWeight: 400
  track-title:
    fontFamily: "Inter"
    fontSize: "14.5px"
    fontWeight: 500
  body:
    fontFamily: "Inter"
    fontSize: "14px"
    fontWeight: 400
  secondary:
    fontFamily: "Inter"
    fontSize: "13px"
    fontWeight: 400
  label:
    fontFamily: "Inter"
    fontSize: "13px"
    fontWeight: 500
  label-strong:
    fontFamily: "Inter"
    fontSize: "13px"
    fontWeight: 600
  detail:
    fontFamily: "Inter"
    fontSize: "12.5px"
    fontWeight: 400
  badge:
    fontFamily: "Inter"
    fontSize: "12.5px"
    fontWeight: 500
  small:
    fontFamily: "Inter"
    fontSize: "11.5px"
    fontWeight: 400
rounded:
  lamp-bar: "2px"
  small: "4px"
  widget: "6px"
  row: "8px"
  field: "10px"
  popover: "12px"
  pane: "14px"
  login-card: "16px"
  console: "18px"
  pill: "9999px"
spacing:
  chip-gap: "6px"
  gap: "8px"
  inset: "12px"
  card-gap: "14px"
  window: "16px"
  desk: "20px"
  page: "24px"
components:
  pane:
    backgroundColor: "{colors.dark-pane-glass}"
    rounded: "{rounded.pane}"
  pane-light:
    backgroundColor: "{colors.light-pane-glass}"
    rounded: "{rounded.pane}"
  console:
    backgroundColor: "{colors.dark-console-glass}"
    rounded: "{rounded.console}"
    height: "88px"
  console-light:
    backgroundColor: "{colors.light-console-glass}"
    rounded: "{rounded.console}"
    height: "88px"
  popover:
    backgroundColor: "{colors.dark-overlay}"
    textColor: "{colors.dark-text}"
    rounded: "{rounded.popover}"
    padding: "6px"
  popover-light:
    backgroundColor: "{colors.light-overlay}"
    textColor: "{colors.light-text}"
    rounded: "{rounded.popover}"
    padding: "6px"
  discovery-desk:
    backgroundColor: "{colors.dark-well-glass}"
    rounded: "{rounded.pane}"
    padding: "20px"
  discovery-desk-light:
    backgroundColor: "{colors.light-well-glass}"
    rounded: "{rounded.pane}"
    padding: "20px"
  button-primary:
    backgroundColor: "{colors.dark-lamp}"
    textColor: "{colors.dark-on-lamp}"
    typography: "{typography.label-strong}"
    rounded: "{rounded.pill}"
    padding: "8px 18px"
  button-primary-hover:
    backgroundColor: "{colors.dark-lamp-hover}"
  button-primary-light:
    backgroundColor: "{colors.light-lamp}"
    textColor: "{colors.light-on-lamp}"
    typography: "{typography.label-strong}"
    rounded: "{rounded.pill}"
    padding: "8px 18px"
  button-primary-light-hover:
    backgroundColor: "{colors.light-lamp-hover}"
  play-key:
    backgroundColor: "{colors.dark-lamp}"
    textColor: "{colors.dark-on-lamp}"
    rounded: "{rounded.pill}"
    size: "36px"
  play-key-desk:
    backgroundColor: "{colors.dark-lamp}"
    textColor: "{colors.dark-on-lamp}"
    rounded: "{rounded.pill}"
    size: "44px"
  button-glass:
    backgroundColor: "{colors.dark-hover-fill}"
    textColor: "{colors.dark-text}"
    typography: "{typography.label-strong}"
    rounded: "{rounded.pill}"
    padding: "8px 18px"
  soft-button:
    backgroundColor: "{colors.dark-surface}"
    textColor: "{colors.dark-text}"
    typography: "{typography.label}"
    rounded: "{rounded.pill}"
    padding: "7px 12px"
  soft-button-hover:
    backgroundColor: "{colors.dark-surface-hover}"
  choice-chip:
    backgroundColor: "{colors.dark-surface}"
    textColor: "{colors.dark-text}"
    typography: "{typography.label}"
    rounded: "{rounded.pill}"
    padding: "7px 12px"
  choice-chip-selected:
    backgroundColor: "{colors.dark-text}"
    textColor: "{colors.dark-window}"
    typography: "{typography.label}"
    rounded: "{rounded.pill}"
    padding: "7px 12px"
  choice-chip-selected-light:
    backgroundColor: "{colors.light-text}"
    textColor: "{colors.light-window}"
    typography: "{typography.label}"
    rounded: "{rounded.pill}"
    padding: "7px 12px"
  field-well:
    backgroundColor: "{colors.dark-well-glass}"
    textColor: "{colors.dark-text}"
    typography: "{typography.secondary}"
    rounded: "{rounded.field}"
    padding: "8px 12px"
  field-well-light:
    backgroundColor: "{colors.light-well-glass}"
    textColor: "{colors.light-text}"
    typography: "{typography.secondary}"
    rounded: "{rounded.field}"
    padding: "8px 12px"
  taste-prompt:
    backgroundColor: "{colors.dark-well-glass}"
    textColor: "{colors.dark-text}"
    typography: "{typography.prompt}"
    rounded: "{rounded.field}"
    padding: "8px 12px"
    width: "560px"
  search-field:
    backgroundColor: "{colors.dark-well-glass}"
    textColor: "{colors.dark-text}"
    typography: "{typography.body}"
    rounded: "{rounded.pill}"
    height: "34px"
  switch-off:
    backgroundColor: "{colors.dark-surface-active}"
    rounded: "{rounded.pill}"
    width: "40px"
    height: "22px"
  switch-on:
    backgroundColor: "{colors.dark-lamp}"
    rounded: "{rounded.pill}"
    width: "40px"
    height: "22px"
  switch-on-light:
    backgroundColor: "{colors.light-lamp}"
    rounded: "{rounded.pill}"
    width: "40px"
    height: "22px"
  nav-row-selected:
    backgroundColor: "{colors.dark-selected-fill}"
    textColor: "{colors.dark-text}"
    rounded: "{rounded.row}"
  nav-row-selected-light:
    backgroundColor: "{colors.light-selected-fill}"
    textColor: "{colors.light-text}"
    rounded: "{rounded.row}"
  playing-title:
    textColor: "{colors.dark-lamp-text}"
    typography: "{typography.track-title}"
  playing-title-light:
    textColor: "{colors.light-lamp-text}"
    typography: "{typography.track-title}"
  connect-pill:
    backgroundColor: "{colors.dark-overlay}"
    textColor: "{colors.dark-lamp-text}"
    typography: "{typography.badge}"
    rounded: "{rounded.pill}"
  connect-pill-light:
    backgroundColor: "{colors.light-overlay}"
    textColor: "{colors.light-lamp-text}"
    typography: "{typography.badge}"
    rounded: "{rounded.pill}"
  feedback-love-selected:
    backgroundColor: "{colors.dark-lamp}"
    textColor: "{colors.dark-on-lamp}"
    rounded: "{rounded.pill}"
    size: "28px"
  feedback-less-selected:
    backgroundColor: "{colors.dark-text}"
    textColor: "{colors.dark-window}"
    rounded: "{rounded.pill}"
    size: "28px"
---

# Design System: Spotiurge

## Overview

**Creative North Star: "The Studio Control Room"**

Music lives in the lit room behind the glass; Spotiurge is the console in front of it. The whole native desktop app (sidebar, top chrome, Home and its For you discovery desk, library, search, collection pages, queue, lyrics, menus, Connect, settings, login and the player console) is built from smoked glass panes that float 8px apart over a dim room. The room is the window colour lit by two soft colour fields: the playing cover's light from the upper left and a counter-light, the same colour turned a third of the way round the hue wheel, from the lower right. With no cover tint the key light is the lamp amber, so the default room reads amber and teal. The dark theme is a blue-black room at night; the light theme is milky glass over a pale cool room that the amber key light warms toward peach.

The glass is rendered natively by the app as painted layers: a translucent fill, a top sheen that fades out within 90pt, a two-tone rim one physical pixel wide (lit along the top third, shaded along the bottom third), and a soft offset shadow. There is no OS backdrop blur, no macOS vibrancy and no per-frame blur pass; the room is already as smooth as blurred light, so translucency over it reads as frosted. Pane, console, popover and well are four glass kinds with their own opacity, sheen, rim and shadow (see Elevation & Depth).

Density is a desktop music console: 56px rows, a 56px top bar, an 88px console, and Inter ranked by weight rather than by colour. One VU-amber lamp marks whatever is live. Everything else, including every choice, is neutral. Motion is a single signature, the fader glide, finite and interruptible, and Reduce Motion (the app's own setting or the macOS system preference) makes every transition immediate. The mark is an amber surge S on a smoked glass tile (`assets/brand/spotiurge-mark.svg`), rasterised at runtime by `util::app_icon_rgba` so the in-app logo and the app icon are the same picture.

The category default this world refuses: a flat black Spotify clone with green accents and card grids.

**Evidence and limits.** Every token here is read from the shipped source of the frozen candidate `.qa/spotiurge-glass-final-candidate` (SHA-256 `fa1b2ab01f6e7ce36b429b17a69af0065dec21c2a6a53135cf1bbaecbdd52480`, macOS arm64, source commit `0b3e670`, all design files hash-identical to `.qa/glass-candidate-source.json`). The finish review (r1, then the r2 verdict) scored all seven r1 fixes resolved with disposition "ship", on 52 static macOS captures (`docs/reviews/spotiurge-glass/after-captures.json`). Later packaged Mac checks restored Spotify and cloud credentials, completed private sync and received twelve Luna suggestions; Spotify catalogue verification timed out. App Reduce motion control operation was exercised in native demo. See `docs/reviews/spotiurge-glass/verification.md` for those separate checks and short CPU observations. Not verified: motion smoothness and frame pacing, a controlled idle-power baseline, live macOS Reduce Motion changes, local playback/Connect in the exact glass package, and any Windows or Linux UI run. No capture represents real listening; the demo data is synthetic and labelled as demo in the UI.

**Key Characteristics:**
- Smoked glass panes floating 8px apart over a cover-lit room, painted by the app, never OS blur.
- One VU-amber lamp for live state; neutral inverted pills for choices.
- Opaque overlays: menus, popovers and dialogs never let content ghost through.
- Inter only, hierarchy by weight on a tight scale.
- Finite exponential ease-out motion in three durations (120 / 220 / 600 ms), immediate under Reduce Motion.

## Colors

A cool blue-black (dark) or pale cool grey (light) neutral room, glass derived from those neutrals, and a single warm amber signal.

### Primary
- **VU Amber Lamp** (dark `dark-lamp`, light `light-lamp`): the live-state signal. It fills the primary play keys, the leading-edge lamp bar on the selected navigation channel and on the playing row, the EQ glyph beside a playing item, the Connect pill's icon and tint, and the room's default key light. Hover deepens or brightens it to `*-lamp-hover`.
- **Lamp Text** (dark `dark-lamp-text`, light `light-lamp-text`): the lamp's colour for small text (playing titles, the Connect pill label, "Listening on this device", the active device). It is derived, not picked: `Palette::accent_text()` takes the ground `surface_active` mixed 20% toward black in light (10% toward white in dark), then steps the accent 5% toward black (light) or white (dark) in gamma space until WCAG contrast against that ground reaches 4.5:1. In light that takes ten steps from the amber to deep amber-brown; in dark the amber already passes, so lamp text equals the lamp. Fills keep the plain lamp.

### Neutral
- **Control Room Window** (`dark-window`, `light-window`): the room behind the glass; also the label colour on an inverted choice pill.
- **Smoked Pane** (`dark-panel`, `light-panel`): base of shell panes and the console before opacity.
- **Surface steps** (`*-surface`, `*-surface-hover`, `*-surface-active`): resting, hovered and pressed fills of soft buttons, unselected chips and egui widgets; `surface-active` is also the off switch track.
- **Outline and Rim** (`*-outline`, `dark-rim`): hairlines. Dark frames take `dark-rim` (the outline with 6% white over it); light frames use `light-outline` directly.
- **Text ranks** (`*-text`, `*-secondary`, `*-dim`): primary text, secondary metadata, and the dimmest tier (icons at rest, hints).
- **Overlay** (`dark-overlay`, `light-overlay`): the opaque base of every popover, menu, dialog, toast and the Connect pill.
- **Glass fills** (`*-pane-glass`, `*-console-glass`, `*-well-glass`): the palette roles at the glass kind's opacity. The console's base is additionally eased 12% toward the playing cover's tint.
- **Hover and Selected fills** (`*-hover-fill`, `*-selected-fill`): light through dark glass, shade on light glass. Keys (tiles and shelf entries) rest at white 5% to 10% (dark) or black 3.5% to 6.5% (light) as they lift.
- **Danger and Warning** (`*-danger`, `*-warning`): palette roles kept for destructive and warning text. Cautions and error notices in the discovery desk use `*-text`, not amber and not warning.
- **Liked Tint** (`liked-tint`): the fixed cover light for the Liked Songs page wash.

### Named Rules
**The One Lamp Rule.** Amber marks live state only: playing, primary play, the current navigation channel, Connect and the active device. Choices, cautions and errors never take it. The build also spends amber on a selected Love rating in the discovery feedback column, a slider fill while it is held, the 1px keyboard focus ring, the text caret and the text-selection tint, and the native Settings switch; those are the recorded extent, not licence for more.

**The Derived Lamp Text Rule.** Never set small amber text by hand. Use the lamp-text role, which deepens until it reads at 4.5:1; light lamp text is `#653706`.

**The Capped Wash Rule.** A page's cover wash pools at the top of the page pane (full strength at 0pt, 45% at 140pt, gone by 360pt). In light it is capped at 20% of the tint so live text stays legible; in dark it is 30% on Home, Search, Settings and Queue and 55% on collection pages.

## Typography

**Display Font:** Inter (bundled at Regular 400, Medium 500, SemiBold 600 and Bold 700), with the monochrome Noto Emoji face behind it and installed system faces for scripts Inter lacks.

**Character:** One family, ranked by weight on a tight scale, like a timetable rack. Bold carries titles, Medium carries row titles and controls, Regular carries everything secondary. Sizes are egui logical points, written here as px.

### Hierarchy
- **Display** (700, 32px): collection hero titles at wide widths; the title steps down toward 28px until it fits.
- **Headline** (700, 28px): page titles such as Library, Settings and Queue. Top songs uses 30px, and the login wordmark is 30px.
- **Desk Title** (700, 26px): the For you desk title and the search top result.
- **Title** (700, 20px): dialog titles and the Home greeting.
- **Section** (700, 18px): settings and queue section heads (the shared section title is 17px).
- **Wordmark** (700, 16px): "Spotiurge" beside the 24px mark in the sidebar.
- **Track Title** (500, 14.5px): row titles.
- **Body** (400, 14px): egui body and button text, the search field.
- **Prompt** (400, 15px): the taste prompt.
- **Secondary** (400, 13px): subtitles, discovery reasons, settings fields.
- **Label** (500, 13px) and **Label Strong** (600, 13px): chips and soft buttons; pill buttons.
- **Detail** (400, 12.5px) and **Badge** (500, 12.5px): row subtitles; the Connect pill.
- **Small** (400, 11.5px): small print.

### Named Rules
**The Weight Ranks Rule.** Rank with weight, then size; never with a second family, colour or uppercase eyebrow. Uppercase appears only in data column headers.

**The Fixed Cell Rule.** Changing state never shifts geometry: durations right-align, and the discovery reason sits in a fixed-width cell so the 28px feedback column holds one x position on every row.

## Layout

A floating three-pane shell. The sidebar pane (minimum 210px wide) on the left, the page pane in the centre, an optional right panel for Queue or Lyrics (minimum 280px), and the player console along the bottom, all separated from each other and from the window edge by an 8px gap. The top bar is 56px tall and sits inside the page pane; on macOS the window content runs under the titlebar and reserves 28px for the traffic lights.

Page content uses 24px side padding, 4px top and 48px bottom. Egui item spacing is 8px by 6px, button padding 12px by 6px, menus 6px inside, windows 16px inside. Rows are 56px (default), 48px (compact) or 36px (thin, no cover); sidebar library rows are 60px or 32px compact. Library cards are 172px with 14px gaps.

Responsive behaviour comes from width, not breakpoints in a stylesheet. The main window's minimum is 760 by 520px, raised while side panels are open. Track tables add the album column above 560px, the date added above 760px and further columns above 920px; the discovery desk treats 920px as wide and drops the reason column to hover text below it. The search field aims for half the free top-bar width between 200px and 440px (never below 80px). When the Connect pill would squeeze the field below 200px it folds to an amber icon chip, keeping at least 8px between field and pill. The taste prompt is capped at 560px wide; the For you overflow menu is 240px to 280px wide, anchored under its button. Captures were taken at 1440 by 900 (normal) and 900 by 760 (narrow).

## Elevation & Depth

Depth is glass over a lit room: translucent fills over soft colour fields, a sheen, a two-tone rim and an offset shadow. It is a hybrid of tonal layering and ambient shadow, and all of it is painted by the app (a pane is four cheap shapes: shadow, fill, sheen, rim).

### Glass kinds
- **Pane** (sidebar, page, queue, lyrics): fill at 72% (dark) or 58% (light); sheen white 4.5% (dark) or 55% (light); rim top white 11% or 95%, bottom black 42% or 10%.
- **Console** (player): fill at 80% or 72%, tinted toward the cover; sheen 6% or 60%; rim top 14% or 100%, bottom 45% or 12%.
- **Popover** (menus, Connect, dialogs, toasts, update window, login card, Connect pill base): fully opaque `*-overlay`; sheen 5% or 50%; rim top 13% or 100%, bottom 50% or 14%. Egui frames that cannot take a gradient rim use a single 1px `dark-rim` or `light-outline` stroke.
- **Well** (fields, the taste prompt, the discovery desk): `*-surface` at 55% (dark) or 75% (light); no sheen, rim or shadow of its own. Text wells add their own 1px rim (see Components).

### Shadow Vocabulary
- **Pane shadow** (offset 0 8px, blur 28px, `*-shadow` at 55% dark or 50% light): floating shell panes.
- **Console shadow** (offset 0 10px, blur 32px, `*-shadow` at 80% dark or 70% light): the player console.
- **Popover shadow** (offset 0 12px, blur 32px, full `*-shadow`): menus, dialogs, popovers.
- **Popup shadow** (offset 0 8px, blur 24px, full `*-shadow`): egui popups.
- **Header shadow** (14px gradient, black up to alpha 110 dark or 36 light, deepening over the first 24pt of scroll): the top bar's shadow on a page scrolled beneath it.
- **Switch knob** (offset 0 1px, blur 4px, black alpha 60): the white 16px knob.

### Named Rules
**The Rendered Glass Rule.** Glass is painted: fill, sheen, rim and shadow over cached colour fields. Never use OS backdrop blur, vibrancy or a per-frame blur pass.

**The Opaque Overlay Rule.** Anything that floats over content (menus, popovers, dialogs, toasts) is opaque glass. Sheen, rim and shadow keep it glass; content beneath never ghosts through its text.

**The One Pixel Rim Rule.** Rims are one physical pixel, lit along the top third and shaded along the bottom third, over the outline at 60% (dark) or 80% (light).

## Shapes

Soft, nested rounding that grows with the size of the piece of glass: lamp bar 2px, small 4px, egui widgets 6px, rows and lamps 8px, text wells 10px, popovers 12px, panes and the discovery desk 14px, the login card 16px, the console 18px. Every button, chip, toggle, search field and the Connect pill is a full pill (radius half the height). Play keys are circles that grow 5% on hover and sink to 94% while pressed. Covers in collection heroes take 10px corners (round for artists). The mark's tile is a 120 unit square with 27 unit corners.

## Components

### Buttons
- **Shape:** full pill (`rounded.pill`).
- **Primary:** lamp fill with on-lamp label, Label Strong, 8px by 18px padding. Primary play is a lamp disc: 36px in the console, 44px on the desk.
- **Hover / Focus:** fills ease toward `*-lamp-hover` over 120 ms; focus draws a 1px lamp ring 2px outside the control with a 4px radius.
- **Glass (secondary):** a clear key, `*-hover-fill` at 60% rising with hover, inside a 1px rim that lights from `dim` to `text`.
- **Soft:** `*-surface` easing to `*-surface-hover`, Label, 7px by 12px; active soft buttons invert to a text fill with a window-colour label.
- **Icon buttons:** frameless; icon colour lifts on hover and the icon sinks to 90% while pressed.

### Chips (choices and segmented controls)
- **Style:** unselected chips are `*-surface` pills, Label, 7px by 12px, 6px apart.
- **State:** the selected choice is a neutral inverted pill (`*-text` fill, `*-window` label) that glides from the old choice to the new one over 220 ms; labels it passes over invert as it covers them. Layout moves snap instead of gliding.

**The Neutral Choice Rule.** Chips, segmented controls and settings options select with the inverted neutral pill, never amber.

**The Native Switch Exception.** On/off settings rows use a native-style switch (40 by 22px pill, white 16px knob): the track eases from `*-surface-active` to the lamp over 220 ms. This is the only choice-like control that turns amber, scoped to boolean switches.

### Cards / Containers
- **Corner Style:** panes 14px, console 18px, popovers 12px, desk 14px.
- **Background:** the glass kinds in Elevation & Depth; settings groups and the For you desk are wells inset in the page pane, not second floating panes.
- **Shadow Strategy:** only floating glass casts shadow; wells do not.
- **Internal Padding:** desk 20px; settings groups 20px by 16px, capped at 760px wide; page 24px.

### Inputs / Fields
- **Style:** text wells (`field_well`) take the well fill, 10px radius, 8px by 12px padding and a 1px rim in `dark-rim` or `light-outline`. The search field is a 34px pill well with a 16px leading search icon.
- **Focus:** the rim eases over 120 ms to 1.5px and to `*-text` at 60%; the caret is a 2px lamp line.
- **Taste prompt:** a three-row well in Prompt type, capped at 560px wide.

### Navigation
- **Sidebar:** the 24px mark and Bold 16 wordmark at the top, then channels. The selected channel takes the lamp: `*-selected-fill` behind an 8px-radius row with a 3px amber bar at its leading edge, 46% of the row height.
- **Top bar:** back and forward keys, the search pill, then the Connect pill (opaque overlay base, amber tint at 14% rising to 22% on hover, a 1px amber stroke at 22% to 36%, a 13px amber icon and Badge lamp-text label), and icon buttons with the avatar.

### The Discovery Desk (signature)
Home's For you desk is a well inset in the page pane with a static glow of the cover tint rising from its upper left. It holds the Desk Title, a mode strip of neutral choice chips (Familiar, Balanced, Adventurous), a 44px lamp play disc, a fixed-width reason column in Secondary and a 28px feedback column: a selected Love fills amber, a selected Less fills with `*-text`. Demo picks are labelled as demo; unplayable matches collapse into "Couldn't play (N)" behind a stroke chevron. Recommendation copy describes results from the configured AI service, which in this build is gpt-6-luna through CLIProxyAPI only; the UI does not claim parity with Spotify's own recommendations.

### Fader Glide (signature motion)
Every helper moves toward its target over a fixed time with exponential ease-out, (1 - 2^(-10t)) / (1 - 2^(-10)), retargets from wherever the value is, and asks for a repaint only while moving.
- **Feedback** (120 ms): hover and press fills, field focus, egui's own animation time.
- **State** (220 ms): the amber lamp travelling between sidebar channels, the neutral pill gliding between chips, switch tracks and feedback fills.
- **Page** (220 ms, rising 6pt): content fades up from 35% opacity and settles 6pt into place after navigation, without delaying clicks.
- **Ambient** (600 ms): the room light and the page wash crossing over to a new cover.

**The Fader Glide Rule.** Selection travels; it never jumps or blinks. When the layout moves the selected item, the lamp follows at once instead of gliding.

**The Still Under Reduce Motion Rule.** With the app's Reduce motion setting on, or the macOS Reduce Motion preference (read at most every 2 s), every helper returns its target immediately, egui animation time is 0 and scroll animation is off.

## Do's and Don'ts

### Do:
- **Do** build every new surface from the four glass kinds (pane, console, popover, well) and their recorded opacities, sheen, rims and shadows.
- **Do** keep panes 8px apart and from the window edge, with 24px page padding.
- **Do** spend amber only on live state, and use the lamp-text role for any amber text.
- **Do** select choices with the neutral inverted pill gliding over 220 ms.
- **Do** make overlays opaque `*-overlay` glass.
- **Do** put text inputs in a 10px-radius well with the 1px rim and the eased focus rim.
- **Do** use only the three durations (120, 220, 600 ms) with exponential ease-out, and make them immediate under Reduce Motion.
- **Do** keep demo, unplayable and remote-playback states honest and labelled.

### Don't:
- **Don't** use OS backdrop blur, vibrancy or a per-frame blur pass for glass.
- **Don't** make menus, popovers or dialogs translucent.
- **Don't** select chips, segmented controls or settings options with amber; boolean switches are the one recorded exception.
- **Don't** colour cautions or error icons amber.
- **Don't** raise the light-theme cover wash above 20%.
- **Don't** introduce a second typeface, gradient text, or uppercase eyebrows.
- **Don't** return to a flat black Spotify clone with green accents and card grids.
- **Don't** claim smoothness, frame rate or CPU cost; none has been measured.

**Not canonized** (residuals in the shipped build, recorded so nobody copies them): the active sort header that paints plain `*-lamp` text (in light below 4.5:1) instead of lamp-text or a neutral; dialog text fields on the older recipe (8px radius, flat rim, no eased focus rim) instead of the 10px field well; placeholder hints in `*-dim`, which read faintly on light wells; the faint light-theme field rim (about 1.2:1 against the pane); a focus ring proven only by test, never captured; macOS shortcuts written "Cmd+" rather than the platform symbols. They stay out of the system because each is a known contrast or consistency gap, not a design decision.
