# Spotiurge agent guide

Spotiurge combines Spotify with Serge/Surge. It is Serge's personal,
standalone fork of [Spotifast](https://github.com/crmne/spotifast).

This file defines the fork's product and workflow policy. Use
`CONTRIBUTING.md` for build instructions, checks, and visual-review procedures.
Where inherited Spotifast policy conflicts with this guide, follow this guide.

## Fork identity

- Work belongs in `SergeSerb2/Spotiurge`. Never submit this fork's changes or
  pull requests back to Spotifast, or merge Spotiurge back upstream.
- Maintain an independent product direction, branding, and release history.
  Upstream changes may be selectively adopted when they benefit Spotiurge;
  compatibility with upstream's product roadmap is not a goal.
- Preserve the MIT license, upstream attribution, and existing copyright
  notices. Rename user-facing identity only within the requested scope.
- Treat inherited Spotifast URLs, update channels, package destinations, and
  release automation as upstream configuration. Before shipping a fork release,
  ensure they target Spotiurge. Never publish to upstream's release channels,
  website, Homebrew tap, or AUR packages.

## Product direction

- Keep the foundation lightweight, native, and fast. Startup time, memory use,
  idle work, and responsive playback remain product features.
- Make the app personal to Serge, with a much prettier liquid/frosted glass
  interface, thoughtful typography, and polished motion.
- Build advanced personalized music discovery, AI recommendations, and AI DJs.
  Aim to match or exceed Spotify's recommendation quality for Serge. This is a
  goal, not a claim about current capabilities; evaluate it with real listening
  feedback before claiming success.
- Assume one user. More expensive models, richer ranking, and deeper
  personalization are acceptable when they improve Serge's experience. Do not
  add multi-user infrastructure or optimize for mass-market scale by default.
- Keep expensive recommendation and AI work off the UI and playback threads.
  Cache useful results, respect service rate limits, and keep playback usable
  when an AI service is slow or unavailable.
- These are directions for future work. Implement only the requested task;
  do not turn a small change into a redesign, AI integration, or general refactor.
  Preserve existing behavior unless the task explicitly changes it.

## Integration boundaries

- Spotify music comes from Spotify through supported Spotify/librespot
  capabilities. Do not substitute another audio catalogue, bypass DRM, or
  advertise unsupported playback features such as lossless.
- Do not embed a browser engine or add telemetry.
- Configured external AI services are allowed for recommendations and AI DJ
  features. Document new network access and what personal data is sent. Keep
  credentials protected and never log secrets or authorization responses.
- Read `docs/_reference/what-spotify-allows.md` before building or promising a
  Spotify-facing feature. Verify actual support rather than inferring it from
  a protobuf or enum. Build Spotiurge's own recommendation layer using supported
  metadata and playback; unavailable Spotify endpoints are not a dependency
  we can assume away.
- Read `docs/_reference/how-it-connects.md` before changing authentication,
  Spotify requests, Connect, credential storage, or network behavior.

## Architecture and behavior

- `src/ui/` draws views and emits `Action`s. Apply actions after drawing in
  `src/app.rs`; do not mutate application state from inside a borrowed view.
- Network and playback work belongs on the runtime in `src/backend.rs` or in
  the engine in `src/player.rs`, never as blocking work on the UI thread.
- Keep platform integrations behind target-specific modules or `cfg` blocks.
  Preserve Linux, macOS, and Windows compilation when changing one platform.
- Keep settings and state readable, backward compatible, and atomically written.
- Read `docs/_reference/queue.md` before changing queue behavior, and read nearby
  tests before changing a state machine or API fallback.
- Preserve optimistic interactions. Playback, queue edits, and library changes
  appear immediately; stale backend responses must not undo or flicker away
  the user's action.
- All visualizers show the signal after EQ and before volume. Volume changes,
  including zero volume, must not change the picture.
- Prefer existing dependencies and explain new crates in `Cargo.toml` when
  their purpose is not obvious. Fix dependencies in a maintainer-owned fork
  pinned to a commit; do not copy dependency source into this repository.
  Contribute generic dependency fixes to their own upstream projects and use the
  pinned fork until a release includes the fix. The prohibition on merging
  Spotiurge upstream still applies to Spotifast.
- Keep the shared egui/winit fork crates on matching revisions
  (`crmne/egui` apps-0.36, `crmne/winit` apps-0.30). Preserve compositor-paced
  Wayland rendering; do not add a separate vsync decision.
- Pass logical text to `crate::bidi` after layout. Never reorder strings before
  layout; the egui fork already shapes right-to-left runs in their own direction.

## Interface changes

- Glass effects must preserve readability, contrast, clear controls, and smooth
  interaction. Consider light/dark themes and reduced motion when applicable.
- Describe visible changes explicitly. Use deterministic `demo` captures and
  the HTML comparison format in `CONTRIBUTING.md`, with matching before/after
  states at representative sizes in light and dark themes.
- A redesign needs Serge's explicit approval of its visual scope before merging.
  A requested visual adjustment authorizes that adjustment. Preserve approval
  across rebases that do not change the approved appearance or interaction.

## Working and verification

- Use focused branches and pull requests against this fork's default branch.
  Specify `SergeSerb2/Spotiurge` explicitly when opening a PR from this fork.
  Keep one topic per change and squash-merge to preserve linear history.
- Fetch the fork's origin and rebase onto its default branch before opening a
  PR. Use fast-forward-only pulls. Do not push merge commits or rewrite published
  history without explicit permission and an exact force-with-lease guard.
- Add focused regression tests for changed behavior. Update README/docs when
  user-visible behavior, settings, files, or network access changes.
- Run the full checks in `CONTRIBUTING.md`. Do not weaken lints or remove tests
  to get a green result. Report unavailable checks and platform coverage honestly.
- Builds use mise/mbx where available; run `mise trust` once per new checkout.
  Each worktree uses its own `target/`; a second concurrent build uses
  `CARGO_TARGET_DIR=target/<name>`. Never use a shared target directory or vary
  compiler flags per agent. Use `mbx gc`, not `cargo clean`, for disk recovery.
- Keep build output and large scratch files out of `/tmp`. Remove one-off QA
  and packaging directories once their results are recorded.
- Keep public issue and PR replies short, direct, and useful. Do not post two
  maintainer comments in a row; edit the last one if nobody has replied.
  Never use em dashes.

## Releases

Verify Spotiurge's release and package destinations first. Follow the build and
native package checks in `CONTRIBUTING.md` and `PACKAGING.md`, in this order:

1. Before tagging, change the `Cargo.toml` version, add the matching release to
   the Flatpak metainfo (`packaging/flatpak/rocks.spotifast.Spotifast.metainfo.xml`
   until renamed), and update `Cargo.lock` with a build. Refresh the `flake.nix`
   vendor hash whenever the lockfile changes, including version-only changes,
   and verify `nix build .#default` locally or in CI. Commit written notes at
   `packaging/release-notes/vVERSION.md`, push the release commit, and wait for
   every required CI job before tagging.
2. Push the `v*` tag. Wait for every required artifact and `checksums.txt`, then
   verify the published written notes, screenshot, and download links. Never
   publish generated placeholder notes or links to files that do not exist.
3. A prerelease stops here. Keep the website and package channels on the latest
   stable release; the prerelease remains on Spotiurge's GitHub releases page.
4. For a stable release, only after the GitHub release exists, update the fork's
   `docs/_config.yml` version key (currently `spotifast_version`) and
   `docs/_data/versions.yml`. Replace the selector's old version entry with the
   latest stable version, make it `current`, and point it at `/download/`.
   Keep only the latest stable version entry; older releases use the Changelog
   link, which must target Spotiurge's releases.
5. Update any configured Spotiurge Homebrew/AUR channels from the release's
   `checksums.txt`, following `PACKAGING.md` and native package validation.
   Unconfigured channels need no publication. Never publish to upstream channels.

Match the previous two stable releases when writing notes: a short summary,
`New`/`Fixed` sections with bold user-facing results and contributor/report credits,
a `Thanks` section, and a full changelog link targeting Spotiurge.
