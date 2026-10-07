# Native Home discovery redesign review

Recorded October 6, 2026. The [comparison](index.html) presents actual native
macOS captures at matching theme and window sizes. `before-*` preserves original
fork main `e46f894`; `preview-*` preserves the prior delivered Home extension at
`5ba0ea9`, which Serge rejected. `after-*` and the current `state-*` family show
the working native personal radio desk replacement.

This record replaces the prior extension's design review. It documents the
implemented scope and the independent finish review's disposition. It does not
grant broader design approval or certify the product milestones.

## Scope and visual authority

The new Home pane opens with a 26-point For you heading and play, shuffle,
refresh and options controls. Familiar, Balanced and Adventurous choices follow.
One catalogue summary precedes compact 56-point playable rows with cover,
artist, duration, a bounded reason and two recommendation feedback controls.
Taste editing opens inline for onboarding or on request. Unplayable suggestions
and AI history are initially hidden; saved mixes use compact native rows.
The sidebar, library shelves, search, player and Connect retain the inherited
layout. The normal captures show the Home shelves below the new pane.

Serge requested a working native redesign, Opus 5.5 high-reasoning design advice,
additional discovery controls, automatic recommendations from saved taste and
feedback, and explicit GPT-6 Luna for recommendations. This visual review does
not independently verify service routing or automatic request behavior; those
claims belong to the [verification record](verification.md).

The current visual authority is [PRODUCT.md](../../../PRODUCT.md), the user's
scoped native Home request, the
[current direction contract](../../../.impeccable/surfaces/src-ui-discovery-rs.md)
and the implemented artifact. The direction seed proof at
`.qa/redesign-concept-seed-proof.txt` records key `0f7cc59f`, assigned candidate
`5`, before implementation. It establishes the grounded personal radio desk
direction; it does not establish user approval of an external image or an
implemented comp.

No selected external QUALITY BAR card was persisted. No approved comp exists
for this code-led implementation. This is a remaining process-evidence gap,
not a fabricated asset or a claim that the chosen-world comparison passed.
Serge's redesign request authorizes this Home visual scope. A finish verdict
does not authorize extending that scope to the inherited shell.

## Implemented system

[DESIGN.md](../../../DESIGN.md) and
[.impeccable/design.json](../../../.impeccable/design.json) now record the actual
native code values. This is the first code-derived record of the new Home world,
with inherited palette and shell decisions preserved. Frontmatter owns primitive
tokens; the sidecar carries native material formulas, interaction facts,
breakpoints and illustrative portable component previews. Its synthesized tonal
ramps are panel metadata, not additional application palette tokens.

[src/theme.rs](../../../src/theme.rs) supplies the inherited Inter typography,
paired native theme palettes, soft buttons, pills, circles and focus treatment.
The discovery feedback vectors are registered there; palette values are preserved.
The Home pane in [src/ui/discovery.rs](../../../src/ui/discovery.rs) uses the
surface color at `gamma_multiply(0.72)` in dark mode or `0.80` in light mode,
14-point corners, a one-point outline and 20-by-18-point inset. A static tint
mesh is reserved below the content, with cover-derived light or accent fallback.
It adds no blur pass, shadow or continuous repaint loop.

Reasons occupy a separate column only above 920 points of available content
width; below that threshold the complete reason is available on row hover.
Feedback keeps two 28-point targets and 15-point vector icons. More uses the
accent when selected; Less uses neutral inversion. The choices use neutral
inversion for active state. These are native controls, not web mockups.

The only change requested by the valid independent finish review was light
Save taste text contrast. Its initial white text on light accent measured
3.19:1. The fix overrides `on_accent` to black only in the taste editor's local
light-theme button palette. The shared theme and inherited controls retain their
source values. Black on light accent now measures 6.59:1 at rest and 5.03:1 on
hover, both above the 4.5:1 small-text threshold.

## Evidence and finish-review disposition

The current set comprises 28 native macOS captures:

| Capture family | Coverage |
| --- | --- |
| `after-{dark,light}-{normal,narrow}.png` | Current replacement in both themes at 1440-by-900 and 900-by-760 |
| `state-{matched,partial,not-found,onboarding,offline,busy,pairing,focus,history,feedback,menu,unmatched}-{dark,light}.png` | Twelve current demo states, each in both themes at 1440-by-900 |

All 28 were recaptured after the contrast fix. The independent reviewer validated
the recaptures and reported no regressions from that fix. The four original
`before-*`, four rejected `preview-*` and one earlier live Mac decoded-audio
image remain separate historical evidence. These families total 37 PNG-named
rasters. Every one carries embedded origin metadata; the provenance scan at
`.qa/redesign-provenance.log` records `SCAN: 37 rasters, 0 missing`.

| Finish finding | Resolution and disposition |
| --- | --- |
| Light Save taste white label fails small-text contrast | Scoped black foreground applied in `taste_editor`; rest 6.59:1, hover 5.03:1. Resolved. |
| Post-fix capture validation | All 28 current captures valid, with no regression from the scored fix. Pass. |
| Selected-world QUALITY BAR evidence | No selected card persisted; chosen-world parity is not established. Gap remains. |

The review verdict is pass, with a ship disposition for the scored contrast fix.
Its scope is static macOS source/capture evidence. It is not a broad system
approval, a chosen-world QUALITY BAR certification, or Serge's approval to merge
a wider redesign. Motion and performance were not assessed.

The screenshots use deterministic demo fixtures and identify sample picks.
Remote-playback gates and disabled demo feedback are visible in the fixtures.
Local candidate fixtures retain their playback controls. The selected-feedback
fixture represents stored demo state; it does not
prove a submitted live rating. Focus is requested programmatically in its fixture.
State captures are candidate-only at normal size, not before/after comparisons or
narrow-state validation. The fixture page, theme and window sizes are aligned,
but the animated player timestamp need not be pixel-identical.

No web detector ran because the artifact is Rust/egui. Source inspection and
native screenshots establish the visible material and layout. Full application
checks and supported platform coverage are reported separately in
[verification.md](verification.md).

## Remaining acceptance

The implemented Home world and the contrast fix are documented. The missing
selected QUALITY BAR asset remains explicit. This documentation does not repair
unrelated inherited visual drift or extend the new appearance to the shell.

Static demo screenshots do not establish authenticated playback, real model
output, recommendation quality, sync behavior, keyboard traversal or assistive
technology support. Windows native rendering/playback, custom-mix outbound
handoff and listening evaluation require their own evidence. iPhone independent
background playback, iOS architecture and TestFlight remain deferred and gated
by real-device evidence. See the
[architecture and milestones](../../_reference/spotiurge-architecture.md) and
the [verification record](verification.md) for product acceptance beyond this
visual review.
