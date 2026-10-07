// Home, the For you radio desk (src/ui/discovery.rs on a phone): heading
// with play and refresh, the exploration fader, an honest one-line status,
// then picks with reasons and feedback. Unmatched picks and AI history
// start collapsed; saved mixes follow.

import SpotiurgeCore
import SwiftUI

struct HomeView: View {
    @Environment(DiscoveryModel.self) private var discovery
    @Environment(Player.self) private var player
    @Environment(SpotifyAccount.self) private var account
    @Environment(\.demoMode) private var demo
    var searchFor: (String) -> Void
    @State private var showUnmatched = false
    @State private var showHistory = false
    @State private var showOptions = false

    var body: some View {
        @Bindable var discovery = discovery
        NavigationStack {
            List {
                if demo { DemoBanner().roomRow() }
                header.roomRow().listRowSeparator(.hidden)
                ExplorationPicker(selection: discovery.exploration, onSelect: discovery.setExploration)
                    .roomRow().listRowSeparator(.hidden)
                statusLines.roomRow().listRowSeparator(.hidden)
                ForEach(discovery.playable, id: \.self) { pick in pickRow(pick) }
                if !discovery.unmatched.isEmpty { unmatched }
                mixes
                history
            }
            .listStyle(.plain)
            .roomBackground()
            .refreshable { discovery.sync(manual: true) }
            .navigationTitle("For you")
            .toolbarVisibility(.hidden, for: .navigationBar)
            .sheet(isPresented: $discovery.editingTaste) { TasteEditor() }
            .sheet(isPresented: $showOptions) { DiscoveryOptions() }
        }
    }

    // MARK: Header

    private var header: some View {
        HStack(alignment: .center, spacing: 8) {
            Mark(size: 32)
            VStack(alignment: .leading, spacing: 2) {
                Text("For you").font(Typeface.display).foregroundStyle(Palette.text).accessibilityAddTraits(.isHeader)
                Text(subtitle).font(Typeface.detail).foregroundStyle(Palette.secondary)
            }
            Spacer(minLength: 8)
            GlassEffectContainer(spacing: 8) {
                HStack(spacing: 8) {
                    Button {
                        discovery.recommend()
                    } label: {
                        Group {
                            if discovery.busy { ProgressView() } else { Image(systemName: "arrow.clockwise") }
                        }
                        .frame(width: 44, height: 44)
                    }
                    .buttonStyle(.plain)
                    .glassEffect(.regular.interactive(), in: .circle)
                    .disabled(discovery.busy || demo)
                    .accessibilityLabel(discovery.busy ? "Finding music" : "Find new picks")
                    moreMenu
                }
            }
            Button {
                if let first = discovery.playable.first?.track {
                    player.play(first, from: discovery.playable.compactMap(\.track))
                }
            } label: {
                Image(systemName: "play.fill")
            }
            .buttonStyle(LampButtonStyle())
            .disabled(discovery.playable.isEmpty)
            .accessibilityLabel("Play all")
            .sensoryFeedback(.impact(weight: .light), trigger: player.playing)
        }
        .padding(.top, 8)
    }

    private var subtitle: String {
        if demo { return "Demo picks" }
        if discovery.busy { return "Finding music…" }
        if let at = discovery.preferences.refreshedAt {
            let date = Date(timeIntervalSince1970: TimeInterval(at))
            return "Updated " + date.formatted(.relative(presentation: .named))
        }
        return discovery.picks.isEmpty ? "From your saved taste" : "Saved picks"
    }

    /// Opens an opaque options sheet: DESIGN.md never lets content ghost
    /// through anything floating over it, which a translucent menu would.
    private var moreMenu: some View {
        Button { showOptions = true } label: {
            Image(systemName: "ellipsis").frame(width: 44, height: 44)
        }
        .buttonStyle(.plain)
        .glassEffect(.regular.interactive(), in: .circle)
        .disabled(demo)
        .accessibilityLabel("More discovery options")
    }

    // MARK: Status

    @ViewBuilder private var statusLines: some View {
        VStack(alignment: .leading, spacing: 6) {
            if !discovery.picks.isEmpty {
                Text(countsLine).font(Typeface.detail).foregroundStyle(Palette.secondary).monospacedDigit()
            }
            if demo {
                Notice(text: "Demo discoveries. Feedback, recommendations and cloud requests are disabled.")
            }
            if !discovery.status.isEmpty {
                Notice(text: discovery.status, symbol: discovery.lastError == nil ? "info.circle" : "exclamationmark.circle")
            }
            if discovery.ready && !demo {
                switch discovery.pairing {
                case .paired: EmptyView()
                case .unpaired:
                    Notice(text: "Not paired with your private cloud. Pair in Settings to get picks and sync with your other devices; picks you already have stay here.", symbol: "link")
                case .unreadable where discovery.status != DiscoveryModel.keychainLocked:
                    Notice(text: DiscoveryModel.keychainLocked, symbol: "lock")
                case .unreadable: EmptyView()
                }
            }
            if let reason = player.unavailableReason, !demo, !discovery.playable.isEmpty {
                Notice(text: reason, symbol: "speaker.slash")
            }
            if discovery.picks.isEmpty && !discovery.busy {
                emptyState
            }
            if discovery.picks.contains(where: { !$0.checked }) && !demo && account.signedIn {
                Button("Check again") { discovery.retryMatches() }
                    .font(Typeface.inter(14, .semibold))
                    .buttonStyle(.glass)
                    .disabled(discovery.busy)
            }
        }
    }

