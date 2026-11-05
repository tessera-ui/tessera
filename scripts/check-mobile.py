import argparse
import json
import os
import plistlib
import re
import subprocess
import time
from pathlib import Path


def run(*args, **kwargs):
    return subprocess.check_output(args, text=True, timeout=120, **kwargs).strip()


def events(read):
    result = []
    for line in read().splitlines():
        try:
            result.append(json.loads(line))
        except json.JSONDecodeError:
            pass
    return result


def wait(read, predicate):
    deadline = time.monotonic() + 90
    while time.monotonic() < deadline:
        records = events(read)
        if predicate(records):
            return records
        time.sleep(0.2)
    latest = frame(records) or {}
    raise RuntimeError(
        "Application did not publish the expected frame or lifecycle event; "
        f"events={[(item.get('type'), item.get('kind'), item.get('frame')) for item in records[-5:]]}, "
        f"render_time_ns={latest.get('render_time_ns')}, components={len(latest.get('components', []))}"
    )


def frame(records):
    return next((item for item in reversed(records) if item.get("type") == "frame"), None)


def valid(record):
    def walk(nodes):
        for node in nodes:
            yield node
            yield from walk(node.get("children", []))
    return record and (record.get("render_time_ns") or 0) > 0 and any(
        (node.get("fn_name") or "").split("::")[-1] == "text"
        for node in walk(record.get("components", []))
    )


def verify(read, background, foreground):
    initial = wait(read, lambda records: valid(frame(records)))
    background()
    suspended = wait(read, lambda records: any(item.get("type") == "runtime" and item.get("kind") == "suspended"
                                              for item in records[len(initial):]))
    foreground()
    def resumed(records):
        following = records[len(suspended):]
        position = next((i for i, item in enumerate(following)
                         if item.get("type") == "runtime" and item.get("kind") == "resumed"), None)
        return position is not None and valid(frame(following[position + 1:]))
    restored = wait(read, resumed)
    print(json.dumps({"first_frame": frame(initial)["frame"], "restored_frame": frame(restored)["frame"]}))


def android(directory):
    sdk = Path(os.environ.get("ANDROID_HOME", os.environ.get("ANDROID_SDK_ROOT", "")))
    aapt = sorted(sdk.glob("build-tools/*/aapt"))[-1]
    abi = run("adb", "shell", "getprop", "ro.product.cpu.abi")
    paths = list(directory.rglob("*.apk")) if directory.is_dir() else [directory]
    matches = [(path, run(str(aapt), "dump", "badging", str(path))) for path in paths]
    apk, metadata = max((item for item in matches if f"'{abi}'" in item[1]), key=lambda item: item[0].stat().st_mtime)
    identifier = re.search(r"package: name='([^']+)'", metadata)[1]
    activity = re.search(r"launchable-activity: name='([^']+)'", metadata)[1]
    run("adb", "install", "-r", str(apk))
    run("adb", "shell", "am", "force-stop", identifier)
    run("adb", "shell", "run-as", identifier, "rm", "-f", "files/acceptance.jsonl")
    launch = lambda: run("adb", "shell", "am", "start", "-W", "-n", f"{identifier}/{activity}")
    def read():
        result = subprocess.run(["adb", "shell", "run-as", identifier, "cat", "files/acceptance.jsonl"],
                                capture_output=True, text=True, timeout=15)
        return result.stdout
    launch()
    try:
        pid = run("adb", "shell", "pidof", identifier)
        verify(read, lambda: run("adb", "shell", "input", "keyevent", "KEYCODE_HOME"), launch)
        assert run("adb", "shell", "pidof", identifier) == pid, "Application restarted instead of resuming"
        run("adb", "shell", "am", "force-stop", identifier)
        output = Path("target/mobile-acceptance/android.jsonl")
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(read())
    finally:
        run("adb", "shell", "am", "force-stop", identifier)


def ios(udid, bundle):
    bundle = Path(bundle)
    identifier = plistlib.loads((bundle / "Info.plist").read_bytes())["CFBundleIdentifier"]
    run("xcrun", "simctl", "install", udid, str(bundle))
    container = Path(run("xcrun", "simctl", "get_app_container", udid, identifier, "data"))
    trace = container / "Documents/acceptance.jsonl"
    trace.unlink(missing_ok=True)
    output = Path("target/mobile-acceptance").resolve()
    output.mkdir(parents=True, exist_ok=True)
    stdout, stderr = output / "ios.stdout.log", output / "ios.stderr.log"
    env = dict(os.environ, SIMCTL_CHILD_TESSERA_PROFILING_OUTPUT=str(trace))
    launch = lambda: run("xcrun", "simctl", "launch", f"--stdout={stdout}", f"--stderr={stderr}", udid, identifier, env=env)
    def read():
        return trace.read_text() if trace.exists() else ""
    pid = launch().split(":")[-1].strip()
    try:
        verify(read, lambda: run("xcrun", "simctl", "launch", udid, "com.apple.mobilesafari"), launch)
        assert launch().split(":")[-1].strip() == pid, "Application restarted instead of resuming"
        run("xcrun", "simctl", "terminate", udid, identifier)
    except Exception:
        for path in (stdout, stderr):
            if path.exists():
                print(path.read_text(errors="replace")[-12000:], flush=True)
        raise
    finally:
        subprocess.run(["xcrun", "simctl", "terminate", udid, identifier], capture_output=True)
        (output / "ios.jsonl").write_text(read())


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--android", type=Path)
    mode.add_argument("--ios", nargs=2, metavar=("UDID", "APP"))
    args = parser.parse_args()
    android(args.android) if args.android else ios(*args.ios)
