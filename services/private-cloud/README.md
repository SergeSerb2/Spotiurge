# Spotiurge private cloud

One personal Railway service with a persistent `/data` SQLite volume. The client
keeps local snapshots; the service stores only a revisioned document and brokers
AI requests through the existing subscription-backed CLIProxyAPI deployment.
No telemetry, browser, Spotify grant, audio storage or direct-provider fallback.

Required Railway variables: `SPOTIURGE_CLOUD_TOKEN` (random, 32–256 ASCII characters),
`CLI_PROXY_BASE_URL` (existing HTTPS proxy's `/v1` URL), `CLI_PROXY_API_KEY`,
`SPOTIURGE_DATA_DIR=/data`. Recommendations use **`gpt-6-luna` only**, with no
model fallback. Legacy `SPOTIURGE_MODELS` values are ignored. Do not alter the existing
CLIProxyAPI service. Deploy this directory as the Docker build root, one replica,
with the volume mounted. The checked-in `railway.json` is supported by the
current deployment but Railway has deprecated it for December 1, 2026; migrate
to Infrastructure as Code before that date. TLS is terminated by Railway. No secret belongs in the
image, repository, deploy command or logs.

`GET /health` is public and returns no private data. All `/v1` endpoints require
the bearer token. `GET /v1/state` returns `{revision, document}`.
`PUT /v1/state` requires `If-Match: revision`; a conflict returns 409 for the
client to refetch and merge. A committed write increments the revision.
SQLite commits are durable on the volume. Recommendations use
`POST /v1/recommendations` with taste, intentional feedback and an allowlisted
`exploration` value (`familiar`, `balanced`, or `adventurous`). Either taste or at
least one feedback record is required. Spotify URIs are removed before prompting;
canonical titles and primary artists are requested. Repeat filtering recognizes
primary and guest credits within Spotify's comma-separated feedback credits,
including featured-artist title groups and trailing credits that name actual
credited artists. Unknown guests, distinct versions and different credited
artists remain distinct.
A 429 includes a bounded `code`: `busy` for another active
request, `rate_limited` for the subscription provider's quota. One AI request
runs at a time, with a bounded Luna proxy call and no model fallback; a proxy
429 is surfaced without another model attempt.

The document is versioned and limited to 1 MiB of compact UTF-8 JSON and 2000
records. Validation, persistence and state responses use this same encoding,
so whitespace or Unicode escaping cannot make a valid client document fail.
Client snapshots
also carry up to 12 local catalogue matches and are limited to 2 MiB. Unknown
document schemas and malformed clocks fail closed. Record deletions are retained
as tombstones. Clients keep at most ten live AI history entries by reusing the
oldest `history:` key after ten exist, and tombstone older live entries in the
same write; the server schema is unchanged. Clients retain the newest 500
feedback records, including cleared ratings. A
`feedback:retention` tombstone stores the highest forgotten logical stamp; merges
discard feedback at or below it, preventing old offline snapshots from
resurrecting discarded ratings. Equal-clock cohorts are discarded together, so
fewer than 500 may remain. A new rating advances above the cutoff. Older client
versions do not enforce this bound and should be upgraded. There is no mix
tombstone compaction;
storage limits produce an error and keep prior state. Before routine use, add
export/deletion controls and a retention policy for mixes.

For an export, use an authenticated GET through a tool that reads its token from
protected storage, save the response to an owner-only file, and back up the volume
using Railway's volume backup facilities. No automated backup schedule is
configured by this slice. Restore requires a stopped service and the consistent
SQLite file. To revoke pairing, rotate `SPOTIURGE_CLOUD_TOKEN` in Railway secrets
and re-pair every device. Server access includes plaintext personal preferences
and feedback; the volume is private and HTTPS protects transport, but this is
not end-to-end encryption.

Run `python3 -m unittest discover -s services/private-cloud -v` from the repository
root. Tests use temporary SQLite files inside the service directory and delete
them afterward. `examples/discovery_probe.rs` exercises the production Rust
merge, TLS and native credential path for two-device verification. It never
prints the pairing token. Synthetic protocol/device checks are not proof of
three-platform application sync, recommendation quality or iPhone playback.

Proxy HTTP errors close their response streams before returning or propagating
a rate-limit result. Busy responses still mean that another device holds the
single AI slot; clients retry after 15 seconds without increasing model-error
backoff. Model requests remain limited to `gpt-6-luna` through CLIProxyAPI.
