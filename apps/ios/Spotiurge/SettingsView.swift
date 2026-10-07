// Settings: Spotify, playback on this iPhone, private-cloud pairing and
// sync, discovery, what leaves the phone, demo data and attribution.

import SpotiurgeCore
import SwiftUI

struct SettingsView: View {
    @Environment(SpotifyAccount.self) private var account
    @Environment(Player.self) private var player
    @Environment(DiscoveryModel.self) private var discovery
    @Environment(\.demoMode) private var demo
    @Binding var demoScenario: DemoScenario?
    @State private var endpoint = ""
    @State private var token = ""
    @State private var pairingError: String?

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    HStack(spacing: 12) {
                        Mark(size: 44)
                        VStack(alignment: .leading, spacing: 2) {
                            Text("Spotiurge for iPhone").font(Typeface.section)
                            Text("Development build. Not for distribution.").font(Typeface.detail).foregroundStyle(Palette.secondary)
                        }
                    }
                    .padding(.vertical, 4)
                    .accessibilityElement(children: .combine)
                }
                .listRowBackground(Palette.grouped)

                if demo {
                    Section { DemoBanner().listRowInsets(EdgeInsets()) }
                        .listRowBackground(Palette.grouped)
                }

                Section {
                    switch account.state {
                    case .signedIn(let name):
                        LabeledContent("Signed in", value: name)
                        Button("Sign out of Spotify", role: .destructive) { account.signOut() }
                    case .signingIn:
                        LabeledContent("Spotify", value: "Waiting for sign-in…")
                    case .failed(SpotifyAccount.notRemoved):
                        Notice(text: SpotifyAccount.notRemoved, symbol: "exclamationmark.circle")
                        Button("Sign out of Spotify", role: .destructive) { account.signOut() }
                    case .signedOut, .failed:
                        if case .failed(let message) = account.state { Notice(text: message, symbol: "exclamationmark.circle") }
                        Button("Sign in to Spotify") { account.signIn() }
                            .disabled(demo)
                    }
                } header: {
                    Text("Spotify").foregroundStyle(Palette.secondary)
                } footer: {
                    Text("Library, search and matching AI picks use Spotify's Web API through the shared app the desktop uses. The grant is kept in this iPhone's Keychain.")
                        .foregroundStyle(Palette.secondary)
                }
                .listRowBackground(Palette.grouped)

                Section {
                    if demo {
                        // The demo session has no engine and never touches the
                        // real playback credential.
                        LabeledContent("Status", value: "Demo: no audio")
                    } else {
                        LabeledContent("Status", value: engineStatus)
                        if case .failed(let message) = player.engine { Notice(text: message, symbol: "exclamationmark.circle") }
                        switch player.engine {
                        case .notIncluded, .connecting:
                            EmptyView()
                        case .needsSignIn, .failed:
                            Button("Sign in for playback on this iPhone") { player.signIn() }
                        case .ready:
                            Button("Take over from the active Spotify device") { player.takeOver() }
                        }
                        if Player.includesEngine && Keychain.has(Keychain.playback) {
                            Button("Forget playback credential", role: .destructive) { player.forget() }
                        }
                    }
                } header: {
                    Text("Playback on this iPhone").foregroundStyle(Palette.secondary)
                } footer: {
                    Text(Player.includesEngine
                         ? "Development engine: Spotiurge's own librespot build from the playback probe, playing through this app's audio session. It still announces the stock librespot iPhone identity, which Spotify refuses, until the identity fix ships in a pinned fork. Spotify Premium is required. Spotiurge never controls the Spotify app instead."
                         : "This build has no playback engine, so nothing plays here. Spotiurge never controls the Spotify app instead.")
                        .foregroundStyle(Palette.secondary)
                }
                .listRowBackground(Palette.grouped)

                Section {
                    LabeledContent("Status", value: pairingStatus)
                    if discovery.pairing == .unreadable && !demo { Notice(text: DiscoveryModel.keychainLocked, symbol: "lock") }
                    if let lastSync = discovery.lastSync {
                        LabeledContent("Last sync") { Text(lastSync, format: .relative(presentation: .named)) }
                    }
                    TextField("HTTPS origin", text: $endpoint)
                        .textContentType(.URL).keyboardType(.URL).textInputAutocapitalization(.never).autocorrectionDisabled()
                    SecureField("Pairing token", text: $token)
                        .textContentType(.password).textInputAutocapitalization(.never).autocorrectionDisabled()
                    if let pairingError { Notice(text: pairingError, symbol: "exclamationmark.circle") }
                    Button(pairButton) {
                        pairingError = discovery.pair(endpoint: endpoint, token: token)
                        token = ""
                    }
                    .disabled(token.isEmpty || demo)
                    Button(discovery.syncing ? "Syncing…" : "Sync now") { discovery.sync(manual: true) }
                        .disabled(!discovery.paired || discovery.syncing || demo)
                    if discovery.paired && !demo {
                        Button("Unpair this iPhone", role: .destructive) { discovery.unpair() }
                    }
                } header: {
                    Text("Private cloud").foregroundStyle(Palette.secondary)
                } footer: {
                    Text("Syncs your taste, ratings, saved mixes and AI history with your other devices, and brokers recommendations. The token is stored only in this iPhone's Keychain, bound to this origin.")
                        .foregroundStyle(Palette.secondary)
                }
                .listRowBackground(Palette.grouped)

                Section {
                    Toggle("Automatic picks", isOn: Binding(get: { discovery.preferences.automatic }, set: discovery.setAutomatic))
                        .tint(Palette.lamp)
                    ExplorationPicker(selection: discovery.exploration, onSelect: discovery.setExploration)
                        .listRowBackground(Color.clear)
                        .listRowInsets(EdgeInsets())
                } header: {
                    Text("Discovery").foregroundStyle(Palette.secondary)
                } footer: {
                    Text("Recommendations always use GPT-6 Luna through your private cloud's existing subscription, with no other model. Automatic picks refresh at most every twelve hours, and soon after you change your taste or exploration.")
                        .foregroundStyle(Palette.secondary)
                }
                .listRowBackground(Palette.grouped)

                Section {
                    Text("To your private cloud: your taste text, ratings (title, artist, rating and track URI), saved mixes and AI history. For recommendations, only the taste text and the titles, artists and ratings of up to 100 ratings. To Spotify: sign-in, library reads, searches and playback. No telemetry. Spotify grants never reach the private cloud or the AI.")
                        .font(Typeface.detail).foregroundStyle(Palette.secondary)
                } header: {
                    Text("What leaves this iPhone").foregroundStyle(Palette.secondary)
                }
                .listRowBackground(Palette.grouped)

                Section {
                    Toggle("Show demo data", isOn: Binding(get: { demoScenario != nil }, set: { demoScenario = $0 ? .home : nil }))
                        .tint(Palette.lamp)
                } footer: {
                    Text("Synthetic picks and library for trying the interface. Nothing is saved, synced or sent while it is on.")
                        .foregroundStyle(Palette.secondary)
                }
                .listRowBackground(Palette.grouped)

                Section {
                    LabeledContent("Version", value: Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "")
                    Text("Spotiurge is Serge's personal fork of Spotifast, under the MIT License. Music comes from Spotify; Spotify Premium is required for playback.")
                        .font(Typeface.detail).foregroundStyle(Palette.secondary)
                } header: {
                    Text("About").foregroundStyle(Palette.secondary)
                }
                .listRowBackground(Palette.grouped)
            }
            .foregroundStyle(Palette.text, Palette.secondary)
            .buttonStyle(ReadableButtonStyle())
            .font(Typeface.body)
            .roomBackground()
            .navigationTitle("Settings")
            .onAppear { endpoint = discovery.endpoint }
        }
    }

    private var pairButton: String {
        switch discovery.pairing {
        case .paired: "Pair again"
        case .unpaired: "Pair this iPhone"
        case .unreadable: "Replace pairing token"
        }
    }

    private var pairingStatus: String {
        switch discovery.pairing {
        case .paired: "Paired"
        case .unpaired: "Not paired"
        case .unreadable: "Keychain unavailable"
        }
    }

    private var engineStatus: String {
        switch player.engine {
        case .notIncluded: "Not in this build"
        case .needsSignIn: "Not signed in"
        case .connecting: "Connecting…"
        case .ready: "Ready"
        case .failed: "Unavailable"
        }
    }
}
