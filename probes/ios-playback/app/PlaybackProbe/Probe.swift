import Security
import AVFAudio
import Foundation
import MediaPlayer
import UIKit

/// Owns the app's audio session and output graph and bridges the Rust
/// engine. Every observation is appended to Documents/evidence.jsonl:
/// track metadata, counters, session and lock state. Never audio, tokens or
/// credentials.
@MainActor
final class Probe {
    static let shared = Probe()

    private var audio = ProbeAudioGraph(render: Probe.render)
    private var engine: AVAudioEngine { audio.engine }
    private var engineObserver: NSObjectProtocol?
    private var awaitingPlaybackRequest = false
    private(set) var connected = false
    private(set) var playing = false
    private(set) var track = ""
    private(set) var positionMs: UInt32 = 0
    /// The latest failure, shown on screen so a silent failure is visible.
    private(set) var lastError = ""
    private var durationMs: UInt32 = 0
    private var wantsEngine = false
    private var autoplayed = false
    private var wasBackgrounded = false
    /// When playback last stopped; the output stops after five idle minutes.
    private var idleSince: Date?
    private var interruptionPlayback = InterruptionPlayback()
    var onChange: (() -> Void)?

    private let evidenceURL = FileManager.default
        .urls(for: .documentDirectory, in: .userDomainMask)[0]
        .appendingPathComponent("evidence.jsonl")

    // MARK: - Start

    func start() {
        let cache = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("librespot").path
        try? FileManager.default.createDirectory(atPath: cache, withIntermediateDirectories: true)
        let result = probe_start(cache, "Spotiurge Probe (iPhone)", { json, _ in
            guard let json else { return }
            let line = String(cString: json)
            DispatchQueue.main.async { Probe.shared.engineEvent(line) }
        }, { data, length, _ in
            guard let data else { return }
            let blob = Data(bytes: data, count: length)
            DispatchQueue.main.async {
                let status = CredentialStore.save(blob)
                if status == errSecSuccess {
                    Probe.shared.record(["t": "credential_stored", "bytes": blob.count])
                } else {
                    Probe.shared.record(["t": "credential_store_failed", "status": Int(status)])
                    Probe.shared.failed("credential_store", "could not protect the reusable credential; the prior credential is preserved")
                }
            }
        }, nil)
        record(["t": "launch", "probe_start": Int(result), "build": Bundle.main.infoDictionary?["CFBundleVersion"] as? String ?? "?"])

        configureSession()
        observeSession()
        configureRemoteCommands()
        Timer.scheduledTimer(withTimeInterval: 15, repeats: true) { _ in
            MainActor.assumeIsolated { Probe.shared.heartbeat() }
        }
        if let stored = CredentialStore.load() {
            connect(kind: 1, stored)
        }
    }

    func failed(_ stage: String, _ message: String) {
        lastError = "\(stage): \(message)"
        record(["t": "error", "stage": stage, "msg": message])
        onChange?()
    }

    /// iOS suspends the app while it is idle in the background, which leaves
    /// the Connect session stale. Start a fresh one from the stored credential.
    private func refreshSessionIfIdle() {
        // The scene's first foreground at launch is not a return from
        // suspension; reconnecting then would start a second session.
        guard wasBackgrounded, !playing, let stored = CredentialStore.load() else { return }
        probe_disconnect()
        connected = false
        connect(kind: 1, stored)
    }

    func connect(token: String) {
        connect(kind: 0, Data(token.utf8))
    }

    private func connect(kind: UInt32, _ data: Data) {
        let result = data.withUnsafeBytes { probe_connect(kind, $0.bindMemory(to: UInt8.self).baseAddress, data.count) }
        record(["t": "connect_requested", "kind": kind == 0 ? "fresh_sign_in" : "stored_credential", "result": Int(result)])
    }

    // MARK: - Audio session and graph

    private func configureSession() {
        let session = AVAudioSession.sharedInstance()
        do {
            try session.setCategory(.playback, mode: .default, policy: .longFormAudio)
        } catch {
            record(["t": "error", "stage": "category", "msg": error.localizedDescription])
        }
    }

