# iOS scenery verification

This is a native SwiftUI development interface, not a production iOS architecture
or TestFlight release. The production playback gate remains partial. Serge asked
to stop physical iPhone operations, and this work used only a Simulator and an
unsigned generic device build. No physical device was installed, launched or read.

## Appearance and evidence

The same bundled mountain photo, flat ridge mark, Inter, forest/mist palette and
quiet content surfaces as desktop replace the earlier amber/cover-light scheme.
Native Liquid Glass stays in navigation and controls. Hard bottom scroll-edge
treatment blurs list text beneath the floating mini-player and tab bar. The
compact accessory caps Dynamic Type at XXX Large. Full-width blocks have equal
16-point margins; track rows retain their 44-point feedback targets. Settings
section hints and disabled controls have explicit readable ink. Dark grouped
rows use forest surfaces. The disabled glass CTA becomes a neutral opaque pill.

[Before/after viewer](index.html) contains 26 paired screens in both schemes,
including Home, Now Playing, Library, Search, Settings, notices, first-run taste,
options, signed-out and unpaired states. Dark Home and Now Playing also have
accessibility-large/Reduce Motion captures. Originals were 1206x2622 PNGs from
agent-device on iPhone 18 Pro Simulator, iOS 27.0. Committed derivatives are
603x1311 PNGs resized with sips. No artistic edits were made. Exact image and
build-input hashes are in [captures.json](captures.json).

Historical baseline images are the original amber candidate's committed
captures, with its input manifest preserved. Demo states match. Some real-state
statuses differ because that baseline was unsigned and engine-linked, whereas
this candidate is ad-hoc signed and omits the engine. Those comparisons are
explicitly marked rather than described as matching states.

## Actually checked

- Platform-neutral Swift core: **40 tests passed**, including merge/CAS, bounded
  feedback, pending offline ratings, matching, limits and Python server wire
  compatibility. Uses local dummy data and protocol stubs.
- Hosted model lifecycle suite: **14 tests passed** in the Simulator, including
  a native Keychain round trip with a disposable test grant. The host is ad-hoc
  signed; unsigned hosts lacked the entitlement and that failed attempt is not
  counted as a pass. Tests were retained unchanged.
- Simulator app build: **passed**, arm64, `--no-engine`, ad-hoc signing.
- Generic iPhoneOS build: **passed**, `--no-engine`, `CODE_SIGNING_ALLOWED=NO`.
  This checks compilation only and does not install on a phone.
- 26 native captures inspected: 23 reviewed r3 frames and three fresh targeted
  corrections. All 52 before/after derivatives have their declared dimensions
  and hashes.
- Prior desktop contract and final iOS design review remain separate from these
  functional checks. The review is design-only, not a security or playback audit.

The local evidence logs live under ignored `target/ios-*` directories. No secrets
are included in committed evidence.

The two final Opus findings were closed by root with targeted captures and pixel
sampling, as recorded in [root-final-confirmation.md](root-final-confirmation.md).
This does not replace the production playback gate.

## Not demonstrated

No live Spotify sign-in/library/catalogue matching, live Luna response or
cross-device sync was exercised in this app. Network requests were disabled in
labelled demo captures; real captures used a signed-out, unpaired Simulator.
The current captures omit the Rust audio engine, so no iOS audio is claimed.
The experimental playback probe's locked run is separate evidence and its local
identity patch is not shipped here.

Motion quality, VoiceOver, Reduce Transparency, Increase Contrast, landscape and
iPad were not measured. Static Reduce Motion captures prove layout only.
No TestFlight upload, App Store Connect record or production provisioning has
been created for this development app. Use the playback gate record and signing
preparation notes before selecting a production architecture.

## Post-capture functional fixes

The desktop's final review fixes were also ported here: manual requests preserve
AI cooldowns, repeated mix saves reuse bounded slots, and Spotify sign-in waits
for a complete bounded callback line. Core and hosted suites were rerun, including
a scripted 429/manual-repeat regression. The unsigned generic device build also
passed with this code. The visual design is unchanged; the capture manifest
continues to identify the exact earlier builds used for those images.

The later restart and catalogue review fixes also preserve appearance. Local
attempt times, pending refreshes, pairing suspension and failure cooldowns now
survive reopening; the serial writer finishes before AI access. Catalogue
matching uses full case folding followed by NFC while preserving accents and
versions. All 42 Swift core tests and 15 hosted iOS Simulator tests passed,
including a saved-429/reopen regression and a disposable native Keychain round
trip. The Simulator test build and unsigned generic iPhoneOS build passed with
the engine omitted. No physical phone, live AI/catalogue/cloud round or
TestFlight upload was tested in this round.
