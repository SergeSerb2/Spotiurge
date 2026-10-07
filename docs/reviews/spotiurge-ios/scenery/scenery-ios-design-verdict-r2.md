disposition: fix (two small corrections; no new design delegate needed)

# Spotiurge iOS scenery design verdict, round 2

Reviewer: Opus design sub-agent. Design only. No code, build, test, git,
physical iPhone, credentials, network or audio was touched.

Inputs: `.qa/scenery-review-brief.md`; `.qa/scenery-ios-finish-review-r1.md`;
root `DESIGN.md`; candidate `.impeccable/surfaces/ios-interface.md`; all 26
PNGs in `t3code-3e6975b7-ios-candidate/target/ios-interface/raw-scenery-r3/`
(1206x2622, iPhone 18 Pro Simulator, iOS 27); and the roles in `Theme.swift`,
`Components.swift`, `SettingsView.swift`, `HomeView.swift`, `LibraryView.swift`
and `NowPlayingView.swift`. Pixel sampling uses darkest/lightest pixel per text
box with the WCAG formula; the script and contact sheets are in
`.qa/ios-design-r2-review/` (`measure.py`).

## Prior findings

1. **Bottom-bar collisions: fixed.** `roomBackground()` applies
   `scrollEdgeEffectStyle(.hard, for: .bottom)` to Home, Library, Search and
   Settings. In `home-*`, `error-*`, `settings-*` and
   `home-ax-reducedmotion-dark`, rows under the accessory and tab bar are
   blurred and illegible. In the AX frame, "Elysian" shows only as a blurred
   ghost in the edge band above the capsule, outside the footprint. The
   accessory text now fits its capsule at AX size (`.dynamicTypeSize(...xxxLarge)`).
   Acceptance met.
2. **Light Settings headers and footers: fixed.** Explicit `Palette.secondary`
   measures 7.3:1 (header) and 7.7:1 (footer) in light, 8.8:1 in dark.
3. **Symmetric blocks: fixed.** Banner, header, lamp, fader, notices and
   plates sit 16 pt from both edges. `roomRow(block: true)` hides separators;
   no stray dividers remain around the light banners.
4. **Disabled ink: fixed for ink, one surface regression (finding B).**
   `ReadableButtonStyle` and `LampButtonStyle` use `Palette.disabled` via
   `isEnabled`. Measured: disabled Sign in 5.3 / 5.8, Save this mix 5.3 / 5.8,
   Love/Less 5.5 / 5.4, lamp 3.9 to 4.1 dark / 4.8 light. Disabled "Write your
   taste" is 3.9 dark / 3.7 light on its capsule, up from about 2.2. Disabled
   ink is outside DESIGN.md's Measured Ink Rule and matches the desktop pill
   design, so these values are accepted.
5. **Stock grey grouped rows: fixed.** Dark Settings, identity card and the
   Discovery options list render forest `#1A221E`; light is white. The options
   sheet gap from the title to the first row is now about 30 pt, down from about 45.

## Material findings, ranked

A. **Light Settings status values fail the 4.5:1 ink floor.**
   - Evidence: `settings-light` and `settings-demo-light`. "Not in this build",
     "Not paired" and "Demo: no audio" render as system grey `#8A8A8E` on white,
     3.44:1. Dark passes at 5.8:1.
   - Cause: `LabeledContent(..., value:)` uses the system secondary style. Round 1
     corrected the headers and footers, but these values still use that style. The same path renders
     "Signed in", "Keychain unavailable", "Last sync" and "Version".
   - Fix: make the Form's hierarchical secondary style `Palette.secondary`.
     For example, apply `.foregroundStyle(Palette.text, Palette.secondary)` on
     the Form, or use a small `LabeledContentStyle` if that does not reach the value.
     These values are the status a user reads to diagnose pairing and playback.
     They must meet 4.5:1 like every other text role.

B. **Dark disabled CTA capsule disappears on its content plate.**
   - Evidence: in `onboarding-dark`, the disabled "Write your taste" capsule
     fill is `(44,58,50)`. The plate under it is `(41,54,48)`, about 1.05:1,
     so the pill loses its shape and the label floats.
   - This is the exact DESIGN.md "Don't": do not paint a dark disabled pill
     with opaque `surface-active`, because it disappears on the dark content plate.
   - Fix: in `ReadableButtonStyle`'s disabled glass branch, use the desktop
     rule. Dark uses a white overlay at 8%, and light keeps `surfaceActive`.
     Light already reads as a clear neutral pill.

No other material findings. The keep list from round 1 is intact:
- the static photo and wash;
- the bare ridge glyph;
- the single moss voice;
- the neutral inverted fader pill and its vertical AX stack;
- the opaque forest taste and options sheets;
- the Now Playing sheet over the room;
- the native tab bar and search.

## Can root confirm without another cycle

Yes. Neither fix changes layout or appearance beyond the two named surfaces.
Root can confirm both from source plus a targeted recapture of its own:
`settings-light`, `settings-demo-light` and `onboarding-dark`, sampled with
`.qa/ios-design-r2-review/measure.py`. Pass criteria:
- Light status values are at least 4.5:1. `Palette.secondary` on white is
  already pixel-proven at 7.3:1 in this Form.
- The dark disabled capsule is visibly distinct from the plate.
- Disabled ink stays at or above 3.5:1.

Do not run a new design delegate or a full 26-frame cycle.

## Final-evidence gaps (not approved, not claimed)

- **No enabled feedback controls in pixels.** Every Love/Less capture is demo.
  The enabled `Palette.dim` ink is unmeasured.
- **Live states not captured:**
  - signed-in Settings values;
  - "Keychain unavailable";
  - paired sync;
  - a live Luna answer;
  - audio.
- **AX coverage is limited.** AX captures exist only in dark and only for Home
  and Now Playing. Light AX and Settings at AX are unseen.
- **Unverified display and motion states:**
  - motion quality: the fader glide, tab-bar minimize, and sheet present and dismiss;
  - Reduce Motion behaviour beyond a static frame;
  - VoiceOver;
  - Increase Contrast;
  - Reduce Transparency (the accessory without glass);
  - landscape;
  - iPad.
- **Not a production candidate.** It is Simulator only and the stock engine
  identity is still blocked. Physical-phone checks are stopped.
