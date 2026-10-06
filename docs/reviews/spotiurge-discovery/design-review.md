# Native Home discovery review

Recorded October 6, 2026. Baseline: fork main at `e46f894`. Candidate: the
working-tree Home extension on that baseline. The [comparison](index.html)
contains actual macOS native demo captures, with light/dark and normal/narrow
selectors and separate candidate state captures.

This record documents the scoped implementation and the disposition of the fresh
finish review. It does not grant new independent approval, establish a new visual
world, or certify the broader product milestones.

## Scope and visual authority

Home now starts with a Spotiurge heading, a personal taste prompt, recommendation
actions, explained catalogue matches, intentional feedback and saved mixes. The
existing library shelves follow the discovery workspace. The populated captures
put them below the first viewport; the empty-state captures show that they remain
present. The sidebar, search, player and Connect controls retain their incumbent
appearance and position.

The authority for this ordinary extension is the inherited native interface,
[PRODUCT.md](../../../PRODUCT.md), the
[surface direction contract](../../../.impeccable/surfaces/discovery.md) and the
matched baseline captures. No approved redesign comp or visual-world seed was
selected. The wider liquid/frosted glass redesign, richer motion and platform-wide
branding remain separate work. macOS is the active priority; Serge deferred iOS.

## Incumbent system checked

[src/theme.rs](../../../src/theme.rs) remains the palette and widget source of
truth and is unchanged from the baseline. These are observed existing roles, not
a new token specification:

| Role | Dark | Light | Use in this extension |
| --- | --- | --- | --- |
| Window | `#0f1114` | `#f8f9fb` | Existing Home background |
| Surface | `#1d2127` | `#eef0f3` | Prompt material and native field |
| Outline | `#2a3038` | `#dde1e6` | Prompt boundary |
| Text | `#f2f4f6` | `#14171a` | Headings, entered taste and track titles |
| Secondary | `#a9b1bc` | `#535b66` | Supporting copy, reasons and status |
| Accent | `#1ed760` | `#15a64a` | Existing primary action and selection vocabulary |

The existing font setup uses Inter at real regular, medium, semibold and bold
weights, with its incumbent emoji and script fallbacks. Discovery uses a bold
36-point product heading, regular 16-point introduction and input, bold 24-point
prompt title, bold 20-point recommendation heading, regular 14-point help and
regular 13-point reasons/status. Shared pill buttons retain semibold 13-point
labels, 18-by-8-point padding, a half-height corner radius, outlined secondary
actions and accent-filled primary actions. Track rows reuse the existing native
row component rather than introducing a new card family.

[src/ui/discovery.rs](../../../src/ui/discovery.rs) adds one quiet prompt frame:
the current surface color at `gamma_multiply(0.88)`, a one-point outline,
16-point corners and a 20-point inset. It creates depth through translucent tonal
layering; it is not a live background-blur or liquid-glass compositor. No new
shadow or motion system was added. The shared theme still supplies existing
widget states and focus treatments.

[src/ui/home.rs](../../../src/ui/home.rs) draws discovery and then the inherited
`library_shelves`. Action groups, recommendation headings, feedback and saved
mix controls use wrapping layouts. At 900-by-760 the taste text and feedback
wrap cleanly while the player and Connect remain accessible. At 1440-by-900
the reasons and feedback share a row. Focus is visible on the prompt in both
themes; the shared pill control also calls the incumbent `focus_ring` helper.

No root `DESIGN.md` or `.impeccable/design.json` was introduced by this scoped
documentation pass. This review preserves the native implementation as the
system authority rather than inventing portable CSS tokens or a new identity.

## Evidence and finish-review disposition

All 18 PNGs in this directory were inspected. Eight populated captures compare
baseline/candidate at 1440-by-900 and 900-by-760 in both themes. Ten additional
captures show candidate states at 1440-by-900 in both themes. Pixel dimensions
match the comparison labels. Artwork in the final baseline and candidate
captures has settled; the same inherited demo fixture URLs supply the covers.

The fresh reviewer requested the following changes. The final source and
captures document their implementation; this disposition is not a second
independent reviewer approval.

| Review finding | Final evidence and disposition |
| --- | --- |
| Demo feedback appeared available despite disabled external work | `app.discovery.ready && !app.offline` now guards both feedback buttons. Populated captures show muted feedback, and the prompt status explicitly says feedback, recommendations and cloud requests are disabled. Resolved for the demo surface. |
| Baseline artwork had not settled | Final `before-*-*.png` captures contain loaded cover art, matching the inherited fixtures used by the candidate. Resolved in the recorded comparison. |
| Empty state was missing | `state-empty-*.png` shows an empty taste field with its hint, no recommendation section and preserved Home shelves. Added in both themes. |
| Loading state was missing | `state-loading-*.png` shows “Finding music…”, a spinner and a status explaining that playback stays available; prior picks remain visible. Added in both themes. |
| Error state was missing | `state-error-*.png` states that recommendations are unavailable and previous discoveries are kept; rows and player remain visible. Added in both themes. |
| Selected feedback was missing | `state-feedback-*.png` shows the first “More like this” selection using the incumbent green selection treatment. This is a preselected, disabled demo fixture, not evidence of a live submitted rating. Added in both themes. |
| Keyboard focus was missing | `state-focus-*.png` shows a green focus outline on the taste field. The fixture requests focus programmatically; full keyboard traversal and assistive-technology behavior are not established by a still image. Added in both themes. |

The captures are native application screenshots, not generated imagery. Demo
recommendations identify themselves as samples for visual review.
[src/demo.rs](../../../src/demo.rs) supplies the synthetic state fixtures and a
Spotify-shaped fixture URI for the selected-feedback example; no service receives
that fixture. No detector ran because the surface is Rust/egui. Review evidence
consists of source inspection and the native captures.

The baseline player timestamp reads 1:28 and the populated candidate reads 1:26.
Fixtures, page, theme, zoom and window sizes match, but the elapsed playback
indicator is not pixel-identical. Additional states are candidate-only and
normal-size; they are not before/after or narrow-state comparisons. These limits
do not conceal the changed layout, but prevent a claim of exact pixel matching.

## Remaining acceptance

The scoped Home extension and the requested finish evidence are recorded. The
translucent prompt is an incremental material treatment; the broader glass
redesign still needs a defined visual scope, implementation and Serge's explicit
visual approval before merging that redesign.

These demo screenshots do not demonstrate authenticated Spotify playback, real
CLIProxyAPI output, recommendation quality or cloud synchronization. The separate
[verification record](verification.md) reports live Mac playback, playlist Connect
handoff and real Mac/Windows offline/concurrent sync. Remaining acceptance includes
custom-mix outbound handoff, Windows native rendering/playback and listening
evaluation against a fixed Spotify baseline. Saved-mix playback and the expanded
AI history are present in source but are not shown in this capture set. Full
application checks and platform build results must be reported separately.

iPhone independent background playback, iOS architecture and TestFlight remain
deferred and gated by real-device evidence. No iOS framework choice or quality
parity claim follows from this work. See the
[architecture and milestones](../../_reference/spotiurge-architecture.md) for
the remaining product and platform acceptance.
