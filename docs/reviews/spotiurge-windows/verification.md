# Private Windows development delivery

Recorded October 7, 2026 after PR #2 merged as `c8f5c42`.

The x86_64 MSVC release was built natively on Windows 11 Pro Insider Preview
10.0.26340 over the existing direct LAN connection. Rust 1.98 used the committed
static-CRT configuration, `--locked --release --no-default-features --features
demo`. The executable's source is `8b60d5f`; subsequent changes only correct
packaging paths and setup instructions, and add verification evidence.
MilkDrop is omitted. Playback, Connect, discovery and other native visualizers
remain compiled in. No compiler flags or dependency pins changed.

Inno Setup 7.1.0's official tool installer was Authenticode-verified as valid,
with Pyrsys B.V. as its signer, before use. The resulting Spotiurge package is
an unsigned private development installer, not a tagged or public release.

## Verified on Windows

- Locked release compilation and embedded `ProductName=Spotiurge` passed.
- `dumpbin /dependents` found no MSVCP or VCRUNTIME DLL dependency.
- The actual installer installed, upgraded and removed its payload. Its
  executable hash matched the source payload; `--version` exited successfully.
- The independent installer GUID, shortcut and link-handler registration were
  checked. Existing Spotify/Spotifast registration values remained identical.
- A fixture in the per-user Spotiurge configuration directory survived both
  installs and removal. The fixture was then deleted; real settings were untouched.
- The distributed executable rendered Home with synthetic demo data in the
  existing interactive Windows user session, in both themes and sizes. Each
  capture exited successfully. The owned scheduled task was removed afterward.
- Copied artifacts and captures matched their Windows SHA-256 hashes. Private
  Drive readback matched the complete installer and executable byte counts and
  SHA-256 hashes; each file's only permission was the owner's account.

The local packaging test also exposed and fixed a PowerShell path issue:
`.NET`'s process working directory differed from PowerShell's location. Relative
output/test paths now resolve from `Get-Location` before becoming absolute.

The installer lifecycle regression runs in the existing Windows CI job.
Local formatting, five launcher packaging tests, four release-name tests and
18 private-cloud tests passed. The full platform matrix runs on the PR; these
focused local checks do not replace it.

## Actual native captures

All images are 1:1 captures from the delivered Windows executable, with demo
requests, synchronization and playback disabled. The approved desktop scenery
design is unchanged; this package exports its existing ridge mark as an ICO.

- [Dark, 1280 × 800](home-dark-normal.png)
- [Light, 1280 × 800](home-light-normal.png)
- [Dark, 760 × 800](home-dark-narrow.png)
- [Light, 760 × 800](home-light-narrow.png)

## Artifacts and limits

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `Spotiurge-0.12.0-dev.20261007-windows-x86_64-setup.exe` | 11,590,176 | `0c838df6f2ec0b7054b3ee952ae1e63129741b2a4f710b7c1b6a1d93bdb09d43` |
| `Spotiurge.exe` | 29,308,928 | `e24da70975721cb687b936120773f800ac40284a118ce841ac02c0e942f9f088` |

The same owner received private download links and masked-prompt pairing
instructions by email. No pairing token, Spotify grant or AI credential is in
the package, these notes or the screenshots.

No real Windows Spotify sign-in, audible playback, live AI recommendation or
new live cloud-sync round was tested in this run. Prior cross-device sync
evidence remains in the discovery review. No Windows ARM64 machine, clean
Windows 10 machine or SmartScreen download-install flow was available.
