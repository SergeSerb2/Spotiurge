---
name: Spotiurge
description: A quiet listening workspace in T3 Pretty's World Scenery family. Flat forest and mist chrome over one static mountain lake, with moss reserved for whatever is live.
colors:
  dark-window: "#0e1110"
  dark-panel: "#141a17"
  dark-surface: "#1a221e"
  dark-surface-hover: "#202a25"
  dark-surface-active: "#2c3a32"
  dark-outline: "#2e3b34"
  dark-text: "#f3f6f3"
  dark-secondary: "#c5cfc8"
  dark-dim: "#b8c5bb"
  dark-disabled: "#84988b"
  dark-moss: "#98d2ac"
  dark-moss-hover: "#b7e6c8"
  dark-moss-text: "#98d2ac"
  dark-on-moss: "#07140c"
  dark-danger: "#ffb0b8"
  dark-warning: "#ffb020"
  dark-overlay: "#202a25"
  dark-shadow: "rgba(0, 0, 0, 0.45)"
  dark-pane-glass: "rgba(20, 26, 23, 0.68)"
  dark-content-glass: "rgba(44, 58, 50, 0.85)"
  dark-console-glass: "rgba(20, 26, 23, 0.80)"
  dark-well-glass: "rgba(26, 34, 30, 0.55)"
  dark-rim: "rgba(243, 246, 243, 0.07)"
  dark-hover-fill: "rgba(255, 255, 255, 0.06)"
  dark-key-fill: "rgba(255, 255, 255, 0.095)"
  dark-selected-fill: "rgba(255, 255, 255, 0.095)"
  dark-chip-fill: "rgba(255, 255, 255, 0.08)"
  dark-chip-fill-hover: "rgba(255, 255, 255, 0.12)"
  dark-scenery-wash: "rgba(0, 0, 0, 0.612)"
  dark-liked-cover: "#3c5d4b"
  light-window: "#f4f6f4"
  light-panel: "#ffffff"
  light-surface: "#eaf0eb"
  light-surface-hover: "#e3e9e4"
  light-surface-active: "#c9d1ca"
  light-outline: "#d8ded9"
  light-text: "#161a17"
  light-secondary: "#4b524c"
  light-dim: "#47504a"
  light-disabled: "#59695e"
  light-moss: "#27633f"
  light-moss-hover: "#225738"
  light-moss-text: "#1a422a"
  light-on-moss: "#ffffff"
  light-danger: "#951524"
  light-warning: "#7b3605"
  light-overlay: "#ffffff"
  light-shadow: "rgba(16, 24, 18, 0.14)"
  light-pane-glass: "rgba(255, 255, 255, 0.68)"
  light-content-glass: "rgba(255, 255, 255, 0.68)"
  light-console-glass: "rgba(255, 255, 255, 0.80)"
  light-well-glass: "rgba(234, 240, 235, 0.75)"
  light-rim: "rgba(22, 26, 23, 0.08)"
  light-hover-fill: "rgba(0, 0, 0, 0.045)"
  light-key-fill: "rgba(0, 0, 0, 0.035)"
  light-selected-fill: "rgba(0, 0, 0, 0.07)"
  light-scenery-wash: "rgba(255, 255, 255, 0.68)"
  light-liked-cover: "#e3efe6"
  brand-tile: "#141a17"
  brand-mint: "#8fceab"
typography:
  hero:
    fontFamily: "Inter"
    fontSize: "32px"
    fontWeight: 700
  display:
    fontFamily: "Inter"
    fontSize: "30px"
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
  shelf-title:
    fontFamily: "Inter"
    fontSize: "17px"
    fontWeight: 700
  wordmark:
    fontFamily: "Inter"
    fontSize: "16px"
    fontWeight: 700
  nav:
    fontFamily: "Inter"
    fontSize: "14.5px"
    fontWeight: 600
  nav-active:
    fontFamily: "Inter"
    fontSize: "14.5px"
    fontWeight: 700
  item-title:
    fontFamily: "Inter"
    fontSize: "14px"
    fontWeight: 600
  body:
    fontFamily: "Inter"
    fontSize: "14px"
    fontWeight: 400
  menu:
    fontFamily: "Inter"
    fontSize: "13.5px"
    fontWeight: 400
  button:
    fontFamily: "Inter"
    fontSize: "13px"
    fontWeight: 600
  label:
    fontFamily: "Inter"
    fontSize: "13px"
    fontWeight: 500
  secondary:
    fontFamily: "Inter"
    fontSize: "13px"
    fontWeight: 400
  item-subtitle:
    fontFamily: "Inter"
    fontSize: "12.5px"
    fontWeight: 400
  meta:
    fontFamily: "Inter"
    fontSize: "12px"
    fontWeight: 400
  small:
    fontFamily: "Inter"
    fontSize: "11.5px"
    fontWeight: 400