    private func mediaServicesReset() {
        record(["t": "media_services_reset"])
        // Apple requires new audio objects and session configuration, with
        // playback left stopped until a new user request.
        awaitingPlaybackRequest = true
        wantsEngine = false
        playing = false
        interruptionPlayback = InterruptionPlayback()
        probe_command(1)
        probe_flush()
        engine.stop()
        if let engineObserver { NotificationCenter.default.removeObserver(engineObserver) }
        audio = ProbeAudioGraph(render: Self.render)
        configureSession()
        observeEngine()
        updateNowPlaying()
        onChange?()
    }

    /// All screen and remote playback requests pass through the reset gate.
    @discardableResult
    func command(_ command: UInt32) -> Int32 {
        if command == 0 || command == 2 || command == 3 || command == 4 {
            awaitingPlaybackRequest = false
        }
        return probe_command(command)
    }

    func load(_ uri: String) -> Int32 {
        awaitingPlaybackRequest = false
        return probe_load(uri)
    }

    /// Runs on Core Audio's real-time thread. Not main-actor isolated.
    private nonisolated static let render: AVAudioSourceNodeRenderBlock = { _, _, frameCount, bufferList in
        let buffers = UnsafeMutableAudioBufferListPointer(bufferList)
        guard buffers.count >= 2,
              let left = buffers[0].mData?.assumingMemoryBound(to: Float.self),
              let right = buffers[1].mData?.assumingMemoryBound(to: Float.self)
        else { return noErr }
        _ = probe_render(left, right, Int(frameCount))
        return noErr
    }

    private func activateAndRun(reason: String) {
        guard !awaitingPlaybackRequest else { return }
        wantsEngine = true
        do {
            try AVAudioSession.sharedInstance().setActive(true)
            if !engine.isRunning { try engine.start() }
            record(["t": "audio_running", "reason": reason])
        } catch {
            record(["t": "error", "stage": "activate", "reason": reason, "msg": error.localizedDescription])
        }
    }

    private func observeSession() {
        let center = NotificationCenter.default
        center.addObserver(forName: AVAudioSession.interruptionNotification, object: nil, queue: .main) { note in
            let info = note.userInfo ?? [:]
            let type = (info[AVAudioSessionInterruptionTypeKey] as? UInt).flatMap(AVAudioSession.InterruptionType.init)
            let options = AVAudioSession.InterruptionOptions(rawValue: info[AVAudioSessionInterruptionOptionKey] as? UInt ?? 0)
            let reason = info[AVAudioSessionInterruptionReasonKey] as? UInt
            MainActor.assumeIsolated { Probe.shared.interruption(type, shouldResume: options.contains(.shouldResume), reason: reason) }
        }
        center.addObserver(forName: AVAudioSession.routeChangeNotification, object: nil, queue: .main) { note in
            let reason = (note.userInfo?[AVAudioSessionRouteChangeReasonKey] as? UInt).flatMap(AVAudioSession.RouteChangeReason.init)
            MainActor.assumeIsolated { Probe.shared.routeChanged(reason) }
        }
        observeEngine()
        center.addObserver(forName: AVAudioSession.mediaServicesWereResetNotification, object: nil, queue: .main) { _ in
            MainActor.assumeIsolated { Probe.shared.mediaServicesReset() }
        }
        for (name, label) in [
            (UIApplication.didEnterBackgroundNotification, "background"),
            (UIApplication.willEnterForegroundNotification, "foreground"),
            (UIApplication.protectedDataWillBecomeUnavailableNotification, "device_locking"),
            (UIApplication.protectedDataDidBecomeAvailableNotification, "device_unlocked"),
        ] {
            center.addObserver(forName: name, object: nil, queue: .main) { _ in
                MainActor.assumeIsolated {
                    Probe.shared.record(["t": "app", "event": label])
                    if label == "background" { Probe.shared.wasBackgrounded = true }
                    if label == "foreground" { Probe.shared.refreshSessionIfIdle() }
                }
            }
        }
    }

