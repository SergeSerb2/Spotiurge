// Playback on this iPhone through Spotiurge's own Rust engine: the playback
// probe's librespot staticlib (probes/ios-playback/engine), linked only in
// development builds that define SPOTIURGE_ENGINE (see build.sh). The app
// owns the audio session and output graph, as the probe proved on a device.
//
// There is no fallback: without the engine, or before it signs in, playback
// is plainly unavailable. This never turns into a remote control for the
// Spotify app.

import AVFAudio
import Foundation
import MediaPlayer
import Observation
import SpotiurgeCore
import UIKit

struct NowPlaying: Equatable {
    var uri: String
    var title: String
    var artist: String
    var durationMs: Int
    var imageURL: URL?
    var light: [UInt32] = []
}

@Observable
final class Player {
    static let shared = Player()

    enum Engine: Equatable {
        /// This build has no engine linked.
        case notIncluded
        case needsSignIn
        case connecting
        case ready
        case failed(String)
    }

    static let locked = "The playback sign-in is locked in the Keychain. Unlock this iPhone and reopen Spotiurge."

    #if SPOTIURGE_ENGINE
    static let includesEngine = true
    #else
    static let includesEngine = false
    #endif

    private(set) var engine: Engine = includesEngine ? .needsSignIn : .notIncluded
    private(set) var nowPlaying: NowPlaying?
    private(set) var playing = false
    /// The last reported position and when it was reported; the scrubber
    /// extrapolates between engine events while playing.
    private(set) var position: (ms: Int, at: Date) = (0, .now)
    /// A one-line honest note about the last request, shown near the transport.
    var notice: String?
    /// Synthetic demo playback: no engine, no audio, clearly labelled.
    private(set) var demo = false

    @ObservationIgnored private var requested: [String: Track] = [:]
    @ObservationIgnored private let secrets: SecretStore

    init(secrets: SecretStore = .keychain) {
        self.secrets = secrets
    }

    func positionMs(at date: Date) -> Int {
        guard playing else { return position.ms }
        let elapsed = Int(date.timeIntervalSince(position.at) * 1000)
        return min(position.ms + elapsed, nowPlaying?.durationMs ?? .max)
    }

    var unavailableReason: String? {
        switch engine {
        case .ready: nil
        case .notIncluded: "This build has no playback engine. Build with the development engine to play on this iPhone."
        case .needsSignIn: "Sign in for playback on this iPhone in Settings."
        case .connecting: "Connecting playback on this iPhone…"
        case .failed(let message): message
        }
    }

    // MARK: - Requests

    /// Plays one track. The development engine loads one track or one
    /// context at a time; it cannot yet queue a list of picks.
    func play(_ track: Track, from list: [Track] = []) {
        guard ready(for: track.name) else { return }
        requested[track.uri] = track
        nowPlaying = NowPlaying(uri: track.uri, title: track.name, artist: track.artistLine, durationMs: track.durationMs, imageURL: track.imageURL)
        notice = list.count > 1 ? "This development engine plays one pick at a time; queueing a list needs the production engine." : nil
        load(track.uri)
    }

    func playContext(uri: String, title: String, subtitle: String, imageURL: URL?) {
        guard ready(for: title) else { return }
        notice = nil
        nowPlaying = NowPlaying(uri: uri, title: title, artist: subtitle, durationMs: 0, imageURL: imageURL)
        load(uri)
    }

    func toggle() {
        if demo {
            setPlaying(!playing, at: positionMs(at: .now))
            return
        }
        guard engine == .ready else { return }
        setPlaying(!playing, at: positionMs(at: .now))
        command(playing ? 0 : 1)
    }

    func next() { if !demo { command(2) } }
    func previous() { if !demo { command(3) } }
    /// Takes over playback from the active Spotify Connect device.
    func takeOver() { if !demo { command(4) } }

    /// Optimistic, so the button answers at once; engine events confirm.
    private func setPlaying(_ value: Bool, at ms: Int) {
        playing = value
        position = (ms, .now)
        updateNowPlayingInfo()
    }

    private func ready(for title: String) -> Bool {
        if demo {
            notice = "Demo data: no audio plays and nothing reaches Spotify."
            return false
        }
        guard engine == .ready else {
            notice = unavailableReason
            return false
        }
        return true
    }

    // MARK: - Demo

    func showDemo(_ track: NowPlaying, positionMs: Int, playing: Bool) {
        demo = true
        nowPlaying = track
        position = (positionMs, .now)
        self.playing = playing
    }

    func setLight(_ light: [UInt32], for uri: String) {
        if nowPlaying?.uri == uri { nowPlaying?.light = light }
    }

    // MARK: - Engine bridge

    private let audio = AVAudioEngine()
    private var wantsAudio = false
    private var wasBackgrounded = false
    private var idleSince: Date?
    private var started = false
    /// Whether audio was playing when an interruption began; only then may
    /// its end resume playback.
    private var resumeAfterInterruption = false

