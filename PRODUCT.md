# Spotiurge

<!-- impeccable:product-schema 1 -->

## Platform

adaptive

## Users

Serge is the only user. He discovers music, listens across his Mac, Windows
computer and iPhone, and gives feedback to improve future recommendations.

## Product Purpose

A standalone personal fork of Spotifast, centered on personalized discovery
and, later, a spoken AI DJ. Recommendation quality is evaluated through real
listening feedback; parity with Spotify is a goal, not a measured result.

## Capabilities and Constraints

Keep Rust, native rendering, asynchronous playback and Spotify Connect on
desktop. No browser engine, telemetry, substitute catalogue or DRM bypass.
Use existing CLIProxyAPI subscriptions for AI. Serge reports permission for
broader Spotify inputs to AI. Document exactly what is sent and keep grants out
of model prompts and sync. Use a small private service in Serge's existing
Railway workspace. The iOS architecture remains undecided until independent
background Spotify playback is demonstrated on a real iPhone. Serge deferred
iOS work on October 6, 2026 and prioritized macOS; preserve that gate for later.

## Brand Commitments

Spotiurge has independent branding and releases. Preserve the MIT license,
Spotifast attribution and copyright notices. The requested interface uses
legible liquid/frosted glass, thoughtful typography and platform interactions.

## Product Principles

- Playback remains responsive when AI or sync fails.
- Local edits work offline and merge predictably after reconnecting.
- Credentials stay in native protected storage or server secrets.
- Measured platform evidence determines claims and architecture choices.

## Evidence on Hand

The inherited desktop application, deterministic demo captures, real Mac
decoded playback and Mac/Windows synchronization through the deployed private
store. No Spotiurge iPhone playback proof, TestFlight build or listening-quality
study exists yet. See `docs/reviews/spotiurge-discovery/verification.md` for
tested behavior and `docs/_reference/spotiurge-architecture.md` for the audit.
