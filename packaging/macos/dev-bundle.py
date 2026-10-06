#!/usr/bin/env python3
"""Create a local Spotiurge development bundle, without publishing a release."""
import argparse
import plistlib
import re
import shutil
import subprocess
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("binary", type=Path)
parser.add_argument("output", type=Path)
parser.add_argument("--identity", help="Installed Apple Development signing identity")
parser.add_argument("--hardened-runtime", action="store_true")
args = parser.parse_args()
if args.output.suffix != ".app" or args.output.exists():
    parser.error("Choose a new .app output path; existing bundles are preserved.")
contents = args.output / "Contents"
(contents / "MacOS").mkdir(parents=True)
shutil.copy2(args.binary, contents / "MacOS" / "Spotiurge")
with (Path(__file__).parent / "Info.plist").open("rb") as source:
    info = plistlib.load(source)
manifest = (Path(__file__).resolve().parents[2] / "Cargo.toml").read_text()
version = re.search(r'^version\s*=\s*"([^"]+)"', manifest, re.MULTILINE).group(1)
info.update(CFBundleName="Spotiurge", CFBundleDisplayName="Spotiurge",
    CFBundleIdentifier="com.sergeserbinenko.spotiurge", CFBundleExecutable="Spotiurge",
    CFBundleShortVersionString=version.split("-")[0], CFBundleVersion="1")
# Development must not register itself as the default Spotify-link handler.
info.pop("CFBundleURLTypes", None)
info.pop("CFBundleIconFile", None)
with (contents / "Info.plist").open("wb") as output:
    plistlib.dump(info, output)
sign = ["codesign", "--force", "--sign", args.identity or "-"]
if args.hardened_runtime:
    sign.extend(["--options", "runtime"])
subprocess.run([*sign, str(args.output)], check=True)
subprocess.run(["codesign", "--verify", "--strict", str(args.output)], check=True)
print(args.output)
