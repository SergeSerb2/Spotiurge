import Foundation
import Network
import Security
import Darwin

@main
struct SignInSupportTests {
    @MainActor
    static func main() async throws {
        try await listenerReadiness()
        let began = Date(timeIntervalSince1970: 1_000)
        var playback = PlaybackStatus()
        playback.receive("connected", at: began)
        playback.receive("playing", at: began)
        precondition(playback.connected && playback.playing && playback.idleSince == nil)
        // A command failure does not invalidate an otherwise live session.
        playback.receive("error", stage: "command", at: began)
        precondition(playback.connected && playback.playing)
        let dropped = began.addingTimeInterval(30)
        playback.receive("reconnecting", at: dropped)
        precondition(!playback.connected && !playback.playing && playback.idleSince == dropped)
        for delay in [1.0, 3.0, 7.0, 15.0, 31.0, 63.0] {
            playback.receive("reconnect_failed", at: dropped.addingTimeInterval(delay))
            precondition(!playback.connected && !playback.playing && playback.idleSince == dropped)
        }
        playback.receive("error", stage: "connect", at: dropped.addingTimeInterval(64))
        precondition(!playback.connected && !playback.playing && playback.idleSince == dropped)
        let recovered = dropped.addingTimeInterval(65)
        playback.receive("connected", at: recovered)
        precondition(playback.connected && !playback.playing && playback.idleSince == dropped)
        playback.receive("playing", at: recovered)
        precondition(playback.connected && playback.playing && playback.idleSince == nil)
        playback.receive("error", stage: "connect", at: recovered.addingTimeInterval(1))
        precondition(!playback.connected && !playback.playing)
        // Paused sessions remain connected; idle time resets only on playback.
        playback.receive("connected", at: recovered)
        playback.receive("playing", at: recovered)
        playback.receive("paused", at: recovered)
        precondition(playback.connected && !playback.playing && playback.idleSince == recovered)
        playback.receive("session_ended", at: recovered.addingTimeInterval(2))
        precondition(!playback.connected && !playback.playing && playback.idleSince == recovered)
        playback.receive("connected", at: recovered)
        playback.receive("playing", at: recovered)
        playback.receive("reconnect_failed", at: recovered)
        precondition(!playback.connected && !playback.playing && playback.idleSince == recovered)
        do {
            _ = try SecureVerifier.make(fill: { bytes in
                bytes = [UInt8](repeating: 7, count: bytes.count)
                return errSecNotAvailable
            })
            preconditionFailure("Failed randomness produced a verifier")
        } catch SecureVerifier.Failure.randomnessUnavailable {}
        let verifier = try SecureVerifier.make(fill: { bytes in
            bytes = [UInt8](repeating: 255, count: bytes.count)
            return errSecSuccess
        })
        precondition(verifier.count == 64 && verifier.allSatisfy { $0 == "_" })
        let nativeVerifier = try SecureVerifier.make()
        precondition(nativeVerifier.count == 64)
        let gate = CredentialCallbackGate()
        precondition(gate.ticket() == nil)
        gate.begin()
        let pending = gate.ticket()!
        gate.revoke()
        precondition(gate.ticket() == nil && !gate.accepts(pending))
        gate.begin()
        precondition(!gate.accepts(pending) && gate.accepts(gate.ticket()!))
        for result: Int32 in [-1, -2, -3, 1] {
            gate.begin()
            let rejected = gate.ticket()!
            precondition(!gate.connectionStarted(result))
            precondition(gate.ticket() == nil && !gate.accepts(rejected))
            gate.begin()
            precondition(!gate.accepts(rejected) && gate.connectionStarted(0))
            precondition(gate.accepts(gate.ticket()!))
        }
        let original = ProbeAudioGraph(render: { _, _, _, _ in 0 })
        let replacement = ProbeAudioGraph(render: { _, _, _, _ in 0 })
        precondition(original.engine !== replacement.engine)
        precondition(original.source !== replacement.source)
        precondition(!original.engine.isRunning && !replacement.engine.isRunning)
        precondition(replacement.engine.attachedNodes.contains(replacement.source))
        var interruption = InterruptionPlayback()
        precondition(!interruption.ended(shouldResume: true))
        interruption.began(playing: false)
        precondition(!interruption.ended(shouldResume: true))
        interruption.began(playing: true)
        precondition(!interruption.ended(shouldResume: false))
        interruption.began(playing: true)
        precondition(interruption.ended(shouldResume: true))
        precondition(!interruption.ended(shouldResume: true))
        let line = "GET /login?code=dummy&state=test-state HTTP/1.1\r\n"
        for split in 0...line.utf8.count {
            let bytes = Data(line.utf8)
            var request = LoopbackRequest()
            let first = try request.append(bytes.prefix(split))
            if split < bytes.count { precondition(first == nil) }
            let complete = try first ?? request.append(bytes.suffix(bytes.count - split))
            precondition(complete.map { LoopbackRequest.redirect($0, state: "test-state") } == .code("dummy"))
        }
        var singleBytes = LoopbackRequest()
        for byte in line.utf8 { _ = try singleBytes.append(Data([byte])) }
        let fullLine = try singleBytes.append(Data())
        precondition(fullLine == "GET /login?code=dummy&state=test-state HTTP/1.1")
        precondition(LoopbackRequest.redirect("GET /favicon.ico HTTP/1.1", state: "x") == .stray)
        precondition(LoopbackRequest.redirect("GET /login-extra?code=x&state=x HTTP/1.1", state: "x") == .stray)
        precondition(LoopbackRequest.redirect("POST /login?code=x&state=x HTTP/1.1", state: "x") == .stray)
        precondition(LoopbackRequest.redirect("GET /login?code=x&state=wrong HTTP/1.1", state: "x") == .stray)
        precondition(LoopbackRequest.redirect("GET /login?code=x&state=x&state=x HTTP/1.1", state: "x") == .stray)
        precondition(LoopbackRequest.redirect("GET /login?code=x HTTP/1.1", state: "x") == .stray)
        precondition(LoopbackRequest.redirect("GET /login?error=access_denied&state=old HTTP/1.1", state: "x") == .stray)
        precondition(LoopbackRequest.redirect("GET /login?error=access_denied&state=x HTTP/1.1", state: "x") == .refused)
        precondition(LoopbackRequest.redirect("GET /login?error=access_denied&code=x&state=x HTTP/1.1", state: "x") == .refused)
        precondition(LoopbackRequest.redirect("GET /login?code=next&state=x HTTP/1.1", state: "x") == .code("next"))
        var oversized = LoopbackRequest()
        do {
            _ = try oversized.append(Data(repeating: 65, count: 16_385))
            preconditionFailure("Oversized callback was accepted")
        } catch {}

        let old = Data("old dummy grant".utf8)
        var held = old
        var addCalls = 0
        let status = CredentialStore.save(Data("new dummy grant".utf8), update: { _, _ in errSecInteractionNotAllowed }, add: { _, _ in
            addCalls += 1
            held = Data()
            return errSecSuccess
        })
        precondition(status == errSecInteractionNotAllowed && addCalls == 0 && held == old)
        let missing = CredentialStore.save(old, update: { _, _ in errSecItemNotFound }, add: { _, _ in errSecMissingEntitlement })
        precondition(missing == errSecMissingEntitlement)

        // A disposable namespace, never the phone's actual playback item.
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: "com.sergeserbinenko.spotiurge.signin-test",
            kSecAttrAccount as String: UUID().uuidString,
        ]
        defer { SecItemDelete(query as CFDictionary) }
        precondition(CredentialStore.save(old, query: query) == errSecSuccess)
        let new = Data("replacement dummy grant".utf8)
        precondition(CredentialStore.save(new, query: query) == errSecSuccess)
        var read = query
        read[kSecReturnData as String] = true
        var value: CFTypeRef?
        precondition(SecItemCopyMatching(read as CFDictionary, &value) == errSecSuccess)
        precondition(value as? Data == new)
        let current = gate.ticket()!
        gate.revoke()
        precondition(CredentialStore.delete(query: query) == errSecSuccess)
        if gate.accepts(current) {
            _ = CredentialStore.save(new, query: query)
        }
        precondition(SecItemCopyMatching(read as CFDictionary, &value) == errSecItemNotFound)
        gate.begin()
        if gate.accepts(current) { _ = CredentialStore.save(new, query: query) }
        precondition(SecItemCopyMatching(read as CFDictionary, &value) == errSecItemNotFound)
        precondition(CredentialStore.delete(query: query, remove: { _ in errSecInteractionNotAllowed }) == errSecInteractionNotAllowed)
        print("PASS: real loopback readiness/exclusive-bind/occupied-port/canceled-replacement checks, reconnect/terminal-error state and idle deadline, secure verifier failure, stale callback revocation, fresh stopped audio graph, interruption intent, fragmented callback, bounds, failure preservation, and disposable native Keychain round trip")
    }

    @MainActor
    private static func listenerReadiness() async throws {
        let bound = LoopbackSignInListener()
        defer { bound.cancel() }
        var opened = 0, accepted = 0, failures = 0
        bound.start(port: 0, onReady: { opened += 1 }, onConnection: {
            accepted += 1
            $0.cancel()
        }, onFailure: { failures += 1 })
        precondition(opened == 0, "Authentication opened before asynchronous listener readiness")
        try await waitUntil { opened + failures > 0 }
        precondition(opened == 1 && failures == 0)
        let port = bound.port!
        precondition(port != 0)
        let client = NWConnection(host: "127.0.0.1", port: NWEndpoint.Port(rawValue: port)!, using: .tcp)
        defer { client.cancel() }
        client.start(queue: .main)
        try await waitUntil { accepted == 1 }

        let occupied = LoopbackSignInListener()
        defer { occupied.cancel() }
        var secondOpened = 0, secondFailed = 0
        occupied.start(port: port, onReady: { secondOpened += 1 }, onConnection: { $0.cancel() },
                       onFailure: { secondFailed += 1 })
        try await waitUntil { secondOpened + secondFailed > 0 }
        precondition(secondOpened == 0 && secondFailed == 1 && occupied.port == nil)

        // A competing process may deliberately opt into SO_REUSEPORT. Our
        // OAuth listener must still refuse to share its callback socket.
        let socket = Darwin.socket(AF_INET, SOCK_STREAM, 0)
        precondition(socket >= 0)
        defer { Darwin.close(socket) }
        var reuse: Int32 = 1
        precondition(setsockopt(socket, SOL_SOCKET, SO_REUSEADDR, &reuse, socklen_t(MemoryLayout.size(ofValue: reuse))) == 0)
        precondition(setsockopt(socket, SOL_SOCKET, SO_REUSEPORT, &reuse, socklen_t(MemoryLayout.size(ofValue: reuse))) == 0)
        var address = sockaddr_in()
        address.sin_len = UInt8(MemoryLayout<sockaddr_in>.size)
        address.sin_family = sa_family_t(AF_INET)
        address.sin_addr.s_addr = inet_addr("127.0.0.1")
        var length = socklen_t(MemoryLayout.size(ofValue: address))
        let boundSocket = withUnsafeMutablePointer(to: &address) {
            $0.withMemoryRebound(to: sockaddr.self, capacity: 1) { Darwin.bind(socket, $0, length) }
        }
        precondition(boundSocket == 0 && Darwin.listen(socket, 1) == 0)
        precondition(withUnsafeMutablePointer(to: &address) {
            $0.withMemoryRebound(to: sockaddr.self, capacity: 1) { getsockname(socket, $0, &length) }
        } == 0)
        let exclusive = LoopbackSignInListener()
        defer { exclusive.cancel() }
        var sharedOpened = 0, sharedFailed = 0
        exclusive.start(port: UInt16(bigEndian: address.sin_port), onReady: { sharedOpened += 1 },
                        onConnection: { $0.cancel() }, onFailure: { sharedFailed += 1 })
        try await waitUntil { sharedOpened + sharedFailed > 0 }
        precondition(sharedOpened == 0 && sharedFailed == 1 && exclusive.port == nil,
                     "OAuth callback listener shared another process's reusable socket")

        let replacement = LoopbackSignInListener()
        defer { replacement.cancel() }
        var stale = 0, replacementOpened = 0, replacementFailed = 0
        replacement.start(port: 0, onReady: { stale += 1 }, onConnection: { $0.cancel() }, onFailure: { stale += 1 })
        replacement.cancel()
        replacement.start(port: 0, onReady: { replacementOpened += 1 }, onConnection: { $0.cancel() },
                          onFailure: { replacementFailed += 1 })
        try await waitUntil { replacementOpened + replacementFailed > 0 }
        try await Task.sleep(for: .milliseconds(100))
        precondition(stale == 0 && replacementOpened == 1 && replacementFailed == 0)
        precondition(opened == 1 && secondFailed == 1)
    }

    @MainActor
    private static func waitUntil(_ condition: () -> Bool) async throws {
        let end = ContinuousClock.now + .seconds(3)
        while !condition() {
            precondition(ContinuousClock.now < end, "Loopback regression timed out")
            try await Task.sleep(for: .milliseconds(10))
        }
    }
}
