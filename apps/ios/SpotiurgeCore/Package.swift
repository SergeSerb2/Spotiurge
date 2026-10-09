// swift-tools-version: 6.2
// Platform-neutral Spotiurge logic for the developmental iPhone app: the
// discovery document and its merge rules, catalogue matching, and the
// private-cloud and Spotify Web API clients. Mirrors src/discovery.rs,
// src/discovery_cloud.rs and the matching helpers in src/backend.rs.
// No UI, no Keychain, no audio: the app supplies tokens through closures.
import PackageDescription

let package = Package(
    name: "SpotiurgeCore",
    platforms: [.iOS(.v26), .macOS(.v15)],
    products: [.library(name: "SpotiurgeCore", targets: ["SpotiurgeCore"])],
    targets: [
        .target(name: "SpotiurgeCore"),
        .testTarget(name: "SpotiurgeCoreTests", dependencies: ["SpotiurgeCore"]),
    ]
)