rounded:
  focus: "4px"
  widget: "6px"
  cover: "6px"
  row: "8px"
  field: "10px"
  hero-cover: "10px"
  popover: "12px"
  card-hover: "12px"
  pane: "14px"
  console: "18px"
  pill: "9999px"
spacing:
  chip-gap: "6px"
  item-x: "8px"
  item-y: "6px"
  gap: "8px"
  card-gap: "14px"
  window-margin: "16px"
  plate-padding: "20px"
  page-padding: "24px"
components:
  pane-dark:
    backgroundColor: "{colors.dark-pane-glass}"
    rounded: "{rounded.pane}"
  pane-light:
    backgroundColor: "{colors.light-pane-glass}"
    rounded: "{rounded.pane}"
  content-plate-dark:
    backgroundColor: "{colors.dark-content-glass}"
    rounded: "{rounded.pane}"
    padding: "20px"
  content-plate-light:
    backgroundColor: "{colors.light-content-glass}"
    rounded: "{rounded.pane}"
    padding: "20px"
  console-dark:
    backgroundColor: "{colors.dark-console-glass}"
    rounded: "{rounded.console}"
    height: "88px"
  console-light:
    backgroundColor: "{colors.light-console-glass}"
    rounded: "{rounded.console}"
    height: "88px"
  popover-dark:
    backgroundColor: "{colors.dark-overlay}"
    textColor: "{colors.dark-text}"
    rounded: "{rounded.popover}"
    padding: "6px"
  popover-light:
    backgroundColor: "{colors.light-overlay}"
    textColor: "{colors.light-text}"
    rounded: "{rounded.popover}"
    padding: "6px"
  menu-item-dark:
    textColor: "{colors.dark-text}"
    typography: "{typography.menu}"
    rounded: "{rounded.row}"
    height: "28px"
  menu-item-disabled-dark:
    textColor: "{colors.dark-disabled}"
    typography: "{typography.menu}"
    height: "28px"
  button-primary-dark:
    backgroundColor: "{colors.dark-moss}"
    textColor: "{colors.dark-on-moss}"
    typography: "{typography.button}"
    rounded: "{rounded.pill}"
    padding: "8px 18px"
  button-primary-hover-dark:
    backgroundColor: "{colors.dark-moss-hover}"
    textColor: "{colors.dark-on-moss}"
  button-primary-disabled-dark:
    backgroundColor: "{colors.dark-chip-fill}"
    textColor: "{colors.dark-disabled}"
  button-primary-light:
    backgroundColor: "{colors.light-moss}"
    textColor: "{colors.light-on-moss}"
    typography: "{typography.button}"
    rounded: "{rounded.pill}"
    padding: "8px 18px"
  button-primary-hover-light:
    backgroundColor: "{colors.light-moss-hover}"
    textColor: "{colors.light-on-moss}"
  button-primary-disabled-light:
    backgroundColor: "{colors.light-surface-active}"
    textColor: "{colors.light-disabled}"
  button-secondary-dark:
    textColor: "{colors.dark-text}"
    typography: "{typography.button}"
    rounded: "{rounded.pill}"
    padding: "8px 18px"
  soft-button-dark:
    backgroundColor: "{colors.dark-surface}"
    textColor: "{colors.dark-text}"
    typography: "{typography.label}"
    rounded: "{rounded.pill}"
    padding: "7px 12px"
  soft-button-active-dark:
    backgroundColor: "{colors.dark-text}"
    textColor: "{colors.dark-window}"
  chip-dark:
    backgroundColor: "{colors.dark-chip-fill}"
    textColor: "{colors.dark-text}"
    typography: "{typography.label}"
    rounded: "{rounded.pill}"
    padding: "7px 12px"
  chip-hover-dark:
    backgroundColor: "{colors.dark-chip-fill-hover}"
  chip-selected-dark:
    backgroundColor: "{colors.dark-text}"
    textColor: "{colors.dark-window}"
  chip-light:
    backgroundColor: "{colors.light-surface}"
    textColor: "{colors.light-text}"
    typography: "{typography.label}"
    rounded: "{rounded.pill}"
    padding: "7px 12px"
  chip-hover-light:
    backgroundColor: "{colors.light-surface-hover}"
  chip-selected-light:
    backgroundColor: "{colors.light-text}"
    textColor: "{colors.light-window}"
  text-well-dark:
    backgroundColor: "{colors.dark-well-glass}"
    textColor: "{colors.dark-text}"
    typography: "{typography.body}"
    rounded: "{rounded.field}"
    padding: "8px 12px"
  search-field-dark:
    backgroundColor: "{colors.dark-well-glass}"
    textColor: "{colors.dark-text}"
    typography: "{typography.body}"
    rounded: "{rounded.pill}"
    height: "34px"
  nav-row-dark:
    textColor: "{colors.dark-secondary}"
    typography: "{typography.nav}"
    rounded: "{rounded.row}"
    height: "38px"
  nav-row-active-dark:
    backgroundColor: "{colors.dark-selected-fill}"
    textColor: "{colors.dark-text}"
    typography: "{typography.nav-active}"
    rounded: "{rounded.row}"
    height: "38px"
  track-row:
    typography: "{typography.item-title}"
    rounded: "{rounded.row}"
    height: "56px"
  track-row-compact:
    height: "48px"
  track-row-thin:
    height: "36px"
  play-key-console-dark:
    backgroundColor: "{colors.dark-moss}"
    textColor: "{colors.dark-on-moss}"
    rounded: "{rounded.pill}"
    size: "36px"
  play-key-desk-dark:
    backgroundColor: "{colors.dark-moss}"
    textColor: "{colors.dark-on-moss}"
    rounded: "{rounded.pill}"
    size: "44px"
  top-bar-key-dark:
    backgroundColor: "{colors.dark-key-fill}"
    textColor: "{colors.dark-secondary}"
    rounded: "{rounded.pill}"
    size: "32px"
  switch-off-dark:
    backgroundColor: "{colors.dark-surface-active}"
    rounded: "{rounded.pill}"
    width: "40px"
    height: "22px"
  switch-on-dark:
    backgroundColor: "{colors.dark-moss}"
    rounded: "{rounded.pill}"
    width: "40px"
    height: "22px"
  media-card:
    typography: "{typography.item-title}"
    rounded: "{rounded.card-hover}"
    width: "172px"