    private var countsLine: String {
        let ready = discovery.playable.count
        let missing = discovery.picks.filter { $0.checked && $0.track == nil }.count
        let unchecked = discovery.picks.filter { !$0.checked }.count
        var parts = ["\(ready) ready to play"]
        if missing > 0 { parts.append("\(missing) not found on Spotify") }
        if unchecked > 0 { parts.append("\(unchecked) not checked yet") }
        var line = parts.joined(separator: " · ")
        if unchecked > 0, let reason = catalogueReason { line += ". " + reason }
        return line
    }

    private var catalogueReason: String? {
        switch discovery.catalogue {
        case .complete: nil
        case .timedOut: "Spotify took too long to answer."
        case .rateLimited: "Spotify asked Spotiurge to slow down."
        case .quotaExhausted: "Spotiurge has used its Spotify search allowance for now."
        case .unavailable: "Spotify search is unavailable right now."
        case .signInNeeded: "Sign in to Spotify to check the rest."
        }
    }

    @ViewBuilder private var emptyState: some View {
        let hasTaste = discovery.ready && discovery.replica.document.hasInputs
        VStack(alignment: .leading, spacing: 10) {
            Text(hasTaste ? "No picks yet." : "Tell Spotiurge what you like")
                .font(Typeface.title).foregroundStyle(Palette.text)
            Text(hasTaste
                 ? "Refresh to find music for your taste. Picks arrive automatically once this iPhone is paired with your private cloud."
                 : "Describe the music you love, or rate a few tracks. Spotiurge asks GPT-6 Luna through your private cloud and checks every pick on Spotify.")
                .font(Typeface.body).foregroundStyle(Palette.secondary)
            Button(hasTaste ? "Tune taste" : "Write your taste") { discovery.editingTaste = true }
                .font(Typeface.inter(15, .semibold))
                .buttonStyle(.glass)
                .disabled(demo)
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .leading)
        .paneGlass()
        .padding(.top, 8)
    }

    // MARK: Rows

    private func pickRow(_ pick: Pick) -> some View {
        let track = pick.track!
        let rating = discovery.rating(track)
        return HStack(spacing: 4) {
            Button {
                player.play(track, from: discovery.playable.compactMap(\.track))
            } label: {
                TrackRow(track: track, reason: pick.suggestion.reason, live: player.nowPlaying?.uri == track.uri)
            }
            .buttonStyle(.plain)
            .accessibilityHint("Plays this pick")
            FeedbackButtons(rating: rating, enabled: !demo) { discovery.rate(track, $0) }
        }
        .roomRow()
        .swipeActions(edge: .leading) {
            Button("More like this", systemImage: "hand.thumbsup") { discovery.rate(track, rating == .love ? nil : .love) }.tint(Palette.surfaceActive)
        }
        .swipeActions(edge: .trailing) {
            Button("Less like this", systemImage: "hand.thumbsdown") { discovery.rate(track, rating == .less ? nil : .less) }.tint(Palette.surfaceActive)
        }
    }

    @ViewBuilder private var unmatched: some View {
        Section {
            DisclosureGroup(isExpanded: $showUnmatched) {
                ForEach(discovery.unmatched, id: \.self) { pick in
                    HStack(spacing: 12) {
                        VStack(alignment: .leading, spacing: 2) {
                            Text(pick.suggestion.title).font(Typeface.rowTitle).foregroundStyle(Palette.text)
                            Text(pick.suggestion.artist).font(Typeface.detail).foregroundStyle(Palette.secondary)
                            Text(pick.checked ? "Not found on Spotify" : "Not checked on Spotify yet")
                                .font(Typeface.caption).foregroundStyle(Palette.dim)
                        }
                        .frame(maxWidth: .infinity, alignment: .leading)
                        Button("Search Spotify") { searchFor("\(pick.suggestion.title) \(pick.suggestion.artist)") }
                            .font(Typeface.inter(13, .semibold))
                            .buttonStyle(.glass)
                    }
                    .frame(minHeight: 56)
                    .accessibilityElement(children: .combine)
                }
            } label: {
                SectionTitle(text: "Couldn't play (\(discovery.unmatched.count))")
            }
            .tint(Palette.secondary)
            .roomRow()
        }
    }

    @ViewBuilder private var mixes: some View {
        let mixes = discovery.replica.document.mixes
        if !mixes.isEmpty {
            Section {
                ForEach(mixes.prefix(5), id: \.key) { mix in
                    HStack {
                        VStack(alignment: .leading, spacing: 2) {
                            Text(mix.title).font(Typeface.rowTitle).foregroundStyle(Palette.text)
                            Text("\(mix.uris.count) tracks").font(Typeface.detail).foregroundStyle(Palette.secondary)
                        }
                        Spacer()
                        Button("Play mix", systemImage: "play.fill") { playMix(mix.uris) }
                            .labelStyle(.iconOnly)
                            .frame(width: 44, height: 44)
                            .buttonStyle(.glass)
                    }
                    .frame(minHeight: 56)
                    .roomRow()
                    .swipeActions { Button("Remove", systemImage: "trash", role: .destructive) { discovery.deleteMix(mix.key) } }
                }
            } header: {
                SectionTitle(text: "Your saved mixes").padding(.top, 12)
            }
        }
    }

    private func playMix(_ uris: [String]) {
        let known = discovery.picks.compactMap(\.track) + account.savedTracks
        let tracks = uris.compactMap { uri in known.first { $0.uri == uri } }
        if let first = tracks.first {
            player.play(first, from: tracks)
        } else {
            player.notice = "This mix's tracks are not loaded on this iPhone yet."
        }
    }

    @ViewBuilder private var history: some View {
        let entries = discovery.replica.document.recentHistory
        if !entries.isEmpty {
            Section {
                DisclosureGroup(isExpanded: $showHistory) {
                    ForEach(entries, id: \.key) { entry in
                        if case .history(let prompt, let suggestions) = entry.record.value {
                            VStack(alignment: .leading, spacing: 4) {
                                Text(prompt.isEmpty ? "From your feedback" : prompt).font(Typeface.body).foregroundStyle(Palette.text).lineLimit(2)
                                Text(suggestions.prefix(4).map { "\($0.title) – \($0.artist)" }.joined(separator: " · "))
                                    .font(Typeface.detail).foregroundStyle(Palette.secondary).lineLimit(2)
                            }
                            .padding(.vertical, 4)
                            .accessibilityElement(children: .combine)
                        }
                    }
                } label: {
                    SectionTitle(text: "Your AI history")
                }
                .tint(Palette.secondary)
                .roomRow()
            }
        }
    }
}

/// The For you options, on an opaque overlay sheet.
struct DiscoveryOptions: View {
    @Environment(DiscoveryModel.self) private var discovery
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            List {
                Button("Tune taste", systemImage: "slider.horizontal.3") {
                    dismiss()
                    discovery.editingTaste = true
                }
                Button("Save this mix", systemImage: "square.and.arrow.down") {
                    discovery.saveMix()
                    dismiss()
                }
                .disabled(discovery.playable.isEmpty)
                Button(discovery.syncing ? "Syncing…" : "Sync my devices", systemImage: "arrow.triangle.2.circlepath") {
                    discovery.sync(manual: true)
                    dismiss()
                }
                .disabled(discovery.syncing)
                Toggle("Automatic picks", systemImage: "sparkles", isOn: Binding(get: { discovery.preferences.automatic }, set: discovery.setAutomatic))
                    .tint(Palette.lamp)
            }
            .font(Typeface.body)
            .foregroundStyle(Palette.text)
            .scrollContentBackground(.hidden)
            .navigationTitle("Discovery options")
            .navigationBarTitleDisplayMode(.inline)
        }
        .presentationDetents([.height(300), .medium])
        .presentationBackground(Palette.overlay)
    }
}

/// The taste editor sheet: plain text, bounded to the wire limit.
struct TasteEditor: View {
    @Environment(DiscoveryModel.self) private var discovery
    @FocusState private var focused: Bool

    var body: some View {
        @Bindable var discovery = discovery
        NavigationStack {
            VStack(alignment: .leading, spacing: 12) {
                Text("Describe the music you love. Spotiurge sends this text and your ratings, never your account or listening history, to GPT-6 Luna through your private cloud.")
                    .font(Typeface.detail).foregroundStyle(Palette.secondary)
                TextEditor(text: Binding(get: { discovery.draft }, set: { discovery.draft = limitPrompt($0) }))
                    .font(Typeface.body)
                    .focused($focused)
                    .scrollContentBackground(.hidden)
                    .padding(8)
                    .background(Palette.surface, in: .rect(cornerRadius: Radius.popover))
                    .overlay(alignment: .topLeading) {
                        if discovery.draft.isEmpty {
                            Text("Warm jazz, spacious electronics, a few surprises…").font(Typeface.body).foregroundStyle(Palette.dim)
                                .padding(16).allowsHitTesting(false)
                        }
                    }
                Text("\(discovery.draft.utf8.count) / \(DiscoveryLimits.maxPromptBytes) bytes")
                    .font(Typeface.caption).foregroundStyle(Palette.dim).monospacedDigit()
            }
            .padding(16)
            .navigationTitle("Your taste")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) { Button("Cancel") { discovery.cancelTaste() } }
                ToolbarItem(placement: .confirmationAction) { Button("Save taste") { discovery.saveTaste() } }
            }
            .onAppear { focused = true }
        }
        .presentationDetents([.medium, .large])
        .presentationBackground(Palette.overlay)
    }
}
