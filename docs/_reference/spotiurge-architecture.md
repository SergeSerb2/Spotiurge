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
have build/test CI. At the audit baseline, no iOS project, Rust bridge or
TestFlight pipeline existed. An isolated Rust/Swift playback probe now exists;
the production iOS application remains gated.
Reuse these boundaries and dependencies rather than replacing the desktop.

The updater repository targets `SergeSerb2/Spotiurge`, but inherited packaging
identities, website, package links and release metadata still use upstream names.
Automatic checks, manual checks and the startup update helper are disabled. Audit
and rename every artifact and installer destination before enabling fork updates.
Do not tag or publish this development slice as a release.

Serge resumed iOS work on October 6, 2026 after connecting his real iPhone.
Desktop delivery continues while an isolated iPhone playback probe establishes
whether the independent-player requirement can be met. The production iOS
architecture remains gated on that proof.

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

Saved mixes have an explicit removal action with a durable tombstone, so a stale
replica cannot restore a removed mix. New saves reuse a removed mix slot first,
or the oldest slot once 100 exist. Imported legacy slots remain readable and
removable; this does not destructively rewrite their keys. A manual AI refresh
observes the same retry deadline as automatic refreshes. Only a pairing error
can be explicitly rearmed before a new attempt.

Feedback storage keeps at most 500 URI records, counting cleared ratings. The
version-one `feedback:retention` tombstone is a shared logical cutoff. Merges
remove feedback at or below that stamp, including records from an old offline
copy. Equal-clock cohorts are forgotten together. New ratings use a clock above
the cutoff; an acknowledged sync re-expresses feedback changed after its snapshot
above the imported clock, preserving immediate local intent. The cutoff itself
is never re-stamped as a user edit. Offline feedback older than the cutoff is
forgotten, even if a device's wall clock is later. Mixes, taste and AI history
are unaffected. All devices must upgrade to enforce this retention policy.
A bounded local `pending_edits` stamp journal distinguishes unsent record edits
from acknowledged replica data. It covers taste, feedback, mix/history edits
and tombstones, and reads older `pending_feedback` entries. On sync it imports
the remote clock and re-expresses pending intent that a higher record or feedback
cutoff would otherwise replace. Successful acknowledgments clear only dispatched
stamps; edits made during sync remain pending. An import without a dispatched
snapshot acknowledges nothing. The journal is atomically saved with local state
and never enters the cloud document or AI prompts.

Recommendation generations are invalidated by saved taste, feedback, exploration
changes or imported taste/feedback. The superseded request retains its worker
slot until completion, then its content and input-specific errors are discarded
without replacing cached picks, refresh time or AI history. Request-wide busy,
rate-limit and pairing failures still install and persist the retry deadline or
automatic suspension. Duplicate old completions cannot throttle newer work.
An old completion cannot
release a newer request's slot. Unfinished taste-editor drafts do not change the
saved recommendation inputs.

AI history is bounded to ten live entries, the newest ten the UI shows. Each
refresh adds a `history:` key only while fewer than ten exist; after that it
overwrites the oldest history key, live or tombstone, with a newer clock. The
same atomic write tombstones live entries beyond the limit, so a legacy document
with hundreds of entries shrinks on its next refresh without new keys. Taste,
feedback, mixes and their clocks are untouched, and validation or clock
exhaustion leaves the document unchanged. Two devices refreshing offline can
both add keys or overwrite the same oldest key; the higher clock wins and the
other device's entry is lost, which is acceptable for history. The latest
catalogue picks are saved locally even when a full document rejects the history
entry; the existing storage-full message is shown instead.

The desktop window's eframe state is `app.ron` in Spotiurge's own state
directory, beside its session and discovery files. Upstream Spotifast's
`app.ron` is neither read nor imported; demo captures keep their separate,
unsaved path.

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

CoreDevice now reports Serge's physical iPhone 17 Pro Max as connected over USB,
paired and booted, with Developer Mode enabled. It runs iOS 27.2; the Mac has
Xcode 27.0. CoreDevice successfully installed and launched the signed probe;
the older Instruments offline listing did not describe actual readiness.
An Apple Development identity is available for team `78A5P57U23`
(`336W29P997` is the certificate label, not the team ID). Existing development
profiles for other apps cannot be reused for Spotiurge. Xcode created a
team-managed development profile covering this phone and the probe's own
`com.sergeserbinenko.spotiurge.playbackprobe` identifier. A Spotiurge App Store
Connect record, iOS distribution identity and upload authorization remain
unverified.

Build 8 passed the locked-background criterion: 17 minutes 22 seconds locked,
four natural track transitions, and Spotify absent from 18 process snapshots.
One server session closure recovered automatically, restoring the queue and
position with 1.36 seconds of silent rendered frames. This proves independent
audio through the Rust decoder and AVAudioSourceNode on the tested phone.
Build 9 corrected a duplicate-session bug and connected from Keychain after
installation. A deliberate interruption/resume, route change and Connect
handoff both ways remain uninstrumented. Serge reported informally that it was
all good, then requested an end to phone checks. Physical-device operations
have stopped. See the [redacted playback evidence](../reviews/spotiurge-ios/playback-gate.md)
for the precise timeline and partial gate verdict. UI development can proceed
in source and Simulator against the desktop design, while production playback
and TestFlight remain gated. The probe uses an ignored local
librespot identity experiment, not a shipping dependency or proof of official
Spotify iOS SDK support. A validated dependency fix must move to a
maintainer-owned pinned fork and be contributed to librespot upstream.

The gate requires a minimal independently signed probe with an iOS-capable audio
sink, AVAudioSession playback category and `UIBackgroundModes=audio`. No
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
