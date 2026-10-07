import Foundation
import Network
import Testing
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

    @MainActor
    private func waitUntil(_ condition: () -> Bool) async throws {
        let end = ContinuousClock.now + .seconds(3)
        while !condition() {
            try #require(ContinuousClock.now < end, "Loopback regression timed out")
            try await Task.sleep(for: .milliseconds(10))
        }
    }
}
