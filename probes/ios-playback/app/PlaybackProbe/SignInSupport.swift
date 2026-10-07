import Foundation
import Security

/// C callbacks copy data on runtime threads, then deliver it on the main actor.
/// A ticket binds that delivery to one connection, including across Forget
/// followed immediately by a new sign-in. All shared state is under this lock.
final class CredentialCallbackGate: @unchecked Sendable {
    private let lock = NSLock()
    private var generation: UInt64 = 0
    private var enabled = false

    func begin() {
        lock.lock(); defer { lock.unlock() }
        generation &+= 1
        enabled = true
    }

    func revoke() {
        lock.lock(); defer { lock.unlock() }
        generation &+= 1
        enabled = false
    }

    func ticket() -> UInt64? {
        lock.lock(); defer { lock.unlock() }
        return enabled ? generation : nil
    }

    func accepts(_ ticket: UInt64) -> Bool {
        lock.lock(); defer { lock.unlock() }
        return enabled && generation == ticket
    }
}

enum SecureVerifier {
    enum Failure: LocalizedError {
        case randomnessUnavailable
        var errorDescription: String? { "Secure sign-in randomness is unavailable. Try again." }
    }

    static func make(fill: (inout [UInt8]) -> OSStatus = { bytes in
        SecRandomCopyBytes(kSecRandomDefault, bytes.count, &bytes)
    }) throws -> String {
        var bytes = [UInt8](repeating: 0, count: 48)
        guard fill(&bytes) == errSecSuccess else { throw Failure.randomnessUnavailable }
        return Data(bytes).base64URL
    }
}

extension Data {
    var base64URL: String {
        base64EncodedString()
            .replacingOccurrences(of: "+", with: "-")
            .replacingOccurrences(of: "/", with: "_")
            .replacingOccurrences(of: "=", with: "")
    }
}

/// An ended notification cannot start music that was idle or already paused.
struct InterruptionPlayback {
    private var wasPlaying = false

    mutating func began(playing: Bool) { wasPlaying = playing }

    mutating func ended(shouldResume: Bool) -> Bool {
        defer { wasPlaying = false }
        return wasPlaying && shouldResume
    }
}

/// TCP may split the callback at any byte. Parse only a complete request line.
struct LoopbackRequest {
    private var bytes = Data()

    mutating func append(_ chunk: Data) throws -> String? {
        guard bytes.count + chunk.count <= 16_384 else {
            throw CocoaError(.fileReadTooLarge)
        }
        bytes.append(chunk)
        guard let end = bytes.range(of: Data([13, 10])) else { return nil }
        guard let line = String(data: bytes[..<end.lowerBound], encoding: .utf8) else {
            throw CocoaError(.fileReadInapplicableStringEncoding)
        }
        return line
    }

    enum Redirect: Equatable {
        case stray, refused, code(String)
    }

    static func redirect(_ line: String, state: String) -> Redirect {
        let parts = line.split(separator: " ", omittingEmptySubsequences: false)
        guard parts.count == 3, parts[0] == "GET",
              parts[2] == "HTTP/1.1" || parts[2] == "HTTP/1.0",
              parts[1].hasPrefix("/"),
              let url = URLComponents(string: "http://127.0.0.1\(parts[1])"),
              url.path == "/login" else { return .stray }
        let items = url.queryItems ?? []
        let states = items.filter { $0.name == "state" }
        let codes = items.filter { $0.name == "code" }
        // Old browser tabs and unbound local requests cannot end this attempt.
        guard states.count == 1, states[0].value == state else { return .stray }
        guard !items.contains(where: { $0.name == "error" }),
              codes.count == 1, let code = codes[0].value, !code.isEmpty else { return .refused }
        return .code(code)
    }
}

/// Device-only reusable credential, readable for reconnect after first unlock.
enum CredentialStore {
    static let query: [String: Any] = [
        kSecClass as String: kSecClassGenericPassword,
        kSecAttrService as String: "com.sergeserbinenko.spotiurge.playbackprobe",
        kSecAttrAccount as String: "librespot-reusable",
    ]

    static func load() -> Data? {
        var query = query
        query[kSecReturnData as String] = true
        var item: CFTypeRef?
        return SecItemCopyMatching(query as CFDictionary, &item) == errSecSuccess ? item as? Data : nil
    }

    /// Updating in place keeps the prior item intact if replacement fails.
    /// Tests inject failing operations; production always uses Security.framework.
    static func save(_ data: Data, query: [String: Any] = query,
                     update: (CFDictionary, CFDictionary) -> OSStatus = SecItemUpdate,
                     add: (CFDictionary, UnsafeMutablePointer<CFTypeRef?>?) -> OSStatus = SecItemAdd) -> OSStatus {
        let changes: [String: Any] = [
            kSecValueData as String: data,
            kSecAttrAccessible as String: kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly,
        ]
        let status = update(query as CFDictionary, changes as CFDictionary)
        guard status == errSecItemNotFound else { return status }
        return add(query.merging(changes) { _, new in new } as CFDictionary, nil)
    }

    static func delete(query: [String: Any] = query,
                       remove: (CFDictionary) -> OSStatus = SecItemDelete) -> OSStatus {
        remove(query as CFDictionary)
    }
}
