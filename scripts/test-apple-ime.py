import json
import os
import platform
import plistlib
import shutil
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "tessera-winit/tests/apple-ime/Cargo.toml"
os.environ["CARGO_TARGET_DIR"] = str(ROOT / "target")


def run(*args, timeout=120):
    try:
        return subprocess.run(args, check=True, text=True, stdout=subprocess.PIPE, timeout=timeout).stdout
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired) as error:
        output = error.stdout or ""
        print(output.decode(errors="replace") if isinstance(output, bytes) else output, flush=True)
        raise


def build(target=None):
    args = ["cargo", "build", "--manifest-path", str(MANIFEST), "--message-format=json"]
    if target:
        args += ["--target", target]
    output = run(*args, timeout=900)
    for line in output.splitlines():
        message = json.loads(line)
        if message["reason"] == "compiler-artifact" and message.get("executable"):
            executable = Path(message["executable"])
    return executable


def check(*args):
    output = run(*args, timeout=90)
    print(output, flush=True)
    if "NATIVE_IME_PASS" not in output:
        raise RuntimeError("Native IME edits did not reach the application")


check(str(build()))
target = "aarch64-apple-ios-sim" if platform.machine() == "arm64" else "x86_64-apple-ios"
run("rustup", "target", "add", target)
executable = build(target)
devices = json.loads(run("xcrun", "simctl", "list", "devices", "available", "--json"))["devices"]
phone = next(device for runtime, group in devices.items() if ".iOS-" in runtime
             for device in group if device["name"].startswith("iPhone"))
udid = phone["udid"]
booted = phone["state"] == "Booted"
try:
    if not booted:
        run("xcrun", "simctl", "boot", udid)
    run("xcrun", "simctl", "bootstatus", udid, "-b", timeout=600)
    with tempfile.TemporaryDirectory() as directory:
        app = Path(directory) / "ImeTest.app"
        app.mkdir()
        shutil.copy2(executable, app / "ime-test")
        with (app / "Info.plist").open("wb") as file:
            plistlib.dump({
                "CFBundleIdentifier": "dev.tessera.ime-test",
                "CFBundleExecutable": "ime-test",
                "CFBundleName": "Tessera IME Test",
                "CFBundlePackageType": "APPL",
                "CFBundleVersion": "1",
                "CFBundleShortVersionString": "1.0",
                "MinimumOSVersion": "13.0",
                "UIDeviceFamily": [1, 2],
            }, file)
        run("xcrun", "simctl", "install", udid, str(app))
        check("xcrun", "simctl", "launch", "--console-pty", "--terminate-running-process",
              udid, "dev.tessera.ime-test")
finally:
    if not booted:
        run("xcrun", "simctl", "shutdown", udid)