---

# Design System: Spotiurge

## Overview

**Creative North Star: "The Quiet Listening Workspace"**

Spotiurge is a quiet listening workspace in T3 Pretty's World Scenery family. Music stays in the foreground, and the scenery gives the app a recognizable place. One static, locally bundled, original mountain and lake image sits beneath a flat black or white contrast wash. Over it, flat translucent forest (dark) or mist (light) chrome holds navigation and the playback console. The main page stays clear. Menus and popovers are opaque.

The native composition is unchanged from the inherited app: sidebar, top bar, page and floating console, with Inter and the existing native control vocabulary. Glass here is painted compositing in egui (a translucent fill over the image and wash). It is not OS blur or backdrop-filter, and nothing is blurred per frame. Moss (mint in dark, deep forest in light) is the only accent, and it marks what is live: the play keys, the playing title, active console toggles and the Connect state.

The scenery is static and never causes idle animation. Motion is finite, interruptible and short, and Reduce Motion removes it. The previous amber studio world, its cover glows, sheen, double rims and leading lamp bars are retired.

**Key Characteristics:**
- One static photographic scene under a flat wash, covering 85% of the base together.
- Flat translucent chrome for panes and the console; clear page content; opaque popovers.
- Forest and mist neutrals with a single moss accent for live state.
- Inter at four weights; dense native rows (56, 48, 36 points).
- Finite motion: 120 ms feedback, 220 ms state and page; Reduce Motion honored.
- A bare mint ridge mark cut into three level-meter columns, on a forest tile for the app icon.

## Colors

Cool forest-neutral greys and greens over a slate-toned photograph, with one moss voice. Every color comes from `Palette::dark()` / `Palette::light()` in `src/theme.rs` or from `src/ui/material.rs`; custom palette files may override the sixteen base roles.

