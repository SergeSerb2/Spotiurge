#!/usr/bin/env python3
"""Package a credential-free Spotiurge development DMG without publishing it."""
import argparse
import hashlib
import shutil
import subprocess
import tempfile
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("binary", type=Path, help="Release binary, optionally universal")
parser.add_argument("output", type=Path, help="New .dmg path in a local build directory")
parser.add_argument("--identity", required=True, help="Installed signing identity")
args = parser.parse_args()
if args.output.suffix != ".dmg" or args.output.exists():
    parser.error("Choose a new .dmg path; existing artifacts are preserved.")
args.output = args.output.resolve()
args.output.parent.mkdir(parents=True, exist_ok=True)
root = Path(__file__).resolve().parents[2]
with tempfile.TemporaryDirectory(prefix="spotiurge-dmg-", dir=args.output.parent) as scratch:
    stage = Path(scratch)
    subprocess.run([
        "python3", str(root / "packaging/macos/dev-bundle.py"),
        str(args.binary.resolve()), str(stage / "Spotiurge.app"),
        "--identity", args.identity, "--hardened-runtime",
    ], check=True)
    (stage / "Applications").symlink_to("/Applications")
    shutil.copy2(root / "LICENSE", stage / "LICENSE.txt")
    (stage / "Read me.txt").write_text(
        "Spotiurge development preview\n\n"
        "Copy Spotiurge.app to Applications, then sign in to Spotify.\n"
        "Spotify Premium is required for independent playback.\n\n"
        "This preview is signed with an Apple Development certificate.\n"
        "It is NOT notarized and may be blocked by macOS Gatekeeper.\n"
        "A Developer ID certificate and notarization are still needed for distribution.\n"
        "No Spotify grants, private cloud pairing token, or AI credentials are included.\n"
        "Pair private sync separately using the documented protected credential store.\n"
        "MilkDrop is omitted; ordinary playback and native visualizers remain available.\n\n"
        "Source and setup: https://github.com/SergeSerb2/Spotiurge\n"
        "MIT license; fork of Spotifast by Carmine Paolino.\n",
        encoding="utf-8",
    )
    subprocess.run([
        "hdiutil", "create", "-volname", "Spotiurge Development",
        "-srcfolder", str(stage), "-format", "UDZO", str(args.output),
    ], check=True)
subprocess.run(["hdiutil", "verify", str(args.output)], check=True)
hasher = hashlib.sha256()
with args.output.open("rb") as artifact:
    for chunk in iter(lambda: artifact.read(1_048_576), b""):
        hasher.update(chunk)
digest = hasher.hexdigest()
args.output.with_suffix(".dmg.sha256").write_text(f"{digest}  {args.output.name}\n")
print(f"{args.output}\nSHA-256: {digest}")
