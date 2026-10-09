disposition: fix

# Spotiurge iOS scenery finish review, round 1

Reviewer: Opus design sub-agent, reviewing directly. The named
`impeccable_finish_reviewer` agent was not available in this environment, so
this review applies its checks and output contract by hand. It covers design
only. It did not change code, run tests, use git, use a physical iPhone or
handle credentials.

Inputs read: the brief at `.qa/scenery-review-brief.md`; the impeccable
`craft-floor.md`, `reference/ios.md` and finish-reviewer contract;
`.impeccable/surfaces/ios-interface.md`; root `DESIGN.md`, which uses the
forest/mist scenery; the desktop reference
`docs/reviews/spotiurge-scenery/after-home-dark-normal.jpg`; all 26 PNGs in
`target/ios-interface/raw-scenery-r1/`; and the parts of `Theme.swift`,
`Components.swift`, `SpotiurgeApp.swift`, `NowPlayingView.swift` and
`SettingsView.swift` that the findings needed.

Not read:
- `docs/reviews/spotiurge-ios/interface-design-review.md` and
  `interface-design-verdict.md` are not in either worktree. Older notes exist at
  `.qa/ios-interface-review-r{1,2}.md`, but this round did not open them.
- `after-home-light-normal.jpg`.
- The full `HomeView.swift` and `LibraryView.swift`. Only the lines grep found
  were read.

Contrast numbers come from simple pixel sampling of the captures. Each value is
the darkest pixel against the lightest pixel in a text box, using the WCAG
formula. Anti-aliasing makes thin glyphs read lighter than their true color, so
these are close estimates, not token math. The sampling script is in
`.qa/ios-contrast-r1/`.

## Evidence (check 0)

All 26 required captures are present and valid. Each is 1206x2622 from the
iPhone 18 Pro Simulator on iOS 27. Each shows the state its filename claims.
None has blank or black regions. Synthetic data is labelled on every demo
screen. The unpaired and signed-out states are real states, honestly labelled.

The captures come from commit 17dc663. The later launch-colour and comment edits
are source-only and were not captured. That is acceptable because no visible page
code changed.

## persistence

Pass.
- `PRODUCT.md` and `DESIGN.md` exist. `DESIGN.md` describes the current
  forest/mist world.
- The surface brief `ios-interface.md` is current and records its own open
  items.
- This is code-led work, so no comp-round state is expected.

## fidelity

There is no approved comp. The desktop forest/mist captures and `DESIGN.md` are
the critique reference.

- **Scenery** (static photo, flat wash, forest/mist room): match, in both
  themes. The image reads as a quiet mountain scene beneath the content.
- **Ridge mark, bare glyph in header and Settings card**: match.
- **Moss lamp play (44/36/72), moss tab, moss playing title and meter**: match.
- **No leading playing bars, no cover light, no amber**: match.
- **Neutral inverted fader pill, and vertical stacking at AX size**: match.
  AX-large shows clean vertical stacking with no truncation.
- **Opaque overlays** (taste sheet `#202A25`, Discovery options, demo banner):
  match in light. In dark this is a partial adaptation, because the grouped
  rows inside the forest sheet are stock neutral grey. See fix 5.
- **Native Liquid Glass limited to the navigation and control layer**: match in
  where it is used. In effect it is contradicted: the bottom accessory and tab
  bar do not keep content legible. See fix 1.
- **Flat empty-state plates** (`#2C3A32` at 85% dark, white at 68% light):
  match.
- **Disabled controls use a neutral surface with readable disabled ink**:
  contradicted for everything except the lamp. See fix 4.
- **TYPE**: match. Inter is ranked by weight and tabular times are present. The
  large navigation titles use Inter.
- **MATERIAL**: match. A real photograph sits under a flat wash, with no
  imitation material.
- **GROUND**: match against the named tokens (`#0E1110` / `#F4F6F4` under the
  wash). The dark Settings form drifts to neutral system grey. See fix 5.
- **Stock iOS Spotify identity**: absent. The app does not imitate Spotify.

## ceiling

Mostly reached for a development candidate. The one unused native device is
the system scroll-edge treatment under the floating bottom bars. That treatment
is what makes Liquid Glass legible over content, and it is the subject of fix 1.

## material_fixes