    /// Starts the engine once per process. Never starts a second session.
    func start() {
        guard !started, !demo else { return }
        started = true
        #if SPOTIURGE_ENGINE
        let cache = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask)[0].appending(path: "librespot").path
        try? FileManager.default.createDirectory(atPath: cache, withIntermediateDirectories: true)
        let started = probe_start(cache, "Spotiurge (iPhone)", { json, _ in
            guard let json else { return }
            let line = String(cString: json)
            DispatchQueue.main.async { MainActor.assumeIsolated { Player.shared.event(line) } }
        }, { data, length, _ in
            guard let data else { return }
            let blob = Data(bytes: data, count: length)
            DispatchQueue.main.async {
                MainActor.assumeIsolated {
                    // The session keeps playing; only reconnecting later needs it.
                    do { try Player.shared.secrets.write(blob, Keychain.playback) } catch {
                        Player.shared.notice = "The Keychain did not keep the playback sign-in; you may need to sign in again later."
                    }
                }
            }
        }, nil)
        guard started == 0 else {
            engine = .failed("The playback engine did not start.")
            return
        }
        configureAudio()
        observeSystem()
        configureRemoteCommands()
        Timer.scheduledTimer(withTimeInterval: 30, repeats: true) { _ in
            MainActor.assumeIsolated { Player.shared.stopIdleAudio() }
        }
        switch Result(catching: { () throws(KeychainError) in try secrets.read(Keychain.playback) }) {
        case .success(let stored?): connect(kind: 1, stored)
        case .success(nil): break
        case .failure: engine = .failed(Self.locked)
        }
        #endif
    }

    func signIn() {
        #if SPOTIURGE_ENGINE
        guard !demo, engine != .connecting else { return }
        engine = .connecting
        Task {
            do {
                let tokens = try await PKCESignIn(.playback).run()
                connect(kind: 0, Data(tokens.access_token.utf8))
            } catch {
                engine = .failed((error as? SignInError)?.errorDescription ?? SignInError.failed.errorDescription!)
            }
        }
        #endif
    }

    /// The engine and the credential belong to the process, never to a
    /// demo session.
    func forget() {
        guard !demo else { return }
        do { try secrets.delete(Keychain.playback) } catch {
            notice = "The Keychain did not remove the playback sign-in. Try again after unlocking."
            return
        }
        #if SPOTIURGE_ENGINE
        probe_disconnect()
        engine = .needsSignIn
        playing = false
        #endif
    }

    #if SPOTIURGE_ENGINE
    private func connect(kind: UInt32, _ data: Data) {
        engine = .connecting
        let result = data.withUnsafeBytes { probe_connect(kind, $0.bindMemory(to: UInt8.self).baseAddress, data.count) }
        if result != 0 { engine = .failed("The playback engine refused the credential.") }
    }

    private func load(_ uri: String) {
        if probe_load(uri) != 0 { notice = "Playback on this iPhone is not connected yet." }
    }

    private func command(_ command: UInt32) {
        if engine == .ready { _ = probe_command(command) }
    }

    private func event(_ line: String) {
        guard let data = line.data(using: .utf8),
              let event = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any],
              let type = event["t"] as? String
        else { return }
        let ms = event["position_ms"] as? Int
        switch type {
        case "connected":
            engine = .ready
        case "session_ended":
            switch Result(catching: { () throws(KeychainError) in try secrets.read(Keychain.playback) }) {
            case .success(nil): engine = .needsSignIn
            case .success: engine = .failed("Spotify ended this iPhone's playback session. Return to Spotiurge or sign in for playback again in Settings.")
            case .failure: engine = .failed(Self.locked)
            }
            playing = false
            idleSince = idleSince ?? .now
        case "track":
            let uri = event["uri"] as? String ?? ""
            let known = requested[uri]
            nowPlaying = NowPlaying(uri: uri, title: event["name"] as? String ?? "", artist: event["artists"] as? String ?? "",
                                    durationMs: event["duration_ms"] as? Int ?? 0, imageURL: known?.imageURL)
        case "playing":
            idleSince = nil
            setPlaying(true, at: ms ?? position.ms)
            activateAudio()
        case "paused", "stopped":
            idleSince = idleSince ?? .now
            setPlaying(false, at: ms ?? position.ms)
        case "position", "seeked":
            position = (ms ?? position.ms, .now)
        case "unavailable":
            notice = "Spotify says this track is unavailable here."
        case "error":
            // Engine messages carry no secrets, but they are terse; keep the
            // user-facing line plain and actionable.
            if event["stage"] as? String == "connect" {
                engine = .failed("Spotify refused playback sign-in on this iPhone. The development engine still announces the stock librespot iPhone identity; see Settings.")
            } else {
                notice = "The playback engine reported an error."
            }
        default:
            return
        }
        updateNowPlayingInfo()
    }

    private func configureAudio() {
        do {
            try AVAudioSession.sharedInstance().setCategory(.playback, mode: .default, policy: .longFormAudio)
        } catch {
            engine = .failed("iOS refused the playback audio session.")
        }
        let format = AVAudioFormat(standardFormatWithSampleRate: 44_100, channels: 2)!
        let source = AVAudioSourceNode(format: format, renderBlock: Self.render)
        audio.attach(source)
        audio.connect(source, to: audio.mainMixerNode, format: format)
    }

    /// Core Audio's real-time thread: never main-actor isolated, never blocks.
    private nonisolated static func render(_: UnsafeMutablePointer<ObjCBool>, _: UnsafePointer<AudioTimeStamp>, frameCount: UInt32,
                                           bufferList: UnsafeMutablePointer<AudioBufferList>) -> OSStatus {
        let buffers = UnsafeMutableAudioBufferListPointer(bufferList)
        guard buffers.count >= 2,
              let left = buffers[0].mData?.assumingMemoryBound(to: Float.self),
              let right = buffers[1].mData?.assumingMemoryBound(to: Float.self)
        else { return noErr }
        _ = probe_render(left, right, Int(frameCount))
        return noErr
    }

    private func activateAudio() {
        wantsAudio = true
        try? AVAudioSession.sharedInstance().setActive(true)
        if !audio.isRunning { try? audio.start() }
    }

    private func stopIdleAudio() {
        // ponytail: fixed five-minute idle stop, as in the probe; tie it to a
        // Connect reachability policy in the production engine.
        guard !playing, wantsAudio, let idleSince, Date().timeIntervalSince(idleSince) > 300 else { return }
        wantsAudio = false
        audio.stop()
        try? AVAudioSession.sharedInstance().setActive(false, options: .notifyOthersOnDeactivation)
    }

    private func observeSystem() {
        let center = NotificationCenter.default
        center.addObserver(forName: AVAudioSession.interruptionNotification, object: nil, queue: .main) { note in
            let info = note.userInfo ?? [:]
            let began = (info[AVAudioSessionInterruptionTypeKey] as? UInt) == AVAudioSession.InterruptionType.began.rawValue
            let resume = AVAudioSession.InterruptionOptions(rawValue: info[AVAudioSessionInterruptionOptionKey] as? UInt ?? 0).contains(.shouldResume)
            MainActor.assumeIsolated {
                let player = Player.shared
                if began {
                    player.resumeAfterInterruption = player.playing
                    probe_flush()
                    if player.playing { player.command(1) }
                } else {
                    let resumePlayback = resume && player.resumeAfterInterruption
                    player.resumeAfterInterruption = false
                    if resumePlayback {
                        player.activateAudio()
                        player.command(0)
                    }
                }
            }
        }
        center.addObserver(forName: AVAudioSession.routeChangeNotification, object: nil, queue: .main) { note in
            let reason = note.userInfo?[AVAudioSessionRouteChangeReasonKey] as? UInt
            MainActor.assumeIsolated {
                // Apple's convention: pause when headphones are removed.
                if reason == AVAudioSession.RouteChangeReason.oldDeviceUnavailable.rawValue, Player.shared.playing {
                    Player.shared.command(1)
                }
            }
        }
        center.addObserver(forName: .AVAudioEngineConfigurationChange, object: audio, queue: .main) { _ in
            MainActor.assumeIsolated {
                let player = Player.shared
                if player.wantsAudio, !player.audio.isRunning { player.activateAudio() }
            }
        }
        center.addObserver(forName: UIApplication.didEnterBackgroundNotification, object: nil, queue: .main) { _ in
            MainActor.assumeIsolated { Player.shared.wasBackgrounded = true }
        }
        // iOS suspends an idle app in the background, leaving its Connect
        // session stale. Replace it only after a real return, never at launch.
        center.addObserver(forName: UIApplication.willEnterForegroundNotification, object: nil, queue: .main) { _ in
            MainActor.assumeIsolated {
                let player = Player.shared
                guard player.wasBackgrounded, !player.playing, let stored = try? player.secrets.read(Keychain.playback) else { return }
                probe_disconnect()
                player.connect(kind: 1, stored)
            }
        }
    }

    private func configureRemoteCommands() {
        let commands = MPRemoteCommandCenter.shared()
        commands.playCommand.addTarget { _ in probe_command(0) == 0 ? .success : .commandFailed }
        commands.pauseCommand.addTarget { _ in probe_command(1) == 0 ? .success : .commandFailed }
        commands.togglePlayPauseCommand.addTarget { _ in
            MainActor.assumeIsolated { Player.shared.toggle() }
            return .success
        }
        commands.nextTrackCommand.addTarget { _ in probe_command(2) == 0 ? .success : .commandFailed }
        commands.previousTrackCommand.addTarget { _ in probe_command(3) == 0 ? .success : .commandFailed }
    }
    #else
    private func load(_ uri: String) {}
    private func command(_ command: UInt32) {}
    #endif

    private func updateNowPlayingInfo() {
        guard !demo, let nowPlaying else { return }
        MPNowPlayingInfoCenter.default().nowPlayingInfo = [
            MPMediaItemPropertyTitle: nowPlaying.title,
            MPMediaItemPropertyArtist: nowPlaying.artist,
            MPMediaItemPropertyPlaybackDuration: Double(nowPlaying.durationMs) / 1000,
            MPNowPlayingInfoPropertyElapsedPlaybackTime: Double(position.ms) / 1000,
            MPNowPlayingInfoPropertyPlaybackRate: playing ? 1.0 : 0.0,
        ]
    }
}
