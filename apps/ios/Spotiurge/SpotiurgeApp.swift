// Spotiurge for iPhone (development). A system tab bar moves between
// channels (Home, Library, Search, Settings); the floating console above it
// is the mini-player, and Now Playing opens as a sheet. Content sits on the
// cover-lit room; glass carries only the controls.

import SpotiurgeCore
import SwiftUI

@main
struct SpotiurgeApp: App {
    /// Hosted unit tests build their own models; the host shows demo data so
    /// it starts no real session.
    @State private var sessions = Sessions(launch: ProcessInfo.processInfo.environment["XCTestConfigurationFilePath"] != nil ? .onboarding : DemoScenario.launch)

    init() {
        // Inter for the system bars too, scaled with Dynamic Type.
        if let bold = UIFont(name: "InterVariable-Bold", size: 34), let semibold = UIFont(name: "InterVariable-SemiBold", size: 17) {
            let appearance = UINavigationBar.appearance()
            appearance.largeTitleTextAttributes = [.font: UIFontMetrics(forTextStyle: .largeTitle).scaledFont(for: bold)]
            appearance.titleTextAttributes = [.font: UIFontMetrics(forTextStyle: .headline).scaledFont(for: semibold)]
        }
    }

    var body: some Scene {
        WindowGroup {
            Root(session: sessions.current, demo: sessions.scenario,
                 demoScenario: Binding(get: { sessions.scenario }, set: { sessions.select($0) }))
                // Fresh view state whenever demo data is turned on or off;
                // the models live in `sessions`.
                .id(sessions.scenario)
        }
    }
}

/// The models for one session: real, or demo.
struct Session {
    let discovery: DiscoveryModel
    let account: SpotifyAccount
    let player: Player

    /// Created at most once per process, by `Sessions`.
    static func real() -> Session {
        let session = Session(discovery: DiscoveryModel(), account: SpotifyAccount(), player: .shared)
        session.player.start()
        session.discovery.account = session.account
        session.discovery.load()
        session.account.restore()
        return session
    }

    /// Synthetic models with their own player, which never touches the engine.
    static func demo(_ scenario: DemoScenario) -> Session {
        let session = Session(discovery: DiscoveryModel(), account: SpotifyAccount(), player: Player())
        session.discovery.showDemo(scenario)
        session.player.showDemo(Demo.nowPlaying, positionMs: 83_000, playing: false)
        return session
    }
}

/// One real session for the life of the process, so there is only ever one
/// replica writer. Demo sessions come and go; while one shows, the real
/// session is suspended (nothing new is sent and late answers are dropped)
/// and its playback is left alone.
@Observable
final class Sessions {
    private(set) var scenario: DemoScenario?
    private(set) var current: Session
    @ObservationIgnored private var real: Session?
    @ObservationIgnored private let makeReal: @MainActor () -> Session

    init(launch: DemoScenario?, makeReal: @escaping @MainActor () -> Session = Session.real) {
        self.makeReal = makeReal
        scenario = launch
        if let launch {
            current = .demo(launch)
        } else {
            let real = makeReal()
            self.real = real
            current = real
        }
    }

    func select(_ scenario: DemoScenario?) {
        guard scenario != self.scenario else { return }
        self.scenario = scenario
        if let scenario {
            real?.discovery.suspend()
            real?.account.suspend()
            current = .demo(scenario)
        } else if let real {
            real.account.resume()
            real.account.foreground()
            real.discovery.resume()
            current = real
        } else {
            let real = makeReal()
            self.real = real
            current = real
        }
    }
}

enum AppTab: Hashable { case home, library, search, settings }

struct Root: View {
    let session: Session
    let demo: DemoScenario?
    @Binding var demoScenario: DemoScenario?
    @State private var tab = AppTab.home
    @State private var query = ""
    @State private var showNowPlaying = false
    @State private var light: [UInt32] = []
    @Environment(\.scenePhase) private var phase
    @Environment(\.accessibilityReduceMotion) private var reduceMotion

    var body: some View {
        TabView(selection: $tab) {
            // The amber lamp marks the current channel in the tab bar only;
            // controls inside the pages stay neutral.
            Tab("Home", systemImage: "dot.radiowaves.left.and.right", value: .home) {
                HomeView { text in
                    query = text
                    tab = .search
                }
                .tint(Palette.text)
            }
            Tab("Library", systemImage: "square.stack", value: .library) { LibraryView().tint(Palette.text) }
            Tab("Settings", systemImage: "gearshape", value: .settings) { SettingsView(demoScenario: $demoScenario).tint(Palette.text) }
            Tab(value: .search, role: .search) { SearchView(query: $query).tint(Palette.text) }
        }
        .tint(Palette.lamp)
        .tabBarMinimizeBehavior(.onScrollDown)
        .tabViewBottomAccessory {
            MiniPlayer { showNowPlaying = true }
        }
        .sheet(isPresented: $showNowPlaying) { NowPlayingSheet().tint(Palette.text) }
        .environment(\.roomLight, light.map { Color(hex: $0) })
        .environment(session.discovery)
        .environment(session.account)
        .environment(session.player)
        .environment(\.demoMode, demo != nil)
        .fontDesign(.default)
        .task(id: session.player.nowPlaying?.uri) { await updateLight() }
        .onChange(of: phase) { _, phase in
            if phase == .active {
                session.account.foreground()
                session.discovery.foreground()
            }
        }
    }

    /// The room takes its light from the playing cover, computed off the main actor.
    private func updateLight() async {
        guard let now = session.player.nowPlaying else { return }
        if demo != nil {
            light = Demo.tile(now.uri)
        } else if let url = now.imageURL {
            let colors = await CoverLight.shared.colors(for: url)
            // A newer track may have started while this cover was fetched.
            guard !Task.isCancelled, session.player.nowPlaying?.uri == now.uri else { return }
            light = colors
            session.player.setLight(colors, for: now.uri)
        }
    }
}
