#!/bin/sh
# Builds the Rust engine for iPhone, then the signed probe app.
# Output stays in the checkout's target/ios-probe. Extra arguments go to
# xcodebuild (for example CODE_SIGNING_ALLOWED=NO for a compile-only check).
set -eu
probe=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$probe/../.." && pwd)
out="$root/target/ios-probe"
export PATH="$HOME/.cargo/bin:$PATH"
export IPHONEOS_DEPLOYMENT_TARGET=26.0

(cd "$probe/engine" && CARGO_TARGET_DIR="$out" cargo build --locked --release --target aarch64-apple-ios)
xcodebuild -project "$probe/app/PlaybackProbe.xcodeproj" -target PlaybackProbe \
    -configuration Release -sdk iphoneos -allowProvisioningUpdates \
    SYMROOT="$out/xcode/build" OBJROOT="$out/xcode/obj" "$@"
echo "$out/xcode/build/Release-iphoneos/PlaybackProbe.app"
