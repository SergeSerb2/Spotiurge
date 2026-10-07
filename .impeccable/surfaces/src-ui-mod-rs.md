---
version: 1
slug: "src-ui-mod-rs"
primary_target: "src/ui/mod.rs"
related_targets: ["src/theme.rs","src/ui/material.rs","src/ui/widgets.rs","src/ui/sidebar.rs","src/ui/player_bar.rs","src/util.rs"]
---

# Spotiurge scenery redesign

Mode: Operate. Native Rust/egui desktop, macOS first, Windows/Linux preserved.
Serge requested a clean natural mountain/scenery identity close to T3 Pretty,
less gradient-based, across the whole app. Working native interface first.
The existing discovery, feedback, sync and playback interactions stay intact.

THESIS: a quiet listening workspace in T3 Pretty's World Scenery family.
Music remains the foreground; the scenery gives the app a recognizable place.

OWN-WORLD: forest and mist palettes, flat translucent chrome, clear page
content over one locally bundled original mountain/lake scene and a flat
contrast wash. A mint ridge cut into three meter columns replaces the amber S.
Menus remain opaque. Inter and the existing native control vocabulary remain.

STORY: Serge opens the app, sees playable personal picks and the current song,
then listens, changes exploration or gives feedback without composing a prompt.

FIRST VIEWPORT: a narrow forest-glass sidebar with the bare ridge mark,
clear content column with the For you desk, and a moss play control in a
flat console. The mountain ridge remains quiet behind text in both schemes.

FORM: the user's pinned T3 Pretty World Scenery reference replaces the old
studio world. Existing native composition retained; inherited layout seed
39307beb is retained, rather than generating competing concept images.

MOTION: preserve the 220ms selection glide and page entrance, 120ms feedback,
and Reduce Motion. Scenery is static and never causes idle animation.

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance
