// Strict catalogue matching for AI suggestions, ported from src/backend.rs
// (`fold_name`, `credits_only`, `without_credits`, `discovery_matches`,
// `discovery_search_term`, `resolve_discovery`). Only an exact, available
// match becomes playable; nothing is fuzzy.

import Foundation

/// Fold presentation-only differences: whitespace, case, typographic
/// apostrophes, quotes and dashes. Meaningful punctuation stays.
func foldName(_ text: String) -> String {
    let mapped = String(String.UnicodeScalarView(text.unicodeScalars.map { scalar -> Unicode.Scalar in
        switch scalar.value {
        case 0x2018, 0x2019, 0x02BC, 0x60, 0xB4: "'"
        case 0x201C, 0x201D: "\""
        case 0x2010...0x2015: "-"
        default: scalar
        }
    }))
    return mapped.lowercased().split(whereSeparator: \.isWhitespace).joined(separator: " ")
}

private let creditSeparators = [
    ", and ", ", ", " & ", " and ", " featuring ", " feat. ", " feat ", " ft. ", " ft ",
    " with ", " vs. ", " vs ", " x ", " + ", " / ",
]

/// Whether `rest` consists only of the track's credited artists joined by
/// separators, with `accept` approving the set of named artists. Backtracks
/// over the actual names, so "Simon & Garfunkel" parses as one artist when
/// that is how Spotify credits them.
func creditsOnly(_ rest: Substring, _ names: [String], _ named: UInt64, _ accept: (UInt64) -> Bool) -> Bool {
    names.prefix(64).enumerated().contains { index, name in
        let bit = UInt64(1) << UInt64(index)
        guard named & bit == 0, !name.isEmpty, rest.hasPrefix(name) else { return false }
        let after = rest.dropFirst(name.count)
        let named = named | bit
        if after.isEmpty && accept(named) { return true }
        return creditSeparators.contains { separator in
            after.hasPrefix(separator) && creditsOnly(after.dropFirst(separator.count), names, named, accept)
        }
    }
}

/// Drop "(feat. X)"/"[with X]" groups and a trailing "feat. X" only when X
/// names credited artists. Remix, Edit, Live and other versions remain.
func withoutCredits(_ title: String, _ names: [String]) -> String {
    let credited = { (credit: Substring) in creditsOnly(credit, names, 0) { _ in true } }
    var title = title
    var from = title.startIndex
    while let open = title[from...].firstIndex(where: { $0 == "(" || $0 == "[" }) {
        let close: Character = title[open] == "(" ? ")" : "]"
        guard let end = title[open...].firstIndex(of: close) else { break }
        let inner = title[title.index(after: open)..<end]
        if ["feat. ", "feat ", "ft. ", "ft ", "featuring ", "with "].contains(where: { inner.hasPrefix($0) && credited(inner.dropFirst($0.count)) }) {
            let offset = title.distance(from: title.startIndex, to: open)
            title.removeSubrange(open...end)
            from = title.index(title.startIndex, offsetBy: offset)
        } else {
            from = title.index(after: end)
        }
    }
    for marker in [" feat. ", " feat ", " ft. ", " ft ", " featuring "] {
        if let range = title.range(of: marker, options: .backwards), credited(title[range.upperBound...]) {
            title = String(title[..<range.lowerBound])
        }
    }
    return title.split(whereSeparator: \.isWhitespace).joined(separator: " ")
}

/// Same title and version, and an artist credit made only of the track's
/// actual artists that includes its primary artist.
public func discoveryMatches(_ suggestion: Suggestion, _ track: Track) -> Bool {
    let names = track.artists.map(foldName)
    return track.isPlayable != false
        && isSpotifyTrack(track.uri)
        && creditsOnly(Substring(foldName(suggestion.artist)), names, 0) { $0 & 1 != 0 }
        && withoutCredits(foldName(suggestion.title), names) == withoutCredits(foldName(track.name), names)
}

/// Free-text search favours recall; `discoveryMatches` decides correctness.
public func discoverySearchTerm(_ suggestion: Suggestion) -> String {
    let artist = foldName(suggestion.artist)
    let primary = [", ", " feat", " ft. ", " ft ", " featuring ", " with ", " x ", " vs"]
        .compactMap { artist.range(of: $0)?.lowerBound }
        .min()
        .map { String(artist[..<$0]) } ?? artist
    return "\(suggestion.title) \(primary)".replacingOccurrences(of: "\"", with: "")
}

public enum CatalogueOutcome: String, Codable, Sendable {
    case complete, timedOut, rateLimited, quotaExhausted, unavailable, signInNeeded
}

public enum CatalogueError: Error, Sendable {
    case rateLimited, signInNeeded, unavailable
}

/// Keep AI suggestions when Spotify is slow; only exact, available matches
/// become playable. One deadline bounds all searches. Only unchecked picks
/// are searched, so a retry keeps earlier matches and order. The first error
/// stops searching: more requests would only extend Spotify's cooldown.
public func resolveDiscovery(
    _ picks: [Pick],
    budget: Duration,
    search: @escaping @Sendable (Suggestion) async throws(CatalogueError) -> [Track]
) async -> (picks: [Pick], outcome: CatalogueOutcome) {
    var picks = picks
    for index in picks.indices where picks[index].track != nil {
        picks[index].checked = true
    }
    let start = picks
    let work = Task { () -> (picks: [Pick], outcome: CatalogueOutcome) in
        var picks = start
        var seen = Set(picks.compactMap { $0.track?.uri })
        for index in picks.indices where !picks[index].checked {
            if Task.isCancelled { return (picks, .timedOut) }
            do throws(CatalogueError) {
                let tracks = try await search(picks[index].suggestion)
                if Task.isCancelled { return (picks, .timedOut) }
                picks[index].track = tracks.first { discoveryMatches(picks[index].suggestion, $0) && seen.insert($0.uri).inserted }
                picks[index].checked = true
            } catch {
                if Task.isCancelled { return (picks, .timedOut) }
                let outcome: CatalogueOutcome = switch error {
                case .rateLimited: .rateLimited
                case .signInNeeded: .signInNeeded
                case .unavailable: .unavailable
                }
                return (picks, outcome)
            }
        }
        return (picks, .complete)
    }
    let timer = Task {
        try? await Task.sleep(for: budget)
        work.cancel()
    }
    let result = await work.value
    timer.cancel()
    return result
}
