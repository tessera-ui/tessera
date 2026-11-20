#!/bin/sh
# Run on macOS with a booted simulator or a provisioned iPhone.
set -eu

device=${1:?Usage: sh ios_platform_view_probe.sh UDID [--ui-tests|--device-tests]}
mode=${2:-}
case "$mode" in
    ''|--ui-tests) sdk=iphonesimulator; destination='iOS Simulator' ;;
    --device-tests)
        sdk=iphoneos
        destination=iOS
        : "${TESSERA_IOS_SIGNING_IDENTITY:?Set an Apple Development signing identity}"
        : "${TESSERA_IOS_TEAM:?Set the device signing team ID}"
        : "${TESSERA_IOS_PROFILE:?Set the absolute path to the probe provisioning profile}"
        : "${TESSERA_IOS_ENTITLEMENTS:?Set the absolute path to the probe signing entitlements}"
        ;;
    *) echo 'Expected --ui-tests or --device-tests' >&2; exit 2 ;;
esac
cd "$(dirname "$0")/../.."

target=aarch64-apple-ios
if [ "$sdk" = iphonesimulator ]; then
    target=aarch64-apple-ios-sim
    if [ "$(uname -m)" = x86_64 ]; then target=x86_64-apple-ios; fi
fi
export SDKROOT
SDKROOT=$(xcrun --sdk "$sdk" --show-sdk-path)
export IPHONEOS_DEPLOYMENT_TARGET=15.0
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0
cargo build --locked -p example --bin example --target "$target"

scratch=$(mktemp -d)
app=$scratch/TesseraPlatformViewProbe.app
mkdir -p "$app"
trap 'rm -f "$app/example" "$app/Info.plist" "$app/embedded.mobileprovision" "$app/_CodeSignature/CodeResources"; rmdir "$app/_CodeSignature" 2>/dev/null || true; rmdir "$app" "$scratch"' 0
binary=${CARGO_TARGET_DIR:-target}/$target/debug/example
cp "$binary" "$app/example"
bundle_id=com.tessera.example.platform-view-probe
cat > "$app/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>example</string>
<key>CFBundleIdentifier</key><string>$bundle_id</string>
<key>CFBundleName</key><string>Tessera Platform View Probe</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>1.0</string>
<key>CFBundleVersion</key><string>1</string>
<key>LSRequiresIPhoneOS</key><true/>
<key>UILaunchScreen</key><dict/>
<key>UISupportedInterfaceOrientations</key><array>
<string>UIInterfaceOrientationPortrait</string>
<string>UIInterfaceOrientationLandscapeLeft</string>
<string>UIInterfaceOrientationLandscapeRight</string>
</array>
<key>UIApplicationSceneManifest</key><dict>
<key>UIApplicationSupportsMultipleScenes</key><false/>
<key>UISceneConfigurations</key><dict>
<key>UIWindowSceneSessionRoleApplication</key><array><dict>
<key>UISceneConfigurationName</key><string>Default Configuration</string>
</dict></array>
</dict>
</dict>
</dict></plist>
EOF
if [ "$mode" = --device-tests ]; then
    cp "$TESSERA_IOS_PROFILE" "$app/embedded.mobileprovision"
    codesign --force --sign "$TESSERA_IOS_SIGNING_IDENTITY" \
        --entitlements "$TESSERA_IOS_ENTITLEMENTS" "$app"
    xcrun devicectl device install app --device "$device" "$app"
else
    codesign --force --sign - "$app"
    xcrun simctl install "$device" "$app"
    data=$(xcrun simctl get_app_container "$device" "$bundle_id" data)
    report=$data/tmp/tessera-platform-view-hit-test.txt
    rm -f "$report"
    xcrun simctl launch --terminate-running-process "$device" "$bundle_id" \
        --platform-view-probe --check-platform-view-hit-testing

    remaining=20
    while [ ! -s "$report" ] && [ "$remaining" -gt 0 ]; do
        sleep 1
        remaining=$((remaining - 1))
    done
    if [ ! -s "$report" ]; then
        echo 'UIKit hit test did not produce a report' >&2
        exit 1
    fi
    cat "$report"
    [ "$(grep -c '^PASS ' "$report")" -eq 5 ]
fi

if [ -n "$mode" ]; then
    output=${CARGO_TARGET_DIR:-target}/ios-platform-view-ui
    mkdir -p "$output"
    cp example/tests/ios-ui/project.yml example/tests/ios-ui/PlatformViewTests.swift "$output/"
    xcodegen generate --spec "$output/project.yml" --project "$output"
    set -- test -project "$output/TesseraProbe.xcodeproj" -scheme TesseraProbe \
        -destination "platform=$destination,id=$device" -parallel-testing-enabled NO \
        -derivedDataPath "$output/DerivedData"
    if [ "$mode" = --device-tests ]; then
        set -- "$@" "DEVELOPMENT_TEAM=$TESSERA_IOS_TEAM" CODE_SIGN_STYLE=Automatic
    fi
    if [ -n "${TESSERA_IOS_TEST_FILTER:-}" ]; then
        set -- "$@" "-only-testing:$TESSERA_IOS_TEST_FILTER"
    fi
    xcodebuild "$@"
fi
