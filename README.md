# Spotiurge

Serge's standalone personal fork of [Spotifast](https://github.com/crmne/spotifast),
written in Rust with native egui rendering, librespot playback and Spotify
Connect. Desktop supports macOS, Windows and Linux. iOS and TestFlight remain
gated on independent background playback on a real iPhone.

**Playback needs Spotify Premium.** Free accounts can browse and search, but
cannot play music through Spotiurge.

https://github.com/user-attachments/assets/a5f669ce-b3b7-4f8e-9933-976a78876c7e

![Spotifast Home with the playlist library, recommendations, queue, and player visible](docs/screenshot.png)

The inherited [Spotifast guide](https://spotifast.rocks/) explains the existing
desktop controls. It describes upstream releases, not Spotiurge downloads:

- [Getting started](https://spotifast.rocks/getting-started/): sign-in, playback on this computer, themes, fonts, proxies
- [Everyday use](https://spotifast.rocks/using-spotifast/): keyboard shortcuts, command-line control, updates
- [Settings and files](https://spotifast.rocks/settings-and-files/) and [Privacy](https://spotifast.rocks/privacy/)
- [How it connects](https://spotifast.rocks/how-it-connects/) and [What Spotify allows](https://spotifast.rocks/what-spotify-allows/)
- [Will my account get banned?](https://spotifast.rocks/what-is-spotifast/#will-my-spotify-account-get-banned)

## Install

No fork release has been published. Build this checkout with
`cargo build --locked`. The internal crate and executable still use `spotifast`;
launch `target/debug/spotifast`. Use `--no-default-features` to omit MilkDrop.
Updates target `SergeSerb2/Spotiurge`, never upstream. State and credentials are
isolated from Spotifast; sign in to Spotify again in Spotiurge. For a local Mac
bundle, run `python3 packaging/macos/dev-bundle.py target/debug/spotifast target/Spotiurge.app`.
Use `--identity` with an installed Apple Development identity for stable native
credential access. This creates a development bundle and does not publish,
notarize or register itself as the default Spotify link handler.
See [development DMG packaging](packaging/macos/README.md) for a universal Mac
preview and its signing/distribution limitations.

To authorize remotely, start Spotify sign-in in the Mac app, then run
`target/debug/spotifast sign-in-url` to retrieve the current authorization
request. After approval on another device, its localhost return URL must be
forwarded securely to this Mac before the ten-minute timeout. That URL contains
a temporary code: never post it publicly or include it in logs. The native
listener still validates state and PKCE; Spotiurge does not use a public OAuth
relay or store authorization callbacks. The fork has its own instance/control
namespace and Connect device name.

## Personal discovery

Home opens with a taste prompt, AI discoveries, intentional feedback and saved
Spotiurge mixes. **Save taste** works locally. **Find music for me** requests
recommendations through the private cloud's existing CLIProxyAPI subscriptions,
then validates title/artist pairs with Spotify catalogue search. Matching has a
shared twenty-second deadline; unverified suggestions stay visible and cached. Only matched
Spotify tracks can play, using the existing local engine or Connect device.
**More like this** and **Less like this** affect the next recommendation request.
**Save this mix** keeps an ordered Spotiurge mix without creating a Spotify
playlist. **Sync my devices** exchanges taste preferences, feedback, mixes and AI
history. Previous discoveries remain cached when AI is unavailable.

Pair each desktop once by launching with `SPOTIURGE_CLOUD_TOKEN` in its process
environment. Obtain it from the private Railway service through a protected
channel; never paste it into a shell command or commit it. The app stores it in
Keychain, Credential Manager or Secret Service, bound to the HTTPS origin.
`SPOTIURGE_CLOUD_URL` overrides the configured private service origin. The
CLIProxyAPI credential stays server-side. There are no direct-provider fallbacks.

AI receives the taste text and up to 100 intentional feedback records containing
track title, artist and rating. The service strips Spotify URIs. Account IDs,
Spotify grants, raw listening history, artwork and audio are not sent to AI.
The private store receives the synchronized document, including mix URIs and
AI suggestions; it receives no Spotify grants, audio or device pairing token in
the document. Networking follows the desktop proxy policy. Authenticated cloud
requests do not follow redirects. All integrations are bounded and off the UI
and playback threads.

Offline edits remain local until a manual sync. Same-record conflicts use a
logical counter and installation ID; separate records merge. Offline music
downloads are not supported. See the [architecture and delivery plan](docs/_reference/spotiurge-architecture.md)
and [private service operations](services/private-cloud/README.md) for limits,
credential rotation, export and the iOS playback proof criteria.

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening an issue or pull
request. To look at the interface without a Spotify account, run
`cargo run --features demo -- --demo`. Translations live in `assets/i18n/`;
see [Translating Spotifast](docs/_reference/translating.md). Release
packaging is described in [PACKAGING.md](PACKAGING.md).

## Acknowledgements

Spotifast uses [librespot](https://github.com/librespot-org/librespot),
[egui](https://github.com/emilk/egui), the [Inter](https://rsms.me/inter/)
typeface (OFL), and [Lucide](https://lucide.dev) icons (ISC).

Spotiurge and Spotifast are independent projects and are not affiliated with Spotify.
Spotify is a trademark of Spotify AB.

Licensed under the [MIT License](LICENSE).
