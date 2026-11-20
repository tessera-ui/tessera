"""Run against a debug example APK on an unlocked portrait Android device.

Run with `--prepare` before building to stage the Java view factory, then build
and install the debug example APK. Requires Pillow. `ANDROID_SERIAL` selects a
device when more than one is connected.
"""

import argparse
from io import BytesIO
import os
from pathlib import Path
import re
import shutil
import subprocess
from tempfile import TemporaryDirectory
import threading
import time
import uuid
import xml.etree.ElementTree as ET

from PIL import Image


PACKAGE = "com.tessera.example"
ACTIVITY = f"{PACKAGE}/com.tessera.TesseraGameActivity"
BACKGROUND = (255, 255, 255)
FIRST_VIEW = (32, 200, 80)
SECOND_VIEW = (240, 150, 30)
BETWEEN = (214, 128, 224)
FOREGROUND = (98, 163, 240)
FOREGROUND_PRESSED = (236, 104, 124)
EXAMPLE = Path(__file__).resolve().parents[1]
FACTORY_SOURCE = EXAMPLE / "tests" / "android" / "PlatformViewProbeFactory.java"
AUTOFILL_SERVICE = "com.tessera.example.test/com.tessera.example.PlatformViewAutofillService"
INSTRUMENTATION = "com.tessera.example.test/com.tessera.example.PlatformViewAccessibilityTest"
FACTORY_DESTINATION = (
    EXAMPLE
    / "gen"
    / "android"
    / "app"
    / "src"
    / "debug"
    / "java"
    / "com"
    / "tessera"
    / "example"
    / "PlatformViewProbeFactory.java"
)


def adb_command(*args):
    serial = os.environ.get("ANDROID_SERIAL")
    device = ["-s", serial] if serial else []
    return ["adb", *device, *args]


def adb(*args):
    return subprocess.check_output(adb_command(*args), timeout=30)


def eventually(observe, check, description):
    deadline = time.monotonic() + 20
    while True:
        actual = observe()
        if check(actual):
            print(f"PASS: {description}: {actual}", flush=True)
            return actual
        if time.monotonic() >= deadline:
            raise AssertionError(f"{description}: observed {actual!r}")
        time.sleep(0.2)


def check_autofill(save):
    user = adb("shell", "am", "get-current-user").decode().strip()
    settings = ("shell", "settings", "--user", user)
    old_service = adb(*settings, "get", "secure", "autofill_service").decode().strip()
    assert b"package:" in adb("shell", "pm", "path", "com.tessera.example.test"), (
        "Install the platform-view instrumentation APK before running the autofill probe"
    )
    try:
        adb(*settings, "put", "secure", "autofill_service", AUTOFILL_SERVICE)
        adb("shell", "am", "force-stop", PACKAGE)
        output = subprocess.check_output(adb_command(
            "shell", "am", "instrument", "-w", "-r", "-e", "autofill", "true",
            "-e", "autofillSave", str(save).lower(), INSTRUMENTATION,
        ), timeout=65).decode()
        assert ("INSTRUMENTATION_RESULT: system filled native editor=true" in output
                and "INSTRUMENTATION_CODE: -1" in output), output
        print(f"PASS: Android autofill populated the native editor\n{output}")
        if save:
            assert "INSTRUMENTATION_RESULT: system saved edited native value=true" in output, output
    finally:
        if old_service and old_service != "null":
            adb(*settings, "put", "secure", "autofill_service", old_service)
        else:
            adb(*settings, "delete", "secure", "autofill_service")


def check_composite_frames(tap, pixel_position, second_color):
    """Sample captured display frames, rather than only each settled layout."""
    assert_frame_transition(
        lambda: tap(150, 300), pixel_position,
        ((190, 124), (240, 150), (287, 124), (316, 274)),
        (second_color, second_color, BACKGROUND, BACKGROUND),
        (FIRST_VIEW, BETWEEN, second_color, (0, 0, 0)),
        repeat=True, description="reorder",
    )
    print("PASS: captured frames keep the main buffer, overlays and native geometry together")


