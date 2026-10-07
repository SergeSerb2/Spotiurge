---
version: 1
slug: "src-ui-mod-rs"
primary_target: "src/ui/mod.rs"
related_targets: ["src/theme.rs","src/ui/widgets.rs","src/ui/sidebar.rs","src/ui/player_bar.rs","src/util.rs"]
---

# Spotiurge whole-app glass redesign

Mode: Operate. Scope: every app-owned native surface (sidebar, top chrome,
Home/discovery, library shelves, search, collection pages, queue, lyrics,
menus/popovers/Connect, settings/login/about/errors, player bar, compact
player chrome) plus the Spotiurge mark. Optional Winamp skins keep their own
pixels. Serge authorized the whole-app visual scope on October 6, 2026, chose
a working native interface before concept images, and pinned liquid/frosted
glass with fluid motion. Mac first; iOS is a separate gated implementation; desktop scope unchanged.

## Direction contract

THESIS: the studio control room. Music lives in the lit live room behind the
glass; Spotiurge is the console in front of it. Refuses the category default
of a flat black Spotify clone with green accents and card grids.

OWN-WORLD: smoked double-pane glass panes floating over a dim room washed by
the playing cover's light. Panes carry a top specular sheen, a two-tone rim
(lit top edge, shaded bottom) and soft offset shadows. One VU-amber signal
lamp marks live state only: playing, primary play, the current navigation
channel. Choices (chips, segmented controls, settings options) select with a
neutral inverted pill instead, a strong native selection that leaves the
amber to playback and navigation; cautions and errors stay neutral too.
Overlays (menus, popovers, dialogs) are opaque glass. Inter
throughout, rank by weight. The mark is an amber surge S on a smoked glass tile.

STORY: Serge sees what is playing and what to play next at once, navigates
like moving a fader between channels, and every control answers immediately.

FIRST VIEWPORT: floating sidebar pane with mark and wordmark, gliding lamp
selection; central glass pane with the cover-lit header and Home's For you
radio desk; floating console pane along the bottom with transport centred.

FORM: candidate 5 of 7 (studio control-room window), seed 39307beb. Ordered
list: hi-fi receiver faceplate, DJ booth at night, record crate, personal radio
desk, studio control-room window, festival ticket stubs, spectrogram data
graphics. Raises: split-flap, fixed cells and tabular time so state changes
never shift geometry; centre-rail, rims drawn crisp at one physical pixel;
boarding pass, the single lamp colour reserved for live state; Crouwel, every
size on a 4-point grid; timetable rack, rank carried by weight on a tight
scale; Saville, inactive items stay neutral and only the live item takes
colour. All six challengers declined on audience identification and product
clarity; QUALITY BAR references kept in .qa/quality-bar/.

MOTION: one signature, the fader glide: selection travels between channels
with exponential ease-out, the amber lamp along the sidebar and the neutral
inverted pill across chips and segmented controls (root decision after
finish review r1, item 7). Page content cross-fades
and settles 6 points in 220 ms; hover/press fills ease in 120 ms; menus fade
from the platform. Motion is finite, repaints only while moving, and reduce
motion (setting or macOS preference) makes every transition immediate.

MATERIAL HONESTY: rendered frosted glass, not OS backdrop blur. Ambient light
is a few cached soft colour fields; no per-frame blur pass.

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance
