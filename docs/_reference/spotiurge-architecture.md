---
title: Spotiurge architecture and delivery
description: Fork audit, incremental milestones and the real-iPhone playback gate.
nav_order: 20
---

# Audit and plan

Baseline: `SergeSerb2/Spotiurge` main, `e46f894`, fetched October 6, 2026.
`AGENTS.md` overrides inherited upstream product policy. `CONTRIBUTING.md`
defines checks and matched deterministic visual comparisons.

The desktop foundation is Rust 1.98, egui/eframe with OpenGL, a Tokio backend,
librespot playback/Connect and native credential stores. Network and playback
are already separated from immediate-mode drawing. Linux, macOS and Windows
have build/test CI; no iOS project, Rust bridge or TestFlight pipeline exists.
Reuse these boundaries and dependencies rather than replacing the desktop.

The updater repository targets `SergeSerb2/Spotiurge`, but inherited packaging
identities, website, package links and release metadata still use upstream names.
Automatic checks, manual checks and the startup update helper are disabled. Audit
and rename every artifact and installer destination before enabling fork updates.
Do not tag or publish this development slice as a release.

Serge deferred iOS work on October 6, 2026 because he is remote. The current
implementation and acceptance focus on macOS; iOS remains a later gated milestone.

## Milestones and acceptance

1. **Desktop discovery slice:** personal taste prompt, bounded recommendation
   requests through existing CLIProxyAPI subscriptions, catalogue validation,
   playable results and explicit listening feedback. Catalogue matching has one
   twenty-second deadline; slow searches retain unchecked suggestions. Strict
   matching parses actual credited collaborators and preserves recording versions.
   A catalogue-only retry preserves matched tracks and never asks AI again. Cache prior results and
   leave the existing player and Connect controls responsive during failures.
2. **Private synchronization:** a single authenticated Railway service with a
   persistent SQLite volume. Sync preferences, feedback, custom mix references
   and AI history. Prove two real desktop devices exchange an edit, including
   a disconnected edit and a concurrent conflict. Local protocol tests alone
   do not satisfy this milestone.
3. **iPhone playback gate:** prove independent playback before deciding the
   production iOS architecture. Keep the SDK controller separate from this test.
4. **iOS first slice and TestFlight:** after the gate, select native UI and a
   narrow Rust bridge for suitable shared domain/network code. Verify the same
   discovery, playback and sync workflow on the real phone. Prepare the signed
   archive, upload, processing and internal tester assignment.
5. **Recommendation evaluation, then spoken DJ:** collect intentional feedback,
   compare discovery/saves/skips against a fixed Spotify baseline over listening
   sessions, and improve ranking. Speech comes afterward, with its own supported
   playback and audio-session investigation. No quality-parity claim yet.

## AI boundary

Recommendations use GPT-6 Luna only through Serge's existing CLIProxyAPI subscriptions, with no model fallback. The
cloud service owns the proxy credential; devices never receive that credential
or upstream subscription grants. No direct-model-provider fallback.

Home automatically refreshes an empty or twelve-hour-old cache from saved taste
and/or intentional feedback. Exploration changes debounce for 1.5 seconds;
feedback debounces for 45 seconds and refreshes at most every ten minutes. AI
failures back off from ten minutes to six hours; pairing failures suspend automatic
requests until an explicit recovery action. Nothing runs automatically in demo
mode, without inputs, while busy or signed out. The scheduler requests a future
repaint instead of polling. Network outages keep prior picks and the local player
usable. Local settings hold exploration, automatic refresh and cache age; synced
taste and feedback remain in the backward-compatible version-one document.

[Spotify's policy](https://developer.spotify.com/policy) prohibits ingestion of
Spotify content into AI models. Serge reports permission for broader use in this
thread. That assertion is not independent verification of the permission's
scope. Document actual inputs with the implementation, and do not expand them
to audio, credentials or account identifiers. The first request can operate
from self-authored taste plus intentional recommendation feedback. Resolve
model-suggested artist/title pairs through supported catalogue search; a model
cannot invent playable URIs or override market availability. Do not depend on
the restricted Spotify recommendations, related-artists or audio-features APIs.

## Sync and offline behavior

Keep a device-local, atomically replaced JSON snapshot and a random writer
ID. Secrets and the local cloud endpoint are excluded from the synced document.
Rotate the writer ID on every profile load, preserving historical stamps and
advancing counters from the existing document. This makes restored or copied
profiles independent writers without a portable secret or hardware identifier.
Use per-record Lamport counters with writer-ID tie breaking, not wall
clocks. Unrelated records merge; concurrent edits of the same record choose the
larger `(counter, writer ID)` pair. Preserve deletions as tombstones so
offline devices cannot resurrect deleted mixes. A playlist/mix is one record:
simultaneous edits choose one complete version, rather than interleaving songs.

The service exposes a revisioned document. A client fetches, merges and writes
with compare-and-swap; a conflict refetches and retries within a bounded deadline.
The UI keeps the dispatched snapshot. Records edited after it are preserved
over an acknowledgment with a higher remote clock, including tombstones. If
necessary they are re-stamped above that clock and remain pending for the next
sync. Validation or clock exhaustion leaves the original local document intact.
Persist locally before cloud writes. Failures keep the local snapshot,
cached recommendations and playback usable. Offline metadata access is allowed;
offline Spotify audio downloads are not a supported feature.

Recommendation generations are invalidated by saved taste, feedback, exploration
changes or imported taste/feedback. The superseded request retains its worker
slot until completion, then its results and errors are discarded without
replacing cached picks, refresh time or AI history. An old completion cannot
release a newer request's slot. Unfinished taste-editor drafts do not change the
saved recommendation inputs.

Single-user bearer authentication over HTTPS is sufficient initially. Store the
device token in Keychain/Credential Manager/Secret Service, with a short native
store deadline. Server secrets belong in Railway variables. Disable redirects
for authenticated requests, bound payloads and request duration, and log neither
request bodies nor authorization headers. Token rotation requires re-pairing
devices. The service does not store Spotify grants or audio. Backup/export,
deletion and a retained-data limit are required before routine use.

## iOS architecture gate and signing preparation

[Spotify's official iOS SDK](https://developer.spotify.com/documentation/ios)
controls playback in the Spotify app. It is not an independent audio engine.
The Web API is a controller too. Embedding the Web Playback SDK would violate
this fork's no-browser boundary and does not establish independent background
playback. libspotify is not a maintained option. The existing librespot engine
is a candidate for a separate feasibility probe, not evidence of iOS support
or permission to distribute it through TestFlight.

The local device inventory currently reports Serge's physical iPhone as
`unavailable`; the connected device is a simulator. The installed signing
identity is Apple Development; the certificate's organizational unit identifies
team `78A5P57U23` (`336W29P997` is the certificate label, not the team ID).
This is not proof of an App
Store Connect app record, distribution profile or upload authorization.

The gate requires a minimal independently signed probe with an iOS-capable audio
sink, AVAudioSession playback category and background-audio entitlement. No
production UI/framework choice is made by that probe. On a real Premium account:
start Spotify music with the Spotify app terminated, verify Spotiurge/probe owns
the audio session, lock the phone for at least ten minutes and multiple track
transitions, test interruption/resume and route changes, and test Connect handoff
both ways. Record exact build/device/OS, audible evidence, session ownership and
logs without grants. A simulator, SDK controller or ten minutes of Spotify-owned
audio is not a pass. A failed probe is reported as a blocker, not replaced by a
controller.

After the gate: reserve a fork-owned bundle ID such as
`com.sergeserbinenko.spotiurge` under the confirmed team; create the App Store
Connect record and explicit App ID; configure Keychain access and only justified
background modes; select distribution signing/profile and protected App Store
Connect upload credentials. Archive for generic iOS, validate/export for App
Store Connect, upload, wait for processing/export compliance, and assign Serge's
internal TestFlight group. Verify installation and background playback on that
exact build. Follow [Apple's distribution guide](https://developer.apple.com/documentation/xcode/distributing-your-app-for-beta-testing-and-releases).
