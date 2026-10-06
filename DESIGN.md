---
name: Spotiurge native desktop
description: A personal radio desk within the existing native music player.
colors:
  dark-window: "#0f1114"
  dark-panel: "#15181c"
  dark-surface: "#1d2127"
  dark-surface-hover: "#262b33"
  dark-surface-active: "#2f353f"
  dark-outline: "#2a3038"
  dark-text: "#f2f4f6"
  dark-secondary: "#a9b1bc"
  dark-dim: "#6e7784"
  dark-accent: "#1ed760"
  dark-accent-hover: "#3ce87a"
  dark-on-accent: "#0a140e"
  dark-warning: "#f2b85c"
  dark-overlay: "#22272e"
  light-window: "#f8f9fb"
  light-panel: "#ffffff"
  light-surface: "#eef0f3"
  light-surface-hover: "#e3e6eb"
  light-surface-active: "#d7dbe1"
  light-outline: "#dde1e6"
  light-text: "#14171a"
  light-secondary: "#535b66"
  light-dim: "#8b939e"
  light-accent: "#15a64a"
  light-accent-hover: "#128f40"
  light-on-accent: "#ffffff"
  light-warning: "#b87a14"
  light-overlay: "#ffffff"
typography:
  headline:
    fontFamily: Inter
    fontSize: "26px"
    fontWeight: 700
  title:
    fontFamily: Inter
    fontSize: "15px"
    fontWeight: 600
  track-title:
    fontFamily: Inter
    fontSize: "14.5px"
    fontWeight: 500
  body:
    fontFamily: Inter
    fontSize: "15px"
    fontWeight: 400
  secondary:
    fontFamily: Inter
    fontSize: "13px"
    fontWeight: 400
  detail:
    fontFamily: Inter
    fontSize: "12.5px"
    fontWeight: 400
  label:
    fontFamily: Inter
    fontSize: "13px"
    fontWeight: 600
  choice:
    fontFamily: Inter
    fontSize: "13px"
    fontWeight: 500
  menu:
    fontFamily: Inter
    fontSize: "13.5px"
    fontWeight: 400
rounded:
  cover: "4px"
  row: "6px"
  menu: "8px"
  window: "10px"
  pane: "14px"
spacing:
  tight: "4px"
  small: "6px"
  base: "8px"
  column: "12px"
  section: "14px"
  outer: "24px"
components:
  save-taste-dark:
    backgroundColor: "{colors.dark-accent}"
    textColor: "{colors.dark-on-accent}"
    typography: "{typography.label}"
    padding: "8px 18px"
  save-taste-dark-hover:
    backgroundColor: "{colors.dark-accent-hover}"
  save-taste-light:
    backgroundColor: "{colors.light-accent}"
    textColor: "#000000"
    typography: "{typography.label}"
    padding: "8px 18px"
  save-taste-light-hover:
    backgroundColor: "{colors.light-accent-hover}"
  button-outline-dark:
    textColor: "{colors.dark-text}"
    typography: "{typography.label}"
    padding: "8px 18px"
  choice-dark:
    backgroundColor: "{colors.dark-surface}"
    textColor: "{colors.dark-text}"
    typography: "{typography.choice}"
    padding: "7px 12px"
  choice-dark-active:
    backgroundColor: "{colors.dark-text}"
    textColor: "{colors.dark-window}"
  choice-light-active:
    backgroundColor: "{colors.light-text}"
    textColor: "{colors.light-window}"
  taste-input-dark:
    backgroundColor: "{colors.dark-surface}"
    textColor: "{colors.dark-text}"
    typography: "{typography.body}"
    rounded: "{rounded.row}"
  discovery-pane:
    rounded: "{rounded.pane}"
    padding: "18px 20px"
  discovery-row:
    height: "56px"
    typography: "{typography.track-title}"
  discovery-menu-dark:
    backgroundColor: "{colors.dark-overlay}"
    textColor: "{colors.dark-text}"
    rounded: "{rounded.menu}"
    padding: "6px"
  play-all:
    width: "44px"
    height: "44px"
  feedback:
    width: "28px"
    height: "28px"
---