### Primary
- **Moss** (`dark-moss`, `light-moss`): fills of the play keys, the primary pill, switches when on, the "More like this" rating and the text cursor. Hover moves to `*-moss-hover`. Ink on a moss fill is `*-on-moss`.
- **Moss Text** (`*-moss-text`): small live text such as the playing title, the active device and the Connect pill. It is derived at runtime by `Palette::accent_text()`, which keeps the accent where it reads at 4.5:1 against a shaded `surface_active` and otherwise steps it darker (light) or brighter (dark). Dark keeps the accent as it is; the light value is the derived result.

### Neutral
- **Night Forest / Mist Window** (`*-window`): the base under the scenery, and the ink on selected chips and active soft buttons.
- **Forest Panel / White Panel** (`*-panel`): the base of pane and console glass and of egui panels.
- **Surface ladder** (`*-surface`, `*-surface-hover`, `*-surface-active`): egui widget fills at rest, hover and press. The dark content plate is built on `dark-surface-active`. The switch track rests on `surface-active`.
- **Outline** (`*-outline`): the rim of text wells and egui non-interactive frames.
- **Rim** (`*-rim`): the one quiet hairline used by the console, popovers, the sidebar divider, the top-bar keys and field wells at rest.
- **Text, Secondary, Dim** (`*-text`, `*-secondary`, `*-dim`): primary ink, supporting lines (subtitles, notices, inactive nav) and hints. Dim still carries information.
- **Disabled** (`*-disabled`): `Palette::disabled_text()`, the ink of disabled menu items and disabled pills. It is separate from dim so a disabled control reads as off.
- **Danger and Warning** (`*-danger`, `*-warning`): blocking failures use danger, degraded results (model failure) use warning, on both icon and text.
- **Overlay** (`*-overlay`): the opaque popover and menu fill.
- **Liked cover** (`*-liked-cover`): the fill behind the moss heart on the Liked Songs cover.

### Brand
- **Brand Tile and Brand Mint** (`brand-tile`, `brand-mint`): the packaged app icon only, from `assets/brand/spotiurge-mark.svg` and `util::app_icon_rgba`. Inside the app the bare ridge glyph is tinted with the palette's moss, so it is mint in dark and deep forest in light.

### Named Rules
**The One Voice Rule.** Moss marks live or primary state only. Selection, hover and grouping use neutral overlays, never the accent.

**The Measured Ink Rule.** Text, secondary, dim, moss text, danger and warning hold at least 4.5:1 over every pixel of the bundled scene after the wash, on bare ground, on pane glass and on content plates, including hover and selected fills. Option labels on chip fills are held to the same floor. A test in `material.rs` checks this; do not change a text role or wash without rerunning it.

## Typography

**Display Font:** Inter (Bold 700)
**Body Font:** Inter (Regular 400, Medium 500, SemiBold 600)
**Label/Mono Font:** egui monospace at 13 px for code only; Noto Emoji follows Inter in every family.

**Character:** One family at four real weights. Hierarchy comes from size and weight steps, not from a second face. Line heights follow Inter's font metrics in egui; no explicit line-height is set.

### Hierarchy
- **Hero** (700, 32 px wide, 28 px narrow, shrinking in 2 px steps to 20 px to fit): collection, album and artist titles beside the cover.
- **Display** (700, 30 px): the login wordmark and "Your top songs".
- **Headline** (700, 28 px): Library, Queue and Settings page titles.
- **Desk title** (700, 26 px): "For you" and the Search heading.
- **Title** (700, 20 px): the Home greeting, dialog and update titles.
- **Section** (700, 18 px): Settings sections, Queue groups and the lyrics header.
- **Shelf title** (700, 17 px): Home shelves and `section_title`.
- **Wordmark** (700, 16 px): "Spotiurge" beside the sidebar mark.
- **Nav** (600, 14.5 px; 700 when active): sidebar navigation.
- **Item title** (600, 14 px) and **Item subtitle** (400, 12.5 px): card and row titles and their artists.
- **Body** (400, 14 px): egui body and button text, search and field input.
- **Menu** (400, 13.5 px): popup menu items.
- **Button** (600, 13 px): pill buttons. **Label** (500, 13 px): soft buttons and choice chips.
- **Secondary** (400, 13 px): subtle lines, discovery reasons and notices.
- **Meta** (400, 12 px) and **Small** (400, 11.5 px): durations, counts and egui small text.

### Named Rules
**The One Family Rule.** Inter at its four weights carries the whole hierarchy. Do not add a display face.

