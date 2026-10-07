# October 7 review follow-ups

The review of `47fd38e` found three remaining defects. This follow-up fixes:

- The macOS **Spotiurge on GitHub** Help action now opens
  `SergeSerb2/Spotiurge`. Its label had already changed, but the dispatched URL
  still pointed upstream.
- Exact discovery matching normalizes title and artist text to NFC after
  presentation folding. Canonically equivalent accents and Hangul match;
  missing accents, unknown credits and different versions stay distinct.
  It reuses ICU's normalizer, already compiled for URL/IDNA, without enabling
  extra default features or adding new registry packages.
- Private-cloud startup accepts the same 32–256 ASCII token contract as the
  desktop. Invalid configuration fails before opening storage or a listener.
  Existing valid pairing tokens do not change.

Local checks passed: formatting, default/all-feature Clippy, default/all-target
tests (988 library tests), all-feature/all-target tests (1,011 library tests),
documentation tests, warning-denying Rustdoc and 15 private-cloud tests.
The Unicode regression covers both directions, collaborator credits and
meaningful differences. The startup regression covers both accepted bounds
and rejected short, long and non-ASCII tokens without real credentials.

The Cargo lock changes only the root dependency list. The new vendor staging
hash is `sha256-X06mrQtHxEJs46ltpcK8/jIDm6plA/f/zxtwkYecBn4=`. It was computed
from verified cached crate tarballs and exact pinned Git trees, including
submodules, with NAR serialization. The same procedure first reproduced the
old hash `sha256-2thLwV0G3+DoKeZF7+xP7v8jgzeTpkamDYey/QBj0E0=` exactly.
No Nix executable is available locally; a full Nix package build still needs
the exact-head CI check.

These three fixes do not alter the photographed desktop appearance. The
existing whole-app comparison remains valid. The universal Mac r3 development
DMG was built from `b96095d`, signed with the fork's Apple Development identity,
and checked after mounting. Its Drive download was hashed against the local
artifact, and the download link, existing pairing token and pairing steps
were emailed to Serge. It is not notarized or Gatekeeper accepted.

The DMG SHA-256 is
`423b2dcd9042ed740b920865fb5f7540c9e844cf463fd3e2e60f9b9900deceaf`.
It contains both arm64 and x86_64 slices. This package did not repeat live
playback, catalogue, AI or cross-device checks from earlier integration
evidence.


## Discovery load errors

A subsequent review found that Home kept showing **Opening your personal
discovery workspace…** after a failed state-file load. It now displays the
stored failure with an alert icon and wraps the complete recovery advice in
narrow windows. The disabled refresh control also reports the load failure.
Discovery stays unavailable until the state can be loaded, so drawing the
error cannot replace the original file.

The regression feeds real corrupt, oversized and unreadable file errors
through `DiscoveryLoaded` into the native view, checks the accessible text
and disabled refresh in both themes and at two content widths, confirms
the fixture files are preserved, then checks a successful load clears the
error and enables refresh. Pending loading retains its existing notice.

[Before/after native captures](load-errors/index.html) show the same synthetic
load failure in light/dark themes at 1440×900 and 900×760 logical window sizes.
The baseline is `b96095d` plus only the new deterministic demo fixture; the
candidate adds this rendering fix and regression. The separate manifests
record binary, source and image hashes. No Spotify, AI or private-cloud
requests ran for these captures. The r3 DMG above predates this final
load-error rendering fix.

Local follow-up checks passed: launcher/package regressions, formatting,
default/all-feature warning-denying Clippy, all-target tests (989 default and
1,012 all-feature library passes, two ignored), doctests, warning-denying
Rustdoc and Jekyll. The first gettext check flagged changed source locations;
regenerating the catalogs and rechecking passed. Message IDs and translations
are unchanged; only locations and the generated POT timestamp changed.