# Design System: Spotiurge native desktop

## Overview

**Creative North Star: "Personal radio desk"**

A calm native listening workspace uses neutral text, compact track geometry and
cover-derived light within a restrained frosted pane. Inter and the existing
theme palettes connect this Home expression to the inherited music player.
Music and controls carry the identity; there is no ornamental display face or
invented imagery.

This is a scan of the implemented Home replacement, not an approval of a wider
redesign. The sidebar, library shelves, search, player and Connect keep their
incumbent layout. Tokens come from [src/theme.rs](src/theme.rs),
[src/ui/discovery.rs](src/ui/discovery.rs) and shared native
[widgets](src/ui/widgets.rs). Portable sizes represent egui logical points as
CSS pixels at unit zoom. Native code remains the runtime source of truth; the
sidecar's HTML/CSS is an illustrative translation, not a browser-based app.

**Key Characteristics:**

- Compact native controls and readable neutral type.
- Static translucent material with bounded, cover-derived light.
- Green for playback and intentional positive state; neutral selection for choices.
- Existing player chrome remains familiar across light and dark themes.

## Colors

Cool neutral surfaces support a green action vocabulary in paired dark and light
palettes. The frontmatter records default palettes, not every custom theme or
album tint the application can load.

### Primary

- **Listening green** (`dark-accent`, `light-accent`): Play all and selected positive feedback, with their matching hover colors.
- **On-accent ink** (`dark-on-accent`, `light-on-accent`): Inherited icon foregrounds. The light Save taste text uses the local black override recorded in its component token; the shared light palette is unchanged.

### Neutral

- **Window and panel**: Existing content ground and persistent shell.
- **Surface, hover and active**: Fields, quiet choices and native interaction states.
- **Outline**: One-point pane and popup boundaries.
- **Text and secondary**: Titles and supporting information respectively.
- **Dim**: Inherited disabled controls and subdued secondary boundaries.
- **Overlay**: Existing popup material.

### Named Rules

**The State Color Rule.** Use the existing palette roles for interaction; cover-derived tint belongs behind the content and does not replace text or state colors.

Warning colors mark request problems. They are semantic status colors, not a
second decorative accent. Theme-specific alpha and cover tint formulas live in
the sidecar because frontmatter color primitives cannot describe native meshes.

## Typography

**Display Font:** None added.
**Body Font:** Bundled Inter with real regular, medium, semibold and bold weights;
the existing fastframe script fallbacks and Noto Emoji remain installed.

**Character:** A compact sans hierarchy communicates the music and the action.
Font metrics determine line height; no separate line-height or tracking token is
introduced by this surface.

### Hierarchy

- **Headline**: Bold Home workspace title.
- **Title**: Semibold taste, saved-mix and history headings.
- **Track title**: Medium song names within inherited track rows.
- **Body**: Regular taste entry.
- **Secondary**: Regular update age, catalogue summaries and notices.
- **Detail**: Regular artist lines, bounded reasons, request status and history detail.
- **Label / choice / menu**: Real weights distinguish pill actions, exploration choices and popup commands.

### Named Rules

**The Native Type Rule.** Use the bundled weight helpers and lay out logical text before bidi positioning; the design does not add pre-reordered strings or a decorative display family.

## Layout

The replacement occupies Home's existing content width. Its pane inset and row
height are recorded in the component tokens. Shared track rows retain covers,
title/artist grouping, duration and library controls. Feedback reserves two
28-point controls with a 4-point gap; columns are separated by 12 points.

At content widths above 920 points, reasons occupy 30% of the pane's inner width,
clamped between 200 and 360 points. Each reason is a single ellipsized line with
the complete text in a tooltip. At 920 points and below, the reason column is
removed and the row tooltip carries the reason. This threshold is a content
width, not a window breakpoint. The catalogue summary is bounded to two lines.

The current Home composition, disclosure order and default hidden sections belong
to the [surface contract](.impeccable/surfaces/src-ui-discovery-rs.md); they are
not requirements for every future Spotiurge screen.

## Elevation & Depth