def assert_frame_transition(change, pixel_position, points, old, new, *, repeat=False, description):
    ffmpeg = shutil.which("ffmpeg")
    if ffmpeg is None:
        raise RuntimeError("--require-frame-coherence needs FFmpeg")
    with Image.open(BytesIO(adb("exec-out", "screencap", "-p"))) as screenshot:
        screen_width, screen_height = screenshot.size
    width, height = screen_width // 4 * 2, screen_height // 4 * 2
    scale = min(width / screen_width, height / screen_height)
    offset_x, offset_y = (width - screen_width * scale) / 2, (height - screen_height * scale) / 2
    display_points = [pixel_position(x, y) for x, y in points]
    points = [(round(px * scale + offset_x), round(py * scale + offset_y))
              for px, py in display_points]
    artifact_dir = os.environ.get("TESSERA_PROBE_ARTIFACT_DIR")
    if artifact_dir:
        artifact_dir = Path(artifact_dir)
        artifact_dir.mkdir(parents=True, exist_ok=True)

    # Hardware encoders can shift recorded colors more than the screenshot path.
    # The two ranges for each sample must still be disjoint to detect mixed frames.
    video_tolerance = 32
    assert all(max(abs(a - b) for a, b in zip(before, after)) > 2 * video_tolerance
               for before, after in zip(old, new))

    def sample_display():
        with Image.open(BytesIO(adb("exec-out", "screencap", "-p"))) as screenshot:
            rgb = screenshot.convert("RGB")
            return tuple(rgb.getpixel(p) for p in display_points)

    def matches(pixels, expected, tolerance=15):
        return all(max(abs(a - b) for a, b in zip(pixel, target)) <= tolerance
                    for pixel, target in zip(pixels, expected))

    remote = f"/data/local/tmp/tessera-{uuid.uuid4().hex}.mp4"
    with TemporaryDirectory() as scratch:
        video = Path(scratch) / "composite.mp4"
        try:
            with subprocess.Popen(adb_command(
                    "shell", "-tt", "screenrecord", "--verbose", "--size", f"{width}x{height}",
                    "--time-limit", "8", remote), stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                    text=True) as recording:
                ready = threading.Event()

                def drain_recording_output():
                    for line in recording.stdout:
                        if "Content area" in line:
                            ready.set()

                reader = threading.Thread(target=drain_recording_output, daemon=True)
                reader.start()
                try:
                    assert ready.wait(timeout=10), "screenrecord did not start"
                    for expected in ((new, old, new, old) if repeat else (new,)):
                        change()
                        eventually(sample_display, lambda pixels: matches(pixels, expected),
                                   f"{description} settles while screenrecord is active")
                    assert recording.wait(timeout=12) == 0, "screenrecord failed"
                finally:
                    if recording.poll() is None:
                        recording.kill()
                    reader.join(timeout=2)
            adb("pull", remote, str(video))
        finally:
            adb("shell", "rm", "-f", remote)

        with subprocess.Popen([ffmpeg, "-v", "error", "-i", str(video), "-threads", "1",
                               "-fps_mode", "passthrough", "-f", "rawvideo", "-pix_fmt", "rgb24", "-"],
                              stdout=subprocess.PIPE) as decoder:
            seen = set()
            mixed = set()
            size = width * height * 3
            while data := decoder.stdout.read(size):
                rgb = Image.frombytes("RGB", (width, height), data)
                pixels = tuple(rgb.getpixel(p) for p in points)
                for name, expected in (("before", old), ("after", new)):
                    if matches(pixels, expected, tolerance=video_tolerance):
                        seen.add(name)
                        break
                else:
                    mixed.add(pixels)
            assert decoder.wait() == 0, "could not decode screenrecord output"

        if artifact_dir and (seen != {"before", "after"} or mixed):
            saved = artifact_dir / Path(remote).name
            shutil.copyfile(video, saved)
            print(f"Saved failing {description} recording: {saved}")
        assert seen == {"before", "after"}, f"recording missed a complete state: {seen}; samples: {mixed}"
        assert not mixed, f"{description} displayed mixed old/new buffers: {mixed}"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--prepare", action="store_true")
    parser.add_argument("--surface-backed", action="store_true")
    parser.add_argument("--both-surface-backed", action="store_true")
    parser.add_argument("--require-accessibility", action="store_true")
    parser.add_argument("--require-backend", choices=("gl", "vulkan"))
    parser.add_argument("--require-scaled-layout", action="store_true")
    parser.add_argument("--require-dynamic-order", action="store_true")
    parser.add_argument("--require-frame-coherence", action="store_true")
    parser.add_argument("--require-resize", action="store_true")
    parser.add_argument("--require-density-change", action="store_true")
    parser.add_argument("--require-native-editor", action="store_true")
    parser.add_argument("--require-final-removal", action="store_true")
    parser.add_argument("--scroll", action="store_true", help="run the separate scroll viewport probe")
    parser.add_argument("--remount", action="store_true", help="run the keyed-view remount probe")
    parser.add_argument("--replace", action="store_true", help="run the separate keyed replacement probe")
    parser.add_argument("--keyboard-traversal", action="store_true",
                        help="verify Tab and Shift+Tab across Tessera and the native editor")
    parser.add_argument("--autofill", action="store_true", help="run the separate native autofill probe")
    parser.add_argument("--autofill-save", action="store_true", help="also submit an edited autofill value")
    args = parser.parse_args()
    args.surface_backed |= args.both_surface_backed
    if args.prepare:
        FACTORY_DESTINATION.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(FACTORY_SOURCE, FACTORY_DESTINATION)
        print(f"Staged {FACTORY_DESTINATION}")
        return

    if args.autofill or args.autofill_save:
        check_autofill(args.autofill_save)
        return

    densities = re.findall(rb"density: (\d+)", adb("shell", "wm", "density"))
    scale = int(densities[-1]) / 160
    api_level = int(adb("shell", "getprop", "ro.build.version.sdk").strip())

    def pixel_position(x, y):
        return round(x * scale), round(y * scale)

    def check_pixels(description, *samples):
        def observe():
            png = adb("exec-out", "screencap", "-p")
            with Image.open(BytesIO(png)) as screenshot:
                rgb = screenshot.convert("RGB")
                return [rgb.getpixel(pixel_position(x, y)) for x, y, _ in samples]

        def matches(actual):
            return all(
                max(abs(a - b) for a, b in zip(pixel, expected, strict=True)) <= 12
                for pixel, (_, _, expected) in zip(actual, samples, strict=True)
            )

        eventually(observe, matches, description)

    def tap(x, y):
        adb("shell", "input", "tap", *map(str, pixel_position(x, y)))

    def wait_for_window_focus(description):
        def current_window():
            windows = adb("shell", "dumpsys", "window").decode()
            return next((line.strip() for line in windows.splitlines() if "mCurrentFocus=" in line), "")

        eventually(current_window, lambda line: ACTIVITY in line, description)

    def launch():
        command = ["shell", "am", "start", "-W", "-n", ACTIVITY,
                   "--ez", "tessera.platform_view_probe", "true"]
        if args.surface_backed:
            command.extend(["--ez", "tessera.platform_view_surface_backed", "true"])
        if args.both_surface_backed:
            command.extend(["--ez", "tessera.platform_view_both_surface_backed", "true"])
        if args.require_native_editor:
            command.extend(["--ez", "tessera.platform_view_editor", "true"])
        if args.scroll:
            command.extend(["--ez", "tessera.platform_view_scroll", "true"])
        if args.replace:
            command.extend(["--ez", "tessera.platform_view_replace", "true"])
        if args.keyboard_traversal:
            command.extend(["--ez", "tessera.platform_view_keyboard", "true"])
        result = adb(*command)
        if b"Status: ok" not in result:
            raise AssertionError(result.decode())

    def process_id():
        return adb("shell", "pidof", "-s", PACKAGE).strip().decode()

    adb("shell", "am", "force-stop", PACKAGE)
    launch()

    if args.keyboard_traversal:
        pid = process_id()
        idle = (0, 0, 0)
        focused = (0, 255, 0)

        def editor_focus(value, occurrences, description):
            def observe():
                logs = adb("logcat", "-d", f"--pid={pid}", "-v", "raw", "-s", "TesseraViewProbe:I")
                return [line for line in logs.decode().splitlines() if line.startswith("editor focus ")]

            expected = f"editor focus {str(value).lower()}"
            eventually(observe, lambda lines: lines.count(expected) == occurrences
                       and lines[-1] == expected, description)

        def tab(reverse=False):
            if reverse:
                adb("shell", "input", "keycombination", "KEYCODE_SHIFT_LEFT", "KEYCODE_TAB")
            else:
                adb("shell", "input", "keyevent", "KEYCODE_TAB")

        check_pixels("neither Tessera target starts focused", (44, 85, idle), (44, 205, idle))
        tab()
        check_pixels("Tab enters the first Tessera target", (44, 85, focused), (44, 205, idle))
        tab()
        editor_focus(True, 1, "Tab enters the native editor")
        check_pixels("Tessera relinquishes focus to the editor", (44, 85, idle), (44, 205, idle))
        adb("shell", "input", "text", "Hello")
        eventually(
            lambda: adb("logcat", "-d", f"--pid={pid}", "-v", "raw", "-s", "TesseraViewProbe:I")
                .decode().splitlines(),
            lambda lines: "editor typed Hello" in lines,
            "the focused native editor receives keyboard text",
        )
        tab()
        editor_focus(False, 1, "Tab leaves the native editor")
        check_pixels("Tab continues to the second Tessera target", (44, 85, idle), (44, 205, focused))
        tab(reverse=True)
        editor_focus(True, 2, "Shift+Tab reenters the native editor")
        tab(reverse=True)
        editor_focus(False, 2, "Shift+Tab leaves the native editor")
        check_pixels("Shift+Tab returns to the first Tessera target", (44, 85, focused), (44, 205, idle))
        adb("shell", "input", "keyevent", "KEYCODE_ENTER")
        check_pixels("Enter activates only the focused target", (44, 85, BACKGROUND), (44, 205, idle))
        return

    if args.scroll:
        pid = process_id()

        def view_logs():
            return adb("logcat", "-d", f"--pid={pid}", "-v", "raw", "-s", "TesseraViewProbe:I").decode().splitlines()

        def swipe(from_y, to_y):
            start = pixel_position(276, from_y)
            end = pixel_position(276, to_y)
            adb("shell", "input", "swipe", *map(str, (*start, *end, 550)))

        check_pixels("the native view and overlay clip at the bottom of the scroll viewport",
                     (80, 204, FIRST_VIEW), (142, 217, BETWEEN), (80, 232, BACKGROUND))
        swipe(208, 86)
        check_pixels("native and Tessera content scroll by the same distance",
                     (80, 105, FIRST_VIEW), (142, 117, BETWEEN), (80, 204, BACKGROUND))
        tap(80, 204)
        assert "click 0" not in view_logs(), "the native view kept its old scroll hit region"
        tap(80, 105)
        eventually(view_logs, lambda lines: lines.count("click 0") == 1,
                   "the native control receives touch at its scrolled position")
        swipe(86, 208)
        check_pixels("scrolling back restores the original composition",
                     (80, 105, BACKGROUND), (80, 204, FIRST_VIEW), (142, 217, BETWEEN))
        created = "create surface-backed 0" if args.both_surface_backed else "create 0"
        assert view_logs().count(created) == 1, "scrolling recreated the native control"
        return

    check_pixels("native controls interleave with three Tessera draws",
                 (306, 92, BACKGROUND), (48, 100, FIRST_VIEW), (80, 150, BETWEEN),
                 (240, 150, SECOND_VIEW), (162, 205, FOREGROUND))
    check_pixels("rectangular clip removes only the overflowing part of the native view",
                 (30, 100, BACKGROUND), (48, 100, FIRST_VIEW))
    if args.both_surface_backed:
        check_pixels("the upper independent surface is visible where both overlap",
                     (190, 124, SECOND_VIEW))

    pid = process_id()

    if args.require_backend:
        def adapter_backends():
            logs = adb("logcat", "-d", f"--pid={pid}", "-v", "raw", "-s", "RustStdoutStderr:I")
            return re.findall(r"^\s*backend: (\w+),", logs.decode(), re.MULTILINE)

        # Surface recreation can initialize another renderer in the same process.
        eventually(adapter_backends,
                   lambda backends: {backend.lower() for backend in backends} == {args.require_backend},
                   f"window uses the WGPU {args.require_backend} backend")

    def factory_logs():
        return adb("logcat", "-d", f"--pid={pid}", "-v", "raw", "-s", "TesseraViewProbe:I").decode().splitlines()

    if args.replace:
        replacement = (30, 160, 240)
        if args.require_frame_coherence:
            assert_frame_transition(lambda: tap(268, 300), pixel_position,
                                    ((240, 150), (316, 274)), (SECOND_VIEW, BACKGROUND),
                                    (replacement, (0, 0, 0)), description="keyed replacement")
            print("PASS: the old and new keyed compositions never mix")
        else:
            tap(268, 300)
        check_pixels("the new identity occupies the old view's placement", (240, 150, replacement),
                     (162, 205, FOREGROUND))
        tap(270, 200)
        logs = eventually(factory_logs, lambda lines: lines.count("click 3") == 1,
                          "only the new identity receives the tap")
        create = "create surface-backed" if args.surface_backed else "create"
        assert logs.count(f"{create} 1") == logs.count(f"{create} 3") == 1, logs
        assert logs.count("reset 1") == logs.count("release 1") == 1, logs
        assert logs.index("reset 1") < logs.index("release 1"), logs
        assert "click 1" not in logs and "release 3" not in logs, logs
        return

    if args.remount:
        tap(270, 200)
        eventually(factory_logs, lambda lines: lines.count("click 1") == 1,
                   "the original item receives touch")
        tap(268, 344)
        check_pixels("the fresh view stays hidden until placed", (240, 150, BETWEEN))

        def remounted(lines):
            created = "create surface-backed 42" if args.both_surface_backed else "create 42"
            return (lines.count("reset 1") == lines.count("release 1") == 1
                    and lines.count(created) == 1
                    and lines.index("reset 1") < lines.index("release 1"))

        eventually(factory_logs, remounted, "the old view is reset and released once on key change")
        tap(44, 300)
        check_pixels("the new item is visible without the old pressed state", (240, 150, SECOND_VIEW))
        tap(270, 200)
        logs = eventually(factory_logs, lambda lines: lines.count("click 42") == 1,
                          "only the new view receives touch")
        assert remounted(logs) and logs.count("click 1") == 1, logs
        return

    if args.require_scaled_layout:
        layout = f"layout 1 {int(84 * scale)}x{int(72 * scale)}"
        eventually(factory_logs, lambda lines: layout in lines,
                   "native view is laid out in unscaled physical pixels")

    if args.surface_backed:
        eventually(
            factory_logs,
            lambda lines: "create surface-backed 1" in lines,
            "the second platform view uses an independent SurfaceView surface",
        )
    accessibility_dump = "/data/local/tmp/tessera-platform-view-accessibility.xml"
    def accessibility_counts():
        adb("shell", "uiautomator", "dump", accessibility_dump)
        tree = ET.fromstring(adb("shell", "cat", accessibility_dump))
        return {
            label: sum(label.casefold() in (node.get("content-desc", "").casefold(),
                                           node.get("text", "").casefold())
                       for node in tree.iter("node"))
            for label in ("Native 1", "Tessera foreground probe")
        }

    if args.require_accessibility and api_level < 35:
        # The first dump can activate the Tessera accessibility adapter before it has published.
        eventually(accessibility_counts, lambda counts: all(count == 1 for count in counts.values()),
                   "one native and one Tessera accessibility node")

    if api_level >= 35:
        expected_layers = {"Tessera scene base", "Tessera scene slice 0", "Tessera scene slice 1"}
        eventually(
            lambda: sorted(set(re.findall(
                r"Tessera scene (?:base|slice \d+)",
                adb("shell", "dumpsys", "SurfaceFlinger", "--list").decode(),
            ))),
            lambda layers: set(layers) == expected_layers,
            "the base and two populated overlays use reusable compositor layers",
        )
    else:
        eventually(
            lambda: adb("logcat", "-d", f"--pid={pid}", "-v", "raw", "-s", "Tessera:I")
                .decode().splitlines(),
            lambda lines: "Using TextureView composition fallback" in lines,
            f"Android API {api_level} selects the TextureView composition fallback",
        )

    if args.require_resize:
        rotation = adb("shell", "wm", "user-rotation").decode().split()
        assert rotation and rotation[0] in ("free", "lock"), rotation

        def screenshot_size():
            with Image.open(BytesIO(adb("exec-out", "screencap", "-p"))) as screenshot:
                return screenshot.size

        try:
            for angle, landscape in (("1", True), ("0", False)):
                adb("shell", "wm", "user-rotation", "lock", angle)
                eventually(screenshot_size, lambda size: (size[0] > size[1]) == landscape,
                           "window rotates for the resize probe")
                check_pixels("the resized window retains the native composite",
                             (48, 100, FIRST_VIEW), (80, 150, BETWEEN),
                             (240, 150, SECOND_VIEW), (162, 205, FOREGROUND))
            assert process_id() == pid, "rotation restarted the process"
        finally:
            adb("shell", "wm", "user-rotation", *rotation)

    if args.require_density_change:
        original_density = int(densities[-1])
        test_density = max(80, original_density - 40)
        assert test_density != original_density, "density probe needs an initial density above 80 dpi"
        first_view_created = "create surface-backed 0" if args.both_surface_backed else "create 0"
        second_view_created = "create surface-backed 1" if args.surface_backed else "create 1"

        def check_density(density, previous_logs):
            nonlocal scale
            scale = density / 160
            dimensions = f"layout 1 {int(84 * scale)}x{int(72 * scale)}"
            eventually(factory_logs, lambda lines: lines.count(dimensions) > previous_logs.count(dimensions),
                       f"native view receives unscaled dimensions at {density} dpi")
            check_pixels(f"native geometry and Tessera slices agree at {density} dpi",
                         (30, 100, BACKGROUND), (48, 100, FIRST_VIEW), (80, 150, BETWEEN),
                         (240, 150, SECOND_VIEW), (162, 205, FOREGROUND))
            logs = factory_logs()
            assert logs.count(first_view_created) == logs.count(second_view_created) == 1, logs
            assert process_id() == pid, "density change restarted the process"
            wait_for_window_focus(f"Android routes input to the Activity at {density} dpi")
            tap(162, 205)
            check_pixels(f"foreground still receives touch at {density} dpi",
                         (162, 205, FOREGROUND_PRESSED))
            assert factory_logs().count("click 1") == logs.count("click 1"), logs
            tap(162, 205)
            check_pixels(f"foreground can be toggled back at {density} dpi", (162, 205, FOREGROUND))

        before_restore = factory_logs()
        try:
            adb("shell", "wm", "density", str(test_density))
            check_density(test_density, before_restore)
        finally:
            override = str(original_density) if len(densities) > 1 else "reset"
            adb("shell", "wm", "density", override)
        check_density(original_density, before_restore)

    tap(30, 100)
    check_pixels("clipped-out touch reaches the Tessera underlay",
                 (30, 100, (0, 0, 0)), (48, 100, FIRST_VIEW))
    assert "click 0" not in factory_logs(), "native button received a touch outside the clip"
    tap(48, 100)
    eventually(factory_logs, lambda lines: "click 0" in lines, "native button receives touch")
    if not args.surface_backed:
        tap(162, 150)
        eventually(factory_logs, lambda lines: lines.count("click 1") == 1,
                   "upper native button receives touch above the middle Tessera overlay")
        check_pixels("the upper native touch did not trigger the covered Tessera overlay",
                     (80, 150, BETWEEN))
        tap(80, 150)
        check_pixels("the middle Tessera overlay consumes touch above the lower native view",
                     (80, 150, (0, 0, 0)))
        assert factory_logs().count("click 0") == 1, "the covered native button also received touch"
        tap(80, 150)
        check_pixels("the middle overlay can be toggled back", (80, 150, BETWEEN))
    tap(44, 300)
    check_pixels("hiding the second native view exposes the preceding Tessera slice",
                 (240, 150, BETWEEN), (162, 205, FOREGROUND))
    tap(44, 300)
    check_pixels("restoring the second native view restores its original z order",
                  (240, 150, SECOND_VIEW), (162, 205, FOREGROUND))
    second_view_clicks = factory_logs().count("click 1")
    tap(270, 200)
    eventually(factory_logs, lambda lines: lines.count("click 1") == second_view_clicks + 1,
               "the second native view receives its own touch")
    if args.surface_backed:
        check_pixels("surface-backed native content updates independently",
                     (270, 200, (160, 90, 30)))
    second_view_clicks = factory_logs().count("click 1")

    tap(162, 205)
    try:
        check_pixels("later Tessera draw consumes touch", (162, 205, FOREGROUND_PRESSED))
    except AssertionError as error:
        logs = factory_logs()
        if logs.count("click 1") > second_view_clicks:
            raise AssertionError(
                "Tessera overlay touch activated the native view underneath it: "
                f"{logs}"
            ) from error
        raise
    assert factory_logs().count("click 1") == second_view_clicks, (
        "Tessera overlay touch leaked through to the covered native view"
    )

    clicks_before_resume = factory_logs().count("click 0")
    adb("shell", "input", "keyevent", "KEYCODE_HOME")

    def resumed_activity():
        activities = adb("shell", "dumpsys", "activity", "activities").decode()
        return next((line.strip() for line in activities.splitlines()
                     if "topResumedActivity=" in line or "mResumedActivity:" in line), "")

    eventually(resumed_activity, lambda activity: activity and PACKAGE not in activity,
               "probe Activity is backgrounded")
    launch()
    resumed_second_view = (160, 90, 30) if args.surface_backed else SECOND_VIEW
    check_pixels("resuming restores the clipped native views and Tessera slices",
                  (48, 100, FIRST_VIEW), (80, 150, BETWEEN), (240, 150, resumed_second_view))
    assert pid == process_id(), "probe restarted instead of resuming in the same process"
    wait_for_window_focus("Android returns input focus to the resumed Activity")
    tap(48, 100)
    eventually(factory_logs, lambda lines: lines.count("click 0") == clicks_before_resume + 1,
               "the retained native button still receives touch after resume")

    if args.require_frame_coherence:
        check_composite_frames(tap, pixel_position, resumed_second_view)

    if args.require_dynamic_order:
        check_pixels("native paint order and extent before reorder",
                     (190, 124, resumed_second_view), (287, 124, BACKGROUND))
        tap(150, 300)
        check_pixels("reordering and moving a retained native view updates the composite",
                     (190, 124, FIRST_VIEW), (240, 150, BETWEEN),
                     (287, 124, resumed_second_view))
        first_clicks = factory_logs().count("click 0")
        tap(190, 124)
        eventually(factory_logs, lambda lines: lines.count("click 0") == first_clicks + 1,
                   "the new topmost native view receives touch")
        second_clicks = factory_logs().count("click 1")
        tap(287, 124)
        eventually(factory_logs, lambda lines: lines.count("click 1") == second_clicks + 1,
                   "the native view receives touch at its new position")
        tap(150, 300)
        check_pixels("restoring paint order also restores both native hit regions",
                     (190, 124, resumed_second_view if not args.surface_backed else SECOND_VIEW),
                     (287, 124, BACKGROUND))

    if args.require_native_editor:
        tap(100, 372)
        eventually(factory_logs, lambda lines: "editor focus true" in lines,
                   "native editor receives focus")
        def ime_visibility():
            return re.findall(rb"\b(?:mInputShown|mImeShowing)=(?:true|false)\b",
                              adb("shell", "dumpsys", "input_method"))

        eventually(ime_visibility, lambda flags: any(flag.endswith(b"=true") for flag in flags),
                   "Android IME opens for the native editor")
        adb("shell", "input", "text", "Hello")
        eventually(factory_logs, lambda lines: "editor typed Hello" in lines,
                   "native editor receives keyboard input")

    if args.require_final_removal:
        before = factory_logs()
        assert "reset 1" not in before and "release 1" not in before, before
        tap(268, 300)
        check_pixels("removing the registered view exposes the Tessera slice",
                     (240, 150, BETWEEN))
        def released_once(lines):
            return (lines.count("reset 1") == lines.count("release 1") == 1
                    and lines.index("reset 1") < lines.index("release 1"))

        eventually(factory_logs, released_once, "the removed native view resets and releases once")
        tap(270, 200)
        assert factory_logs().count("click 1") == before.count("click 1")

    logs = factory_logs()
    first_view_created = "create surface-backed 0" if args.both_surface_backed else "create 0"
    second_view_created = "create surface-backed 1" if args.surface_backed else "create 1"
    assert logs.count(first_view_created) == logs.count(second_view_created) == 1, logs
    if args.require_accessibility and api_level >= 35:
        # The shell XML dumper cannot reliably traverse a SurfaceView's embedded windows.
        result = adb("shell", "am", "instrument", "-w", "-r", "-e", "surfaceBacked",
                     str(args.surface_backed).lower(),
                     "-e", "bothSurfaceBacked", str(args.both_surface_backed).lower(),
                     INSTRUMENTATION)
        output = result.decode()
        assert ("INSTRUMENTATION_RESULT: Native 1=1" in output
                and "INSTRUMENTATION_RESULT: Tessera foreground probe=1" in output
                and "INSTRUMENTATION_CODE: -1" in output), output
        print(f"PASS: embedded accessibility and destruction\n{output}", flush=True)