    private func observeEngine() {
        engineObserver = NotificationCenter.default.addObserver(forName: .AVAudioEngineConfigurationChange, object: engine, queue: .main) { note in
            MainActor.assumeIsolated {
                let probe = Probe.shared
                guard let changed = note.object as? AVAudioEngine, changed === probe.engine else { return }
                probe.record(["t": "engine_configuration_change", "running": probe.engine.isRunning])
                if probe.wantsEngine, !probe.engine.isRunning { probe.activateAndRun(reason: "configuration_change") }
            }
        }
    }

    private func interruption(_ type: AVAudioSession.InterruptionType?, shouldResume: Bool, reason: UInt?) {
        switch type {
        case .began:
            interruptionPlayback.began(playing: playing)
            record(["t": "interruption_began", "was_playing": playing, "reason": reason ?? 0])
            probe_flush()
            if playing { probe_command(1) }
        case .ended:
            record(["t": "interruption_ended", "should_resume": shouldResume])
            if interruptionPlayback.ended(shouldResume: shouldResume) {
                activateAndRun(reason: "interruption_ended")
                probe_command(0)
            }
        default:
            break
        }
    }

    private func routeChanged(_ reason: AVAudioSession.RouteChangeReason?) {
        record(["t": "route_change", "reason": reason.map { "\($0.rawValue)" } ?? "?", "outputs": outputs()])
        // Apple's playback convention: pause when headphones are removed.
        if reason == .oldDeviceUnavailable, playing {
            probe_command(1)
        }
    }

    private func outputs() -> [String] {
        AVAudioSession.sharedInstance().currentRoute.outputs.map(\.portType.rawValue)
    }

    // MARK: - Engine events