The Home pane uses tonal layering and a static tinted mesh, with a one-point
outline. Its fill is `palette.surface.gamma_multiply(0.72)` in dark mode and
`gamma_multiply(0.80)` in light mode. The mesh center lies at 22% across and 18%
down the pane, with tint multiplied by 0.16 or 0.12 respectively and transparent
edge vertices. Tint comes from the existing now-playing art mechanism, falling
back to the theme accent. This is not live backdrop blur.

The pane adds no shadow. Existing popups still use soft shadows; the discovery
options menu uses native offset `[0, 6]`, blur `20`, spread `0`, and the theme
shadow color. Window and generic popup shadows remain inherited and are listed
in the sidecar.

### Named Rules

**The Quiet Material Rule.** Keep the pane's light static and bounded; do not add an idle repaint loop for blur or shimmer.

Shared controls change hover/focus state directly: circles grow to 1.05 on hover,
and frameless icons scale to 0.92 while pressed. The existing busy spinner rotates
at 1.2 turns per second and requests a repaint after 33 milliseconds only while
drawn. These source facts are not motion or performance acceptance results.

## Shapes

Small rounded covers, native rows and menus sit within the more generous pane.
Pill buttons and exploration choices use half their measured height as the
radius; circle controls use half their diameter. Row hover/selection uses the
shared rounded highlight. The fixed radius tokens describe existing code values,
not a newly generalized radius ladder.

## Components

### Buttons

Pill actions retain semibold labels, 18-point horizontal and 8-point vertical
padding. Save taste uses accent fill and the matching hover fill; its light text
is black to meet the scoped contrast requirement. Secondary actions remain
transparent with a one-point dim outline that becomes text-colored on hover.
The native focus ring is one point, two points outside the allocated rect, with
four-point corners. A circle Play all action is 44 points with an icon sized at
46% of its diameter. Frameless header icons are 18 points within 30-point targets.

### Chips

Exploration choices are the existing soft-button primitive: medium labels,
12-point horizontal and 7-point vertical padding, surface fill at rest and hover
surface on interaction. The active choice inverts neutral text and window colors.
The shape is a pill; selected state does not require a green fill.

### Cards / Containers

The pane is the surface's material container, not a new repeating card family.
It has 14-point corners, a one-point outline and a 20-by-18-point inset. Music
remains in the inherited track-row primitive rather than card tiles.

### Inputs / Fields

Taste uses a native three-row multiline TextEdit with regular body type, the
existing surface states, six-point widget corners and accent caret/selection.
It expands to the available content width. Focus is visible in the native capture;
full keyboard traversal and assistive technology acceptance require separate evidence.

### Navigation

The existing sidebar and top bar remain the navigation system. This Home change
adds an ellipsis menu, not a new navigation family. Its overlay material, eight-point
corners, six-point inset and 28-point menu items come from shared widgets; the
menu's minimum content width is 220 points.

### Discovery Rows and Feedback

The 56-point row keeps native music identity: a 40-point cover with four-point
corners, a medium title, a secondary artist line and duration. Recommendation
reasons and feedback sit alongside it without changing the player row primitive.
The 28-point feedback toggles use 15-point inherited vector icons. Positive
selection uses accent/on-accent; Less selection uses text/window. Hover or focus
adds the hover surface. Library liking and recommendation feedback remain distinct.

## Do's and Don'ts

### Do:

- **Do** use the existing theme roles and real Inter weight helpers.
- **Do** keep static pane light below neutral text and interactive controls.
- **Do** preserve native row geometry and keep recommendation text bounded.
- **Do** preserve the local black light-theme Save taste label when using that component.
- **Do** keep shell changes outside this Home replacement unless separately requested.

### Don't:

- **Don't** turn cover tint into a new foreground or selection palette.
- **Don't** treat the static mesh as proof of a native live-blur compositor.
- **Don't** promote this Home composition into a rule for every future surface.
- **Don't** infer visual approval, motion quality or platform-wide validation from static macOS captures.

Not canonized or repaired: the selected external QUALITY BAR card and an approved
comp were not persisted. No broader design approval is claimed. Unrelated inherited
visual drift, motion, performance and platform behavior remain outside this scan.
