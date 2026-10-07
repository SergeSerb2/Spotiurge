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

These changes do not alter the photographed desktop appearance. The existing
visual comparison remains valid. A new universal Mac development package is
being built with these fixes; its delivery and signing evidence will be
recorded separately after validation.
