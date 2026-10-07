import Foundation

/// TCP may split the callback at any byte. Parse only a complete request line.
public struct LoopbackRequest: Sendable {
    public init() {}
    private var bytes = Data()

    public mutating func append(_ chunk: Data) throws -> String? {
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

    public enum Redirect: Equatable, Sendable {
        case stray, refused, code(String)
    }

    public static func redirect(_ line: String, state: String) -> Redirect {
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