1. **Fix the legibility of the bottom accessory and tab bar.**
   - **Evidence:** Row text under the mini-player and tab bar is crisp and
     legible through both:
     - `home-*`: "From You ... 3:56", "Demo reason: warm textures close to your
       saved taste", "Day by Day" and "Khruangbin" sit beneath "Tides" and
       "Little Simz".
     - `error-*`: "o breathe." runs into the mini-player title.
     - `settings-*`: the pairing-token notice reads between the bars.
     - `home-ax-reducedmotion-dark`: this is the worst case. The accessory
       title "Tides / Little Simz" overprints "Elysian / Floating Points ·
       5:28". The scaled text also fills the accessory's full height.
   - **Measurement:** The sampled contrast of the artist line stays near 6.8:1.
     The failure is glyph-on-glyph collision, not low contrast, so a contrast
     ratio does not show it.
   - **Fix:** Restore a native edge treatment under the bottom bars on the Home,
     Library and Search lists and on the Settings form. Use
     `scrollEdgeEffectStyle(.hard, for: .bottom)` or whichever native setting
     leaves no legible underlying glyphs. Do not hand-roll a glass layer.
   - **AX sizes:** Cap the accessory's Dynamic Type size the way the system tab
     bar caps its own labels. The text must not fill or overflow the capsule.
   - **Acceptance:** With rows scrolled under the bars, in both themes at
     default and AX-large text, no underlying text is readable inside the
     accessory or tab-bar footprint.

2. **Make light Settings section headers and footers meet 4.5:1.**
   - **Evidence:** The system `secondaryLabel` text measures:
     - "Spotify", "Playback on this iPhone", "Private cloud" and the long
       footers in `settings-light` and `settings-demo-light`: about 3.4 to
       3.5:1 against the mist scenery.
     - The same text in dark: about 6.7:1, which passes.
   - **Fix:** Style the Form section headers and footers with
     `Palette.secondary` (`#4B524C` light). This matches desktop, where every
     hint role was lifted to at least 4.5:1.

3. **Make the full-width blocks symmetric.**
   - **Evidence:** `roomRow()` sets `trailing: 8`, but it is applied to the
     demo banner, the header (mark, title, refresh, options, lamp), the
     exploration fader, the status notices and the empty or signed-out plates.
     These blocks sit 16 pt from the left edge but 8 pt from the right. The
     lamp disc and fader capsule nearly touch the right screen edge on every
     Home, Library and Search capture.
   - **Fix:** Keep the 8 pt trailing inset only for track and playlist rows,
     where the 44 pt feedback targets provide the visual inset. Give full-width
     blocks 16/16 insets.
   - **Related:** In light, the demo banner row also shows stray list
     separators above and below it (`home-light`, `library-light`,
     `search-light`). Hide the separators on the banner and header rows.

4. **Give demo-disabled controls readable disabled ink.**
   - **Evidence:** Only the lamp uses `Palette.disabled`, which measures
     4.3:1 in dark and 5.4:1 in light. The other disabled controls fall back
     to system dimming:
     - "Write your taste", the onboarding's only call to action: about 2.2 to
       2.3:1 in both themes.
     - "Sign in to Spotify" in `settings-demo-dark`: about 2.3:1.
     - Love/Less glyphs in light: about 2.7:1.
     - "Save this mix": about 3.4:1.
   - **Why it matters:** Demo mode disables most of these controls, so these
     ghosted states dominate the demo screens. It is the same neutral-disabled
     rule the desktop review enforced.
   - **Fix:** Drive the disabled foreground of buttons from `Palette.disabled`
     through `isEnabled`. Keep the shape and surface neutral.

5. **Unify the dark grouped surfaces with the forest tokens.**
   - **Evidence:** In dark, the rows of the Settings form and the identity card
     render as stock iOS `#1C1C1E`. The Discovery options sheet shows a neutral
     grey `#2C2C2E` inset list inside its forest `#202A25` sheet. Next to the
     forest plates and banner, these read as stock iOS dark, not forest.
   - **Fix:** Set the dark row background to the forest surface tokens
     (`Palette.surface` or `surfaceActive`, matching the empty-state plate).
     Light already uses white and passes.
   - **Related, minor:** The options sheet has about 45 pt of dead space
     between its title and the first row, in both themes. Tighten it while
     touching the sheet.

## keep

Do not dilute any of these while fixing:
- The static photo and flat wash.
- The bare ridge glyph.
- The single moss voice: lamp, current tab and playing title.
- The neutral inverted fader pill and its vertical AX stack.
- The opaque forest taste sheet.
- The Now Playing sheet over the room.
- The native system tab bar and search.

## Unverified

The captures prove static appearance only. They do not prove the following:
- **Motion:** the fader glide, the tab-bar minimize on scroll, and how sheets
  present and dismiss.
- **Reduce Motion:** only static captures exist.
- **VoiceOver:** no walkthrough has been done.
- **Enabled feedback buttons:** every capture with Love/Less is a demo, so
  these controls are always disabled. Their enabled `Palette.dim` ink is
  unmeasured in pixels.
- **Other display modes:** Increase Contrast, Reduce Transparency (how the
  accessory looks without glass), landscape and iPad.
- **Live states:** real signed-in or paired data and audio.
- **Edge-effect fix:** rows scrolled under the bars should be recaptured after
  fix 1. The current captures only show that the problem exists.
