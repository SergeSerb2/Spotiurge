import Foundation
import Security

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
        guard states.count == 1, states[0].value == state,
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

    static func delete() {
        SecItemDelete(query as CFDictionary)
    }
}
