#!/bin/sh
# Builds the developmental Spotiurge iPhone app.
#
#   apps/ios/build.sh [simulator|device] [--no-engine] [xcodebuild settings...]
#
# simulator (default): an iOS Simulator build. device: an iphoneos build;
# pass DEVELOPMENT_TEAM=... to sign, or CODE_SIGNING_ALLOWED=NO to compile only.
# --no-engine leaves the Rust playback engine out (playback then reports
# itself unavailable). Output stays in the checkout's target/ios-interface.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)
out="$root/target/ios-interface"
export PATH="$HOME/.cargo/bin:$PATH"
export IPHONEOS_DEPLOYMENT_TARGET=26.0

platform=simulator
engine=yes
case "${1:-}" in simulator|device) platform=$1; shift ;; esac
case "${1:-}" in --no-engine) engine=no; shift ;; esac

# Stage Inter from the desktop's pinned fastframe-fonts and the app icon
# from assets/brand. Both are ignored copies, never committed here.
rev=$(sed -n 's/^source = "git+https:\/\/github.com\/crmne\/fastframe?tag=[^#]*#\([0-9a-f]\{7\}\).*/\1/p' "$root/Cargo.lock" | head -n 1)
font=$(ls "${CARGO_HOME:-$HOME/.cargo}"/git/checkouts/fastframe-*/"$rev"/crates/fastframe-fonts/fonts/InterVariable.ttf 2>/dev/null | head -n 1 || true)
if [ -z "$font" ]; then
    (cd "$root" && cargo fetch --locked >/dev/null)
    font=$(ls "${CARGO_HOME:-$HOME/.cargo}"/git/checkouts/fastframe-*/"$rev"/crates/fastframe-fonts/fonts/InterVariable.ttf | head -n 1)
fi
cp "$font" "$here/Spotiurge/InterVariable.ttf"
cp "$root/assets/brand/png/spotiurge-1024.png" "$here/Spotiurge/Assets.xcassets/AppIcon.appiconset/spotiurge-1024.png"
cp "$root/assets/scenery/alpine-lake.jpg" "$here/Spotiurge/Assets.xcassets/Scenery.imageset/alpine-lake.jpg"

if [ "$platform" = simulator ]; then
    triple=aarch64-apple-ios-sim
    destination='generic/platform=iOS Simulator'
    # The engine is built for Apple silicon simulators only.
    set -- "$@" ARCHS=arm64
else
    triple=aarch64-apple-ios
    destination='generic/platform=iOS'
fi

if [ "$engine" = yes ]; then
    # Development-only import: the playback probe's engine, unmodified, at
    # the stock librespot pin. Spotify refuses its iPhone identity; see README.
    (cd "$root/probes/ios-playback/engine" && CARGO_TARGET_DIR="$out" cargo build --locked --release --target "$triple")
    set -- "$@" SPOTIURGE_ENGINE_FLAG=SPOTIURGE_ENGINE "SPOTIURGE_ENGINE_LDFLAGS=$out/$triple/release/libspotiurge_playback_probe.a"
fi

xcodebuild -project "$here/Spotiurge.xcodeproj" -scheme Spotiurge -configuration Debug \
    -destination "$destination" -derivedDataPath "$out/xcode" -allowProvisioningUpdates "$@" build
echo "$out/xcode/Build/Products/Debug-$( [ "$platform" = simulator ] && echo iphonesimulator || echo iphoneos )/Spotiurge.app"