## Layout

A fixed native frame: a resizable sidebar on the left (210 to 600 points, 250 by default), a 56-point top bar, the scrolling page, an optional right panel for Queue or Lyrics (at least 280 points) and an 88-point console floating along the bottom. Panes sit 8 points apart and 8 points from the window edge. On macOS the content reaches the top edge and leaves a 28-point inset for the traffic lights, except in fullscreen.

Pages use 24-point side padding, 4 points on top and 48 at the bottom. Content plates use 20-point padding (Settings sections use 20 by 16 and cap at 760 points wide). Cards are 172 points wide with 14-point gaps; library grids use at least 108 points per column. egui item spacing is 8 by 6, button padding 12 by 6, menu margin 6 and window margin 16.

Responsive changes come from available width, not breakpoints in a stylesheet. Collection heroes use a 212-point cover and 32 px title above 720 points of width, otherwise 160 points and 28 px. Discovery rows show their reason column above 920 points (30% of the width, clamped to 200 to 360). Console side regions take 30% of the width, clamped to 200 to 420. Captures cover 1440 by 900 and 900 by 760 logical points.

## Elevation & Depth

Depth is tonal and flat. The image and wash form the ground, translucent glass sits on it and opaque popovers sit above. Panes and content plates have no shadow and no rim. The console and popovers carry one quiet hairline rim. Only popovers cast a structural shadow. Cover art casts a soft shadow in the dark scheme so it lifts off the glass.

### Shadow Vocabulary
- **Popover** (`0 8px 24px`, `*-shadow`): menus, popovers, toasts and egui windows.
- **Card art, dark only** (`0 10px 28px rgba(0, 0, 0, 0.47)`): card and search result covers.
- **Hero cover** (dark `0 14px 36px` at 90% of `dark-shadow`; light `0 8px 24px` at 80% of `light-shadow`): the collection hero cover.
- **Switch knob** (`0 1px 4px rgba(0, 0, 0, 0.24)`): the white knob of a switch.

### Named Rules
**The Single Cue Rule.** A surface steps off its host with fill or with a hairline, never both, and never with a glow, sheen or second rim.

**The Still Scenery Rule.** The scene is decoded once off the UI thread, cached as one texture and drawn under a flat wash. It never animates, never follows playback and is never blurred per frame.

## Shapes

Soft, consistent corners that grow with the size of the surface: 4 points for the focus ring, 6 for egui widgets and card covers, 8 for rows, nav items and menu items, 10 for text wells and hero covers, 12 for popovers and card hover plates, 14 for panes and content plates, 18 for the console. Buttons, chips, the search field, switches, badges and play keys are full pills or discs. Artist covers are circles.

The mark is one asymmetric mountain cut into three level-meter columns (`assets/brand/spotiurge-glyph.svg`). The app icon places it in mint on a forest tile with a corner of 27 on a 120-unit plate. At 32 pixels and below the column edges snap to whole pixels so the two gaps stay open.

## Components

### Buttons
- **Shape:** full pill (height / 2).
- **Primary pill:** moss fill, on-moss SemiBold 13 px label, 8 by 18 padding. Hover eases the fill to moss hover over 120 ms.
- **Primary disabled:** dark uses a white overlay at 8% under disabled ink, so the pill keeps its shape on any host. Light uses `light-surface-active` under disabled ink.
- **Secondary pill:** a clear key. The hover fill at 60% (to 150% under the pointer) inside a 1-point rim that moves from dim to text.
- **Soft button:** surface fill easing to surface hover, Medium 13 px, 7 by 12 padding, optional 15-point icon. Active inverts to a text fill with window ink.
- **Play keys:** moss discs, 36 points in the console and 44 on the For you desk. Hover grows the disc 5%; press sinks it to 94% at once and releases over 120 ms. When nothing can play, the desk key goes neutral (selected fill, secondary icon) at full opacity.
- **Icon buttons:** frameless, icon tint lifts on hover; press sinks the icon to 90%.
- **Focus:** a 1-point moss ring 2 points outside the control, 4-point corner.

### Chips
- **Style:** pills in Medium 13 px with 7 by 12 padding and 6-point gaps. Dark rests at a white overlay of 8%, rising to 12% under the pointer. Light rests on surface and eases to surface hover.
- **State:** the selected fill is solid text with window ink. It glides between choices over 220 ms, and the labels it covers turn to window ink as it passes. If the layout moves the selected chip, the fill follows at once.

