#!/bin/sh
# Host-only regressions. Never connects to or changes the iPhone.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)
out="$root/target/ios-signin-tests"
mkdir -p "$out"
xcrun swiftc -swift-version 5 "$here/app/PlaybackProbe/SignInSupport.swift" \
    "$here/tests/SignInSupportTests.swift" -o "$out/signin-tests"
codesign --force --sign - "$out/signin-tests"
"$out/signin-tests"
