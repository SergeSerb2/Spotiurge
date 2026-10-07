disposition: fix

Missing inputs: no QUALITY BAR card, no approved comp (code-led), no detector or hook findings (native egui). The ceiling check uses T3 Pretty `scenery.css` and `glass.ts` as the reference. Not read: `src/ui/*.rs` source (the review uses captures and pixel samples), `CONTRIBUTING.md`, and T3 Pretty files other than the scenery stack.

Evidence: all 82 named captures are present and valid. That is 41 before and 41 after: 17 surfaces x light/dark at 1440x900 (2880x1800 px), plus Home, Search and Settings at 900x760 (1800x1520 px). Each file shows the state its name claims and has no blank regions. I also inspected the user's error screenshot, `assets/brand/spotiurge-mark.svg`, `spotiurge-glyph.svg`, every `assets/brand/png` size and `assets/scenery/alpine-lake.jpg` (960x540).

## persistence

Pass, with one note.
- `PRODUCT.md` exists.
- The direction contract in `.impeccable/surfaces/src-ui-mod-rs.md` has THESIS, OWN-WORLD, STORY, FIRST VIEWPORT, FORM, MOTION and FINISH. FORM inherits layout seed `39307beb`, and the Home sub-contract has seed `0f7cc59f`. Neither seed can be checked against a roll record, but this is a code-led build that keeps the existing composition, as both contracts state.
- The build is code-led, so no comp state file is required.
- `DESIGN.md` still describes the old "studio control room / VU-amber lamp" world (amber tokens, `lamp-bar`). This is a new world, so the documenter rewrites it after this review, as the contract's FINISH line requires. It must not merge in its current state.

## fidelity

No comp. Each element is judged against OWN-WORLD, the confirmed answers and the before captures.

| Element | Verdict | Evidence |
|---|---|---|
| Topology / reading order (sidebar, top bar, page, console) | match | Same composition in all 17 surfaces and both sizes. The full-height page plate was removed, as the confirmed "clear main content" direction requires. |
| Sidebar and console chrome | match | Flat translucent forest/mist plates. No double rims, sheen or cover glow. |
| Scenery | match | One static, locally bundled photographic raster under a flat black/white wash. It is visible in every after capture and not buried: login shows it most clearly, light schemes show it as mist. |
| Popovers (overflow menu, Connect) | match | Opaque, with legible separation in both schemes. |
| Mark (sidebar, login, icon tile, tray template) | match | Bare mint ridge cut into three meter columns on a forest tile, consistent from 16 px to 1024 px. |
| Accent / primary action | match | Moss play control, active console toggles, now-playing title. Amber is fully removed. |
| TYPE | match | Inter, with the same scale and weights as before. |
| MATERIAL | match | Real raster scenery and flat chrome. No faked bevels or emboss. |
| GROUND | adaptation | The net ground is cool slate (dark top band about rgb 48,51,55, lower bands 15-22/20-27/24-33; light about 244,245,247). The chrome is forest-neutral (dark sidebar about 32,36,35). The hue comes from the photo under T3 Pretty's own black/white wash, as the "visually unified with T3 Pretty" answer requires. |
| Unmatched / partial results (user screenshot) | match | Repeated "Spotify match not verified" rows collapse into "Couldn't play (2)" with search affordances. |
| Unselected chips, dark scheme (Home exploration strip, Library filters) | contradicted | Chip fill ≈ host fill: chip rgb 25,33,28 against For you card 28,32,31, the same against the sidebar. "Familiar", "Adventurous", "Albums", "Artists" and "Podcasts" read as plain text. The light scheme and the dark Search filters still show pills, and the before dark capture had visible pills. |
| Content plates, dark scheme (Settings sections, Search Top result) | contradicted | Settings cards rgb 19-20,25-26,25-26 against washed ground 15-22,20-27,24-33, which is essentially a zero step. Section grouping depends on hairlines. The light scheme keeps a step of about 20 levels (244 against 220-223). |
| Disabled menu items (overflow menu "Save this mix", "Sync my devices") | contradicted | Brightest pixel of disabled against enabled text: dark 197/250 after, 124/247 before; light 73/20 after, 137/19 before. Disabled items now look enabled. The light "Save taste" disabled label (white on pale mint) goes the other way and is barely readable. |
| Blocking / failure notices (load error, model failure) | contradicted (inherited) | "Cannot read discovery state..." and "Recommendations are unavailable..." use the same secondary text and circle icon as the "Demo discoveries" info line. A blocking failure has the same weight as a demo footnote, even though danger and warning roles were strengthened for this purpose. |
| Active side stripes (sidebar nav, library row, now-playing rows, Connect row) | added without approval (inherited, floor) | Each is a 2-3 pt colored left bar on a list item. Fill and moss text already mark the state. |

## ceiling

- Mark distinctiveness: at sidebar size (about 22 pt) the three-column glyph reads as a bar chart or signal icon. It sits in the same viewport as the top bar's level-meter visualizer icon (ılılı), so two meter glyphs compete. T3 Pretty's mark has heavy cuts and real letterform character. This mark relies only on its sloped tops, and at 16 px those collapse toward plain rectangles. Within the approved concept, sharpen the ridge silhouette by deepening the shoulder cuts and offsetting the peak.
- The scenery has no unobstructed moment except login. That is acceptable for an Operate app, and I record no device as unused.
- Motion: none to judge. The scenery is static and finite motion is preserved.

## material_fixes

1. Dark unselected chips: give the Home exploration strip and the Library filters a fill one step above the host surface, about `#2c3a32` (surface_active) or equivalent. Match the pill visibility of the light scheme and of the dark Search filters, so one chip vocabulary holds across pages. (fidelity, contradicted, dark)
2. Dark content plates: raise the dark card/plate fill (Settings sections, For you card, Search Top result) so it steps visibly off the washed ground in the lower and darker bands. Aim for at least the light scheme's step. Use one elevation cue, fill or outline, not both. (fidelity, contradicted, dark)
3. Disabled state: create a disabled text/icon token separate from the strengthened `dim`/secondary. Restore a clear disabled step in menus: disabled ink near the old 124/247 dark ratio, with the light scheme equivalent. Keep disabled primary buttons readable as labels. The light "Save taste" label is near-invisible. (fidelity, contradicted, both schemes)
4. Failure hierarchy: render the discovery load error in the danger role and the model failure in the warning role, on both icon and text. Keep the demo/info line secondary. The model-failure line keeps its existing refresh affordance. (contract: "one honest catalogue status and retry"; inherited)
5. Floor, side stripes: remove the colored left bar from the active sidebar item, the library row, the now-playing track rows and the selected Connect row. The existing selected fill, moss text and playing meter carry the state. (craft-floor Refuse: colored border-left above 1px on list items; inherited)

## keep

Keep the flat T3-style stack exactly as built: one static photographic scene, a flat black/white wash, flat translucent chrome, opaque popovers, a moss-only accent and no glows or sheens. Fix the dark-scheme steps without adding blur, gradients or rims.