def save_failure_artifacts():
    root = os.environ.get("TESSERA_PROBE_ARTIFACT_DIR")
    if not root:
        return
    folder = Path(root) / f"failure-{uuid.uuid4().hex}"
    folder.mkdir(parents=True)
    for filename, command in (
        ("screen.png", ("exec-out", "screencap", "-p")),
        ("window.txt", ("shell", "dumpsys", "window")),
        ("input.txt", ("shell", "dumpsys", "input")),
        ("surface-flinger.txt", ("shell", "dumpsys", "SurfaceFlinger")),
        ("composition-logcat.txt", ("logcat", "-d", "-v", "threadtime", "-s",
                                     "Tessera:V", "TesseraViewProbe:V", "RustStdoutStderr:V",
                                     "SurfaceSyncGroup:W", "InputDispatcher:W")),
        ("anrs.txt", ("logcat", "-b", "events", "-d", "-s", "am_anr")),
    ):
        try:
            output = subprocess.check_output(adb_command(*command), timeout=8)
            (folder / filename).write_bytes(output)
        except (OSError, subprocess.SubprocessError) as error:
            print(f"Could not collect {filename}: {error}")
    print(f"Saved failed probe state: {folder}")


if __name__ == "__main__":
    try:
        main()
    except (AssertionError, subprocess.SubprocessError):
        save_failure_artifacts()
        raise
