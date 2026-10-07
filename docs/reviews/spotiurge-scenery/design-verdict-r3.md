disposition: fix (one remaining item, one line of source)

Scope: final verdict on the two findings left open by `.qa/scenery-finish-design-verdict-r2.md` and the podcast-bar residual. No new polish hunt. Design only. No code, test, PR or device operations.

Reviewer note: the `impeccable-finish-reviewer` agent type is not installed in this environment. This pass ran directly on Opus as the design sub-agent, with the same scope and inputs.

Evidence: binary `fc38912b...6dd2c` per `docs/reviews/spotiurge-scenery/after-captures.json`. On-disk hashes of `src/theme.rs` (`5b599801...`), `src/ui/material.rs`, `src/ui/discovery.rs`, `src/ui/show.rs` and `src/ui/widgets.rs` match `.qa/scenery-source-state-r3.json`. So the source I read is the source that was captured. Captures inspected: dark home normal (plus a full-resolution crop of the exploration strip), dark home-taste, dark settings normal. No pixel tool was available this round (no PIL), so the judgements are visual, at full resolution.

## Scores

| # | Finding | r2 | r3 |
|---|---|---|---|
| 1 | Dark unselected chips on Content plates | partial | unresolved |
| 3 | Dark disabled primary pill | partial | resolved |
| 5 | Podcast episode leading bar | residual | resolved |

Findings 2 (dark content plates) and 4 (failure hierarchy) were resolved in r2. Nothing in r3 touches them. I did not recheck them beyond the captures above, where both still hold.

### 1. Dark unselected chips: unresolved

- The described change is not in the source. `src/theme.rs:972-975`, in `choice_chips`, still paints the dark unselected chip as opaque `palette.surface_active.lerp_to_gamma(palette.surface_hover, lift)`. No `WHITE.gamma_multiply(0.08 + 0.04 * lift)` exists anywhere in `src/`. The only `0.04 * lift` hit is `material.rs:226`, which is the key fill, not the chips.
- The captures agree with the source. In `after-home-dark-normal.jpg`, "Familiar" and "Adventurous" have no visible pill on the For you card. They read as loose text beside the white "Balanced" pill, the same as in r2. `after-home-taste-dark-normal.jpg` shows the same strip. In `after-settings-dark-normal.jpg`, "Normal" and "High" show only a faint shape on the Settings plate. The sidebar Library chips still read, as before.
- Likely cause: the edit went to `pill_button` (see 3) but not to `choice_chips`, or it was lost before the r3 source snapshot was taken. The r3 recaptures are consistent with the unchanged source, so the capture set is not stale. The source is.

### 3. Dark disabled primary pill: resolved

- `src/theme.rs:755-761`: the dark disabled primary fill is now `Color32::WHITE.gamma_multiply(0.08)`. The ink stays `disabled_text()`. The light branch is unchanged (`surface_active`).
- In `after-home-taste-dark-normal.jpg`, "Save taste" now has a visible, quiet pill on the For you card. It reads as a disabled button beside the outlined "Cancel" button. The hierarchy is correct: the disabled pill is quieter than the enabled outline and much quieter than the moss primary.

### 5. Podcast episode bar: resolved

- `src/ui/show.rs` no longer paints a leading bar for the current episode. Only `accent_text()` on the title marks it (around `show.rs:249`). This matches the rest of the app.
- Cosmetic, not visual: the doc comment at `src/ui/material.rs:241-242` still says the lamp has "a short bar of signal colour at the leading edge". The function paints only the fill. Root said the obsolete bar comments were removed, but this one remains. Fix it in the same edit as item 1. It does not block on its own.

## Required fix

1. In `theme::choice_chips`, dark branch (`src/theme.rs:972-975`), replace the opaque `surface_active` to `surface_hover` lerp with `Color32::WHITE.gamma_multiply(0.08 + 0.04 * lift)`. Keep the light branch as it is. This is the change the r2 verdict asked for and the r3 brief describes.
2. Optional, in the same edit: reword `material.rs:241-242` to describe only the fill.

Verification: rebuild, then recapture only dark `home` (normal, narrow), `home-taste`, `home-onboarding`, `home-unmatched`, `home-menu`, `home-error`, `home-load-error`, `devices` and `settings` (normal, narrow). Confirm the following:
- "Familiar" and "Adventurous" show a visible pill on the For you card.
- "Normal" and "High" show a pill on the Settings plate as clear as the sidebar Library chips.
- The pill does not darken on hover.
- The chip-label contrast test still passes on the new fill.

Light captures need no recapture. Then update the source closure and binary hash. No further design round is needed if those four checks hold. The root can confirm them against this verdict without a new review.

## Not blocking

- Everything else is as in r2 "Not blocking": scenery stack, opaque popovers, moss-only accent, no glows, typography. The `DESIGN.md` rewrite still belongs to the documenter. Logo sharpening is still an optional ceiling note.
