# Whole-app native glass finish verdict

Opus 5.5 at high reasoning reviewed the frozen macOS demo artifact and all 52
candidate images, with their hashes checked against the manifest. The second
round's disposition was **ship**, within the seven material fixes raised by
the first round. It did not claim live playback, Windows coverage or measured
animation performance. Serge explicitly requested the whole-app visual scope.

The visible replacement is a smoked/milky glass music console with cover-lit
ambient fields, Inter typography, an amber live signal and a new surge S mark.
It applies to the shell, library, collections, search, settings, dialogs,
Connect, queue, lyrics, login, discovery and player, while preserving optional
Winamp skin artwork. It paints native layered materials without an OS blur or
browser. State changes use finite 120/220/600 ms transitions, with a six-point
page rise and immediate results under reduced motion.

The second round confirmed all seven requested fixes:

1. Menus, Connect and shortcut overlays are opaque, with no underlying text
   showing through.
2. Small live text uses a deeper light-theme amber. Measured text contrast on
   captured backgrounds ranged from 6.4:1 to 8.5:1; dark text remained 6.9:1 to
   9.9:1.
3. In 900-pixel windows, Connect folds to an icon and leaves a readable search
   field and a clear gap.
4. Every discovery feedback pair shares a fixed trailing column, including
   partial, error and narrow states.
5. The discovery options menu is about 293 pixels wide including margins,
   rather than about 600.
6. Taste and Settings fields share the search well's rim/focus recipe; the
   taste editor is capped at 560 logical points.
7. Exploration/settings choices use neutral inverted pills, cautions stay
   neutral, and the unresolved section uses the stroke chevron. Boolean
   switches keep their native amber on-state exception.

Retained residuals: active sort labels bypass the deeper amber text role;
dialog fields still use an older rim/radius recipe; light placeholders and
field rims are faint; shortcut wording remains “Cmd+”. These were recorded
without starting a new polish pass. Focus-ring typing is tested, but no focused
native capture was used as its proof.

[DESIGN.md](../../../DESIGN.md) and its structured sidecar were replaced after
the verdict with the source-derived palette, material, typography, components
and motion contract. See [verification.md](verification.md) for artifact
identity, fresh packaged-app checks and honest platform/performance limits.