    private func engineEvent(_ line: String) {
        guard let data = line.data(using: .utf8),
              var event = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any],
              let type = event["t"] as? String
        else { return }
        switch type {
        case "connected":
            connected = true
            lastError = ""
            // `devicectl ... process launch ... -autoplay <uri>` starts playback
            // over the cable, so diagnosis does not need taps on the phone.
            let arguments = ProcessInfo.processInfo.arguments
            if !autoplayed, let index = arguments.firstIndex(of: "-autoplay"), index + 1 < arguments.count {
                autoplayed = true
                let uri = arguments[index + 1]
                record(["t": "load_requested", "uri": uri, "result": Int(probe_load(uri)), "via": "launch_argument"])
            }
        case "session_ended":
            connected = false
            playing = false
            idleSince = idleSince ?? Date()
        case "track":
            track = "\(event["name"] as? String ?? "") — \(event["artists"] as? String ?? "")"
            durationMs = event["duration_ms"] as? UInt32 ?? 0
        case "playing":
            if awaitingPlaybackRequest {
                probe_command(1)
                probe_flush()
                break
            }
            idleSince = nil
            playing = true
            positionMs = event["position_ms"] as? UInt32 ?? positionMs
            activateAndRun(reason: "playing")
        case "paused", "stopped":
            idleSince = idleSince ?? Date()
            playing = false
            positionMs = event["position_ms"] as? UInt32 ?? positionMs
        case "position", "seeked":
            positionMs = event["position_ms"] as? UInt32 ?? positionMs
        case "error":
            lastError = "\(event["stage"] as? String ?? "error"): \(event["msg"] as? String ?? "")"
            if event["stage"] as? String == "connect", CredentialStore.load() != nil {
                event["note"] = "stored credential rejected or network failed"
            }
        default:
            break
        }
        record(event.merging(stats(), uniquingKeysWith: { a, _ in a }))
        updateNowPlaying()
        onChange?()
    }

    // MARK: - Lock screen and remote commands

    private func configureRemoteCommands() {
        let commands = MPRemoteCommandCenter.shared()
        commands.playCommand.addTarget { _ in MainActor.assumeIsolated { Probe.shared.command(0) == 0 ? .success : .commandFailed } }
        commands.pauseCommand.addTarget { _ in MainActor.assumeIsolated { Probe.shared.command(1) == 0 ? .success : .commandFailed } }
        commands.togglePlayPauseCommand.addTarget { _ in
            MainActor.assumeIsolated { Probe.shared.command(Probe.shared.playing ? 1 : 0) == 0 ? .success : .commandFailed }
        }
        commands.nextTrackCommand.addTarget { _ in MainActor.assumeIsolated { Probe.shared.command(2) == 0 ? .success : .commandFailed } }
        commands.previousTrackCommand.addTarget { _ in MainActor.assumeIsolated { Probe.shared.command(3) == 0 ? .success : .commandFailed } }
    }

    private func updateNowPlaying() {
        MPNowPlayingInfoCenter.default().nowPlayingInfo = [
            MPMediaItemPropertyTitle: track,
            MPMediaItemPropertyArtist: "Spotiurge Probe",
            MPMediaItemPropertyPlaybackDuration: Double(durationMs) / 1000,
            MPNowPlayingInfoPropertyElapsedPlaybackTime: Double(positionMs) / 1000,
            MPNowPlayingInfoPropertyPlaybackRate: playing ? 1.0 : 0.0,
        ]
        MPNowPlayingInfoCenter.default().playbackState = playing ? .playing : .paused
    }

    // MARK: - Evidence

    func stats() -> [String: Any] {
        let stats = probe_stats()
        return [
            "decoded_frames": stats.decoded_frames,
            "rendered_frames": stats.rendered_frames,
            "silent_frames": stats.silent_frames,
            "rms": (Double(stats.last_rms) * 10_000).rounded() / 10_000,
        ]
    }

    func statusText() -> String {
        let stats = probe_stats()
        let seconds = Double(stats.rendered_frames) / 44_100
        return """
        \(connected ? "Connected to Spotify" : "Not connected")  ·  \(playing ? "playing" : "idle")
        \(track.isEmpty ? "No track" : track)
        position \(positionMs / 1000)s · decoded audio heard \(Int(seconds))s · RMS \(String(format: "%.4f", stats.last_rms))
        route \(outputs().joined(separator: ", ")) · engine \(engine.isRunning ? "running" : "stopped")
        \(lastError.isEmpty ? "" : "Last error: \(lastError)")
        """
    }

    private func heartbeat() {
        // ponytail: fixed five-minute idle stop; a shipping app would tie this
        // to Connect reachability policy.
        if !playing, wantsEngine, let idleSince, Date().timeIntervalSince(idleSince) > 300 {
            wantsEngine = false
            engine.stop()
            try? AVAudioSession.sharedInstance().setActive(false, options: .notifyOthersOnDeactivation)
            record(["t": "audio_stopped", "reason": "idle_5_min"])
        }
        let session = AVAudioSession.sharedInstance()
        var beat: [String: Any] = [
            "t": "heartbeat",
            "app_state": ["active", "inactive", "background"][UIApplication.shared.applicationState.rawValue],
            "protected_data_available": UIApplication.shared.isProtectedDataAvailable,
            "engine_running": engine.isRunning,
            "category": session.category.rawValue,
            "other_audio_playing": session.isOtherAudioPlaying,
            "outputs": outputs(),
            "connected": connected,
            "playing": playing,
            "position_ms": positionMs,
        ]
        beat.merge(stats(), uniquingKeysWith: { a, _ in a })
        record(beat)
        onChange?()
    }

    func record(_ event: [String: Any]) {
        var event = event
        event["ts"] = ISO8601DateFormatter().string(from: Date())
        guard var line = try? JSONSerialization.data(withJSONObject: event, options: [.sortedKeys]) else { return }
        line.append(0x0A)
        if let handle = try? FileHandle(forWritingTo: evidenceURL) {
            handle.seekToEndOfFile()
            handle.write(line)
            try? handle.close()
        } else {
            try? line.write(to: evidenceURL, options: .completeFileProtectionUntilFirstUserAuthentication)
        }
    }
}
