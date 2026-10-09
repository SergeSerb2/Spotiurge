import Foundation
import Network
import Testing
import Darwin
@testable import Spotiurge

struct LoopbackListenerTests {
    @Test @MainActor
    func listenerReadiness() async throws {
        let bound = LoopbackSignInListener()
        defer { bound.cancel() }
        var opened = 0, accepted = 0, failures = 0
        bound.start(port: 0, onReady: { opened += 1 }, onConnection: {
            accepted += 1
            $0.cancel()
        }, onFailure: { failures += 1 })
        #expect(opened == 0, "Authentication opened before asynchronous listener readiness")
        try await waitUntil { opened + failures > 0 }
        #expect(opened == 1 && failures == 0)
        let port = try #require(bound.port)
        #expect(port != 0)
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
        #expect(secondOpened == 0 && secondFailed == 1 && occupied.port == nil)

        let replacement = LoopbackSignInListener()
        defer { replacement.cancel() }
        var stale = 0, replacementOpened = 0, replacementFailed = 0
        replacement.start(port: 0, onReady: { stale += 1 }, onConnection: { $0.cancel() }, onFailure: { stale += 1 })
        replacement.cancel()
        replacement.start(port: 0, onReady: { replacementOpened += 1 }, onConnection: { $0.cancel() },
                          onFailure: { replacementFailed += 1 })
        try await waitUntil { replacementOpened + replacementFailed > 0 }
        try await Task.sleep(for: .milliseconds(100))
        #expect(stale == 0 && replacementOpened == 1 && replacementFailed == 0)
        #expect(opened == 1 && secondFailed == 1)
    }

    @Test @MainActor
    func reusableCallbackPortCannotBeShared() async throws {
        // Another process may opt into SO_REUSEADDR/SO_REUSEPORT. OAuth must
        // fail before opening authentication rather than share that socket.
        let socket = Darwin.socket(AF_INET, SOCK_STREAM, 0)
        try #require(socket >= 0)
        defer { Darwin.close(socket) }
        var reuse: Int32 = 1
        try #require(setsockopt(socket, SOL_SOCKET, SO_REUSEADDR, &reuse, socklen_t(MemoryLayout.size(ofValue: reuse))) == 0)
        try #require(setsockopt(socket, SOL_SOCKET, SO_REUSEPORT, &reuse, socklen_t(MemoryLayout.size(ofValue: reuse))) == 0)
        var address = sockaddr_in()
        address.sin_len = UInt8(MemoryLayout<sockaddr_in>.size)
        address.sin_family = sa_family_t(AF_INET)
        address.sin_addr.s_addr = inet_addr("127.0.0.1")
        var length = socklen_t(MemoryLayout.size(ofValue: address))
        let boundSocket = withUnsafeMutablePointer(to: &address) {
            $0.withMemoryRebound(to: sockaddr.self, capacity: 1) { Darwin.bind(socket, $0, length) }
        }
        try #require(boundSocket == 0 && Darwin.listen(socket, 1) == 0)
        try #require(withUnsafeMutablePointer(to: &address) {
            $0.withMemoryRebound(to: sockaddr.self, capacity: 1) { getsockname(socket, $0, &length) }
        } == 0)
        let exclusive = LoopbackSignInListener()
        defer { exclusive.cancel() }
        var opened = 0, failed = 0
        exclusive.start(port: UInt16(bigEndian: address.sin_port), onReady: { opened += 1 },
                        onConnection: { $0.cancel() }, onFailure: { failed += 1 })
        try await waitUntil { opened + failed > 0 }
        #expect(opened == 0 && failed == 1 && exclusive.port == nil)
    }

    @MainActor
    private func waitUntil(_ condition: () -> Bool) async throws {
        let end = ContinuousClock.now + .seconds(3)
        while !condition() {
            try #require(ContinuousClock.now < end, "Loopback regression timed out")
            try await Task.sleep(for: .milliseconds(10))
        }
    }
}
