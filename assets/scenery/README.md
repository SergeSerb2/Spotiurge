# Bundled mountain scenery

`alpine-lake.jpg` is an original generated scenery asset, not a named place or
an actual photo of a claimed location. It was made with the built-in image_gen
tool on October 7, 2026 and converted to JPEG with macOS `sips`. It is bundled
locally; displaying it sends no requests or personal data to an image service.

Prompt: an original Pacific Northwest alpine lake with misty slate-blue
mountain ridgelines, distant evergreen silhouettes, cool overcast daylight,
soft stone/forest/sage tones and a calm sky. Wide composition for a native
music app beneath flat light/dark legibility washes. No text, logos, people,
buildings, neon, glow or saturated gradients.

It is the Alpine Lake setting, and stands in while an Unsplash photo loads.

## Unsplash photo sets

`photos.tsv` is the seed pool of T3 Pretty's five photo sets, and
`catalog.tsv` the curated places the app searches Unsplash for. Both are
generated from T3 Pretty's `apps/web/src/scenery` by
`contrib/scenery/build-photos.py`, which also measures each photo's mean
and accent colour from a small thumbnail, the same way the app measures
album covers, so pages can pick the closest photo. Photos stay on Unsplash
and load at run time under the Unsplash License, credited to their
photographers in Settings.

The app draws one cached photo under theme-coloured flat washes. It does not
animate scenery or apply a per-frame backdrop blur. The native shell follows
T3 Pretty's World Scenery vocabulary while preserving readable music content.
