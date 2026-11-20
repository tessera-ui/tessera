# Platform view probes

The probes run the example's real mobile renderer. They need an unlocked phone
or emulator; the unit test suite does not run them automatically.

## Android

Run `python example/tests/android_platform_view_probe.py --prepare` before
building the example's debug APK. Install the APK and select the device with
`ANDROID_SERIAL`. With Pillow installed, run:

```text
python example/tests/android_platform_view_probe.py --require-accessibility --require-scaled-layout
python example/tests/android_platform_view_probe.py --surface-backed --require-accessibility --require-scaled-layout
```

Use `--require-backend gl` or `--require-backend vulkan` to reject a run on the
wrong graphics backend. For a GLES-only emulator run, start the AVD with
`-gpu host -feature -Vulkan` and require `gl` in the probe. The GLES path
requires an OpenGL ES 3 RGBA EGL window configuration; an AVD that exposes
only GLES 2 cannot validate it.
Use `--require-dynamic-order` to assert movement, overlap order, and touch
targets after reordering. Combine it with `--both-surface-backed` to test two
overlapping independent surfaces. Use `--require-native-editor` to launch with
a native `EditText` and verify focus, IME visibility, and text entry.
Use `--require-frame-coherence` with FFmpeg on the host to check recorded
reorder frames for a mixture of old and new geometry or Tessera buffers.
Set `TESSERA_PROBE_ARTIFACT_DIR` to retain failed recordings and capture device
state and compositor logs when a probe fails. Run one emulator at a time for
recorded-frame and resume checks to avoid host GPU contention.
Use `--require-final-removal` to verify that removal while the Activity stays
active updates the composite, stops native input, and resets and releases once.
Use `--require-resize` to verify the same native views remain composited after
rotating to landscape and back. This restores the device's rotation setting.
Use `--require-density-change` to change display density while the Activity is
running and check native layout, clipping, composition, and touch at both scales.
The probe restores the original density setting.
Run `--scroll` separately to verify native placement, clipping, and touch as
the surrounding Tessera viewport scrolls. Add `--both-surface-backed` to use
an independent surface in the viewport.
Run `--remount` separately to verify that changing a logical key releases the
old native view and mounts a fresh one even while its layout is hidden.
Run `--replace --require-frame-coherence` separately to verify that replacing a
keyed view presents the new object and Tessera content together, releases the
old object once, and routes touch only to the replacement.
Run `--keyboard-traversal` on API 31 or later to check hardware Tab and
Shift+Tab through a native editor between two Tessera targets. The probe uses
Android's `input keycombination` command for Shift+Tab and verifies that the
editor receives keyboard text before focus leaves it.

On API 35 and later, `--require-accessibility` uses instrumentation because the
shell XML dumper does not reliably traverse embedded windows. Instrumentation
also checks final release on Activity destruction. Build a test APK with
Android SDK 35: compile and link `android/res` and `android/AndroidManifest.xml`
with `aapt2`, compile `PlatformViewAccessibilityTest.java` and
`PlatformViewAutofillService.java` with `javac` and `d8`, and add `classes.dex`.
Zip-align and sign the result with the same debug key as the example APK.
Install both and run:

```text
adb shell am instrument -w -r com.tessera.example.test/com.tessera.example.PlatformViewAccessibilityTest
adb shell am instrument -w -r -e surfaceBacked true com.tessera.example.test/com.tessera.example.PlatformViewAccessibilityTest
adb shell am instrument -w -r -e bothSurfaceBacked true -e accessibilityFocus true com.tessera.example.test/com.tessera.example.PlatformViewAccessibilityTest
adb shell am instrument -w -r -e lateAccessibility true -e relaunchActivity true com.tessera.example.test/com.tessera.example.PlatformViewAccessibilityTest
```

`INSTRUMENTATION_CODE: -1` is Android's `RESULT_OK`. With `relaunchActivity`,
the test checks that a replacement Activity lays out its native views and
releases them on destruction without restarting the process.

On a test device with the instrumentation APK installed, run `--autofill`
separately. It selects the test autofill provider, accepts its synthetic value
in a native editor, then restores the previous provider.
Use `--autofill-save` to also edit that value and verify the system save request.

## iOS Simulator

On macOS with Xcode, XcodeGen, and a booted iPhone simulator:

```sh
sh example/tests/ios_platform_view_probe.sh SIMULATOR_UDID --ui-tests
```

The probe checks UIKit hit testing and native layout sizing before launching the
XCUITest suite for screenshots, touch, scrolling, orientation, keyboard,
reordering, and lifecycle. Set `TESSERA_IOS_TEST_FILTER` to an Xcode test
identifier to rerun one test.

## iPhone

Pair an unlocked iPhone with the Mac and enable Developer Mode. Set
`TESSERA_IOS_SIGNING_IDENTITY` to an Apple Development identity,
`TESSERA_IOS_TEAM` to its team ID, and `TESSERA_IOS_PROFILE` and
`TESSERA_IOS_ENTITLEMENTS` to absolute paths for the signed probe bundle
`com.tessera.example.platform-view-probe`. Xcode also needs provisioning for
the UI test runner under the same team. Run the same XCUITest suite on the phone:

```sh
sh example/tests/ios_platform_view_probe.sh DEVICE_UDID --device-tests
```

On the phone, enable VoiceOver and swipe through the probe past the native
buttons and the Tessera foreground button. Verify that each is announced once,
activation reaches the announced control, and hiding a native view removes it
from traversal. Repeat the keyboard scene with a connected hardware keyboard
to verify that Tab and Shift+Tab enter and leave the native editor in order.