### Cards / Containers
- **Pane:** sidebar glass, panel at 68%, 14-point corner, no rim, no shadow.
- **Content plate:** the For you desk, Settings sections and the Search top result. Dark uses `surface-active` at 85%, light uses panel at 68%. 14-point corner, no rim.
- **Console:** panel at 80% with a rim and an 18-point corner. With "album-art colour" on, the console blends 4% of the cover's softened tint into its fill, fading over 0.45 s. The visualizer stays clear of the rounded corners.
- **Media card:** 172 points wide, cover with a 6-point corner (circle for artists), title in Item title, subtitle in Item subtitle. Hover or focus eases in a 12-point plate at 1.4 times the hover fill and raises the cover 3 points over 220 ms.

### Inputs / Fields
- **Text well:** well glass inside a 1-point rim, 10-point corner, 8 by 12 padding.
- **Search field:** a 34-point pill with a 16-point secondary search icon and dim hint text.
- **Focus:** the rim widens to 1.5 points and moves to text at 60% over 120 ms.
- **Switch:** a 40 by 22 pill whose track eases from surface active to moss over 220 ms, with a 16-point white knob.

### Navigation
- **Sidebar:** the ridge mark at 24 points and the wordmark, then 38-point nav rows with a 20-point icon. Inactive rows use secondary ink; hover brings text ink and a quiet hover fill. The selected row is a neutral selected fill with text ink and bold weight. That fill glides between Home and Search over 220 ms. A rim hairline separates navigation from the library. There is no leading color bar.
- **Top bar:** 32-point circular keys on key fill with a rim; the icon lifts from secondary to text on hover; disabled keys use dim. The Connect badge is a pill on opaque overlay with a moss tint (14% rising to 22%), a moss rim and moss text.
- **Menus:** opaque overlay with a rim, 12-point corner, popover shadow and 6-point margin. Items are 28 points tall in Menu type with a 16-point secondary icon; hover fills at 1.6 times the hover fill. Disabled items use disabled ink on icon and label and take no clicks.

### Track rows
56 points (48 compact, 36 thin) with an 8-point corner. Hover eases in a hover fill; the selected row shows the selected fill only. The playing row uses moss text and the playing meter. Rows that cannot play stay visible with a dim title and do not start playback.

### Discovery desk
The For you content plate: Desk title, the update age (or "Demo picks"), shuffle and the 44-point play key, the exploration chips, one summary, then playable 56-point rows. Above 920 points each row shows a Secondary reason column. Each row ends in two 28-point rating discs; "More like this" fills moss and "Less like this" fills text, easing in over 220 ms. Notices are a 14-point icon and one Secondary line: danger for a blocking load failure, warning for a model failure, secondary for demo and info lines. Unmatched suggestions collapse into one "Couldn't play (n)" group with search actions. History and taste editing open inline on request.

## Do's and Don'ts

### Do:
- **Do** keep the stack: window base, the one static scene, the flat wash, translucent chrome, clear page content, opaque popovers.
- **Do** derive small live text with `Palette::accent_text()` and keep every text role at 4.5:1 over the real scene pixels.
- **Do** make overlay fills relative to their host (white overlays in dark, black in light) so chips and disabled pills keep a visible step on panes and content plates.
- **Do** use `FEEDBACK` (120 ms) for hover and press, `STATE` (220 ms) for selection and fills, `PAGE` (220 ms, 6-point rise) for page entrance, and return targets at once under Reduce Motion.
- **Do** mark state with neutral fill plus text weight or moss ink, not with extra decoration.
- **Do** use danger for blocking failures and warning for degraded results, on icon and text together.
- **Do** keep demo labels, unplayable suggestions and the remote-playlist gate visible and honest.

### Don't:
- **Don't** animate the background, add ambient cover glows, sheens, double rims or gradients to chrome.
- **Don't** describe or build the glass as OS blur or a per-frame backdrop filter.
- **Don't** reintroduce the amber studio palette or the leading lamp bar on selected rows.
- **Don't** use moss for selection, hover or grouping.
- **Don't** paint a dark chip or disabled pill with opaque `surface-active`; it disappears on the dark content plate.
- **Don't** use a second cue (rim plus fill) to separate a plate from its host.
- **Don't** fetch scenery or imagery from a network service; the scene is bundled.
- **Don't** claim playback features Spotify does not support.
