import Foundation
import Security

@main
struct SignInSupportTests {
    @MainActor
    static func main() throws {
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
        print("PASS: fresh stopped audio graph, interruption resume intent, fragmented callback, request bounds, failure preservation, and disposable native Keychain round trip")
    }
}
