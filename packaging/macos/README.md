# Spotiurge development packaging

The fork's local scripts do not use inherited Spotifast release destinations.
They preserve existing outputs and never include state or credentials.

Build the release without the optional MilkDrop dependency, separately for
`aarch64-apple-darwin` and `x86_64-apple-darwin`, using `cargo build --locked
--release --no-default-features --target TARGET --bin spotifast`. Install the
second Rust target with `rustup target add x86_64-apple-darwin` when needed.
Combine the binaries and package locally:

```sh
mkdir -p dist
lipo -create target/aarch64-apple-darwin/release/spotifast target/x86_64-apple-darwin/release/spotifast -output dist/Spotiurge-universal
python3 packaging/macos/development-dmg.py dist/Spotiurge-universal dist/Spotiurge-development-universal.dmg --identity 'Apple Development: YOUR NAME (CERTIFICATE ID)'
```

The development bundle uses `com.sergeserbinenko.spotiurge`, enables the hardened
runtime, preserves upstream copyright, and does not register a Spotify URL
handler. The DMG contains only the app, an Applications shortcut, license and
installation notes. Scratch packaging files stay beside the output and are
removed automatically. Verify both architectures, linked libraries, signature,
mounted contents and actual app launch before delivery.

An Apple Development signature is **not** a notarized distribution build.
Gatekeeper can block it on another Mac. A production package needs an installed
Developer ID Application certificate, secure notarytool credentials, Apple's
successful notarization, a stapled ticket and Gatekeeper validation. Do not
disable Gatekeeper or remove quarantine to claim that this gate passed. See
[Apple's notarization troubleshooting](https://developer.apple.com/documentation/security/resolving-common-notarization-issues).

Sign in to Spotify separately on each laptop. The cloud pairing token must be
provided separately through a protected channel and stored in the native
credential store, following the [private cloud setup](../../services/private-cloud/README.md).
Do not put it in the DMG, source tree, or command arguments.
