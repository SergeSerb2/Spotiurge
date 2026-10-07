# Scenery preview verification

Recorded October 7, 2026. This extends the [discovery](../spotiurge-discovery/verification.md)
and [glass preview](../spotiurge-glass/verification.md) records. Those earlier
integration tests are not tests of this new package.

## Native interface

The [comparison](index.html) pairs 40 before and 40 after native macOS arm64
captures: 17 surfaces in light/dark at 1440 × 900 logical, with Home, Search and
Settings also at 900 × 760. Retina images retain native resolution as JPEG
quality 92; original PNG hashes remain in the manifests. Demo data is synthetic,
labelled, and isolated from Spotify/cloud credentials and requests.

Before is commit `e2a68a8`, binary SHA-256
`7b393ac9841a5e6764587ee5683bd1b14961e89e44f13a03aef7f47b1ed36820`.
After is `4736cd8` plus the recorded dirty source closure in
[after-captures.json](after-captures.json), binary SHA-256 `3faae7ac9cff07f6855ec5d504cb7c9e56e32525c19ab6211490cb60c1c4b6da`.
The capture executable includes demo support; distributed executables exclude it.

The replacement identity follows T3 Pretty's forest/mist scenery language:
one original local mountain/lake photo, a flat contrast wash, quiet translucent
chrome, clearer content plates and a mint ridge/meter mark. Ambient cover glows,
sheens, double rims and decorative playing bars are removed. Layout, native
controls, Inter, finite transitions and Reduce Motion remain. The optional art
colour setting retains a 4% flat tint in the player console. There is no new
image service, shader, browser engine, telemetry or continuous background animation.

## Design review and final correction

Opus 5.5 high performed design-only review and verdicts
([r1](design-review-r1.md), [r2](design-verdict-r2.md),
[r3](design-verdict-r3.md)). The named `impeccable-finish-reviewer` agent was not
installed in its environment; Opus reviewed directly with the same scope.
No HTML/CSS detector applies to the native desktop UI. The documenter also
found a hard-coded amber notch-player accent; it now derives from the shared
dark palette. The notch surface was not captured; this separate native change
is compile/test verified, rather than attributed to the r4 demo captures.

The r3 verdict found that the requested dark-chip edit had not actually applied.
Root corrected the exact `choice_chips` branch and the stale selection-bar
comment, rebuilt, and recaptured all 40 states. The reviewer explicitly allowed
root confirmation of this remaining edit without another review round.
Root inspected the resulting Home/Settings/taste/error captures: unselected
Familiar/Adventurous and Normal/High options have visible pills comparable to the
Library options. The white overlay rises from 8% to 12% on hover, so it brightens.
A regression checks option-label contrast over every bundled JPEG pixel in both
themes, alongside the existing six text roles, translucent chrome, content plates
and hover/selection grounds. All must reach 4.5:1. Disabled control text is a
separate quiet role. This records the final correction; it is not an invented
fresh overall Opus ship verdict or measured motion-smoothness claim.

## Checks and behaviour

All required local checks passed on Apple Silicon macOS: launcher tests,
formatting, default/all-feature all-target Clippy with warnings denied,
default/all-feature all-target tests (996 / 1019 library tests passed, two
ignored, plus binary/integration/example targets), all-feature doctests,
rustdoc with warnings denied, gettext check and Jekyll build.
The independent private-cloud suite passes 16 tests. The translation refresh
changes only source locations and the POT date in 15 catalogues; semantic entries
are identical. Brand tests compare every committed PNG and template against the
runtime raster. The original generated scenery and raster intent are documented
in assets; the provenance scan reports ten shipping rasters with zero missing prompts.

The PR fixes also bound feedback to the newest 500 records, including clears,
with a shared logical tombstone cutoff preventing stale offline resurrection.
Equal-clock cohorts are forgotten together. Tests exceed 2,000 rating/clear edits,
merge two large legacy snapshots, preserve post-sync edits, and keep mixes/history
usable. Both Flatpak manifests grant Spotiurge's MPRIS name. A local dummy receiver
verifies the configured Connect sender name and derived identity; it uses no real
grant. No Flatpak runtime or physical receiver acceptance was performed here.

## Performance and platform scope

The earlier scenery r1 paused-demo comparison sampled 60 seconds per executable
after 15 seconds settling. CPU was 0.03 seconds, about 0.05% of one core, for
both baseline and candidate; resident memory differed by about 1.3 MiB. This is
a narrow static-demo observation of r1, not a final-package benchmark, frame-rate,
startup, live-session or endurance measurement.

- Apple Silicon Mac: native captures, full local checks and universal packaging.
  Exact new-package auth/sync/playback results, when run, are recorded separately.
- Intel Mac: release compilation and universal payload checks, no native launch.
- Windows/Linux: exact pushed-head CI is separate. Earlier real Windows sync is
  in the discovery record; no new Windows/Linux UI or audio run is implied.
- iOS: a separate native candidate follows the desktop. Physical-phone operations
  remain stopped at Serge's request. The independent probe proved 17 min 22 s
  locked playback with reconnect using an ignored identity experiment; stock
  production identity and TestFlight remain blocked. No workaround is shipped.

Recommendation quality still needs listening feedback. Catalogue rate limiting
can leave suggestions unchecked, and unchecked suggestions stay unplayable.
The app uses only GPT-6 Luna through CLIProxyAPI for recommendations. Spoken DJ,
production signing, three-platform app acceptance and TestFlight are not complete.

## Final PR findings

The later cooldown, saved-mix and probe sign-in findings were fixed and checked
separately. See [review-fixes-r3/verification.md](review-fixes-r3/verification.md)
for the full checks (1003/1026 library tests), host-only credential regressions
and four matched native UI pairs. These do not change the scenery design.

The subsequent [restart/catalogue/probe review fixes](review-fixes-r4.md) passed
the full checks with 1004/1027 library tests. They preserve the same appearance
and add no physical-device coverage.

The later [diagnostic and sign-in identity fixes](review-fixes-r5.md) add an
actual-file reopen regression and correct the browser-visible fork name.
That record also distinguishes the r7 package's restored state from its failed
live catalogue retry, without claiming new playback or listening-quality proof.

The [startup and connection fixes](review-fixes-r6/verification.md) protect failed
loads from exploration edits and load local discovery independently of proxy
credential restoration. Four matched native pairs show the disabled controls.
Host-only probe connection cancellation tests do not extend the physical gate.
