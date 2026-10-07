disposition: fix (one root cause, small)

Scope: verdict on the five material fixes from `.qa/scenery-finish-design-review-r1.md`. No new polish hunt. Design only. No code, test, PR or device operations.

Reviewer note: the `impeccable-finish-reviewer` agent type is not installed in this environment. This pass ran directly on Opus as the design sub-agent, with the same scope and inputs.

Evidence: candidate binary `fe3c5d2f...0253`, `docs/reviews/spotiurge-scenery/after-captures.json` (40 after captures, macOS arm64, source commit e2a68a8 plus the uncommitted r2 state in `.qa/scenery-source-state-r2.json`). I inspected these captures visually: home dark, home-menu light and dark, home-taste light, home-load-error dark, home-error light, settings dark, search dark, devices dark and liked light. I took pixel samples from home dark and light, home-taste dark and light, and settings dark (JPEG derivatives; values are approximate to about 2 levels). Source read: `src/theme.rs` (palette, `disabled_text`, `choice_chips`, `pill_button`), `src/ui/material.rs` (`glass`, `lamp_shapes`, `row_highlight`), `src/ui/widgets.rs` (menu item and pill colours), `src/ui/discovery.rs` (notices) and `src/ui/show.rs` (episode row).

## Scores

| # | Finding | Score |
|---|---|---|
| 1 | Dark unselected chips | partial (regressed on Content plates) |
| 2 | Dark content plates | resolved |
| 3 | Disabled state | partial (light resolved, dark disabled primary lost its shape) |
| 4 | Failure hierarchy | resolved |
| 5 | Side stripes | resolved for all named targets, one residual outside the captures |

### 1. Dark unselected chips: partial

- Library filters (sidebar): resolved. Chip rgb 45,58,49 against sidebar 30,34,33. The pills read clearly.
- Search filters: resolved (they sit on the clear ground).
- Home exploration strip ("Familiar", "Adventurous"): unresolved, and now a regression from the before capture. Chip 45,58,49 against the For you card 45,58,51, so the step is zero. Before, the step was 28,32,43 against 49,44,40, which was visible. In the dark home, home-menu, home-load-error, devices and taste captures, these chips read as plain text again.
- Settings "Audio quality" choices (Normal, High): nearly unresolved. Chip 45,58,49 against the Settings plate 40,53,46.
- Cause: fix 2 made the dark `Kind::Content` base `surface_active` at 85%. `theme::choice_chips` paints dark chips with opaque `surface_active`. A chip on any Content plate now matches its host.

### 2. Dark content plates: resolved

- For you card and Settings sections now step clearly off the washed ground: card about 45,58,51, with the ground in the 15-33 range. The step is at least as strong as in the light scheme.
- Search Top result uses one brighter fill and has no outline. That is one elevation cue, as asked.
- Keep it as built. Do not undo this to repair fix 1.

### 3. Disabled state: partial

- Menus: resolved in both schemes. In dark, "Save this mix" and "Sync my devices" are clearly grey against white enabled items. `disabled_text` 0x84988b against text 0xf3f6f3 matches the old 124/247 step. In light, 0x59695e against 0x161a17 gives a visible step. Icons follow the same ink.
- Light disabled primary ("Save taste"): resolved. Ink 88,99,91 on fill 202,209,202 gives about 4.0:1. It reads as a disabled button.
- Dark disabled primary ("Save taste" in home-taste dark): unresolved. The `pill_button` disabled fill is `surface_active` at full opacity, and the For you card is `surface_active` at 85%. The fill 45,58,49 sits on a card of 45,57,53, so the pill shape disappears. "Save taste" now reads as loose grey text next to the outlined "Cancel" pill. Same cause as fix 1.

### 4. Failure hierarchy: resolved

- Load failure (`home-load-error`): danger icon and danger text. The full recovery sentence is wrapped (`discovery.rs` lays it out with `usize::MAX` rows). It reads as the blocking state.
- Model failure (`home-error`): warning icon and warning text. Hover detail is preserved. It is distinct from the secondary "Demo discoveries" info line and the "Choose this computer" note.
- `notice()` derives text colour from the icon role. That is acceptable.

### 5. Side stripes: resolved, one residual

- `material::lamp_shapes` now paints only the selected fill. Sidebar nav, library row (Liked Songs selected), now-playing rows (moss title and meter glyph) and the selected Connect row show no leading bar in the captures. Fill, moss text and the meter carry the state.
- Residual: `src/ui/show.rs:255-261` still paints a 3 pt accent bar at the leading edge of the current podcast episode. The podcast show page is not in the 17 captured surfaces. It is the same craft-floor item, so delete those lines. Accent text already marks the current episode (`show.rs:249`).
- Stale comments describe the removed bar: the `lamp_shapes` doc at `material.rs:241-242` and `discovery.rs:743`. They have no visual effect. Update them when the code is next touched.

## Required fix (one root cause)

In the dark scheme, chips and the disabled primary pill use opaque `surface_active`. That is now also the dark Content plate's base. Make these two fills relative to their host, so they keep a step on any surface:

- `theme::choice_chips`, dark unselected chip: replace opaque `surface_active` with a white overlay, for example `Color32::WHITE.gamma_multiply(0.08)` at rest, rising toward about 0.12 under hover. This matches the existing `material::selected_fill` idiom. On the sidebar it gives about the current visible step. On the For you card and Settings plates, it restores a visible pill. The current dark hover lerps toward `surface_hover`, which is darker than `surface_active`. The overlay also corrects that, so the chip no longer darkens on hover.
- `widgets::pill_button`, dark disabled primary fill: use the same overlay, not `surface_active`. Keep the ink at `disabled_text()`.
- Light scheme: no change.
- Keep the Content plate fill from fix 2.
- Remove `show.rs:255-261` in the same batch.

Verification for the root's next round: recapture only the dark captures that show Content-hosted chips or the disabled pill: home, home-narrow, home-menu, home-taste, home-load-error, home-error, home-onboarding, home-unmatched, devices and settings (normal and narrow). Then sample chip against host. Target a step of at least the sidebar chip's current step (about 15 levels). The pixel contrast tests should still cover chip label text on the new fill.

## Not blocking

- No other change was found in the recaptured surfaces. The scenery stack, opaque popovers, moss-only accent, absence of glows and the typography all hold as in r1 "keep".
- `DESIGN.md` rewrite (r1 persistence note) is still owned by the documenter. It is not part of this verdict.
- Logo sharpening remains an optional ceiling note, as the root decided.
