import argparse
import configparser
import json
import os
import plistlib
import re
import shutil
import struct
import subprocess
import tempfile
import tomllib
import zipfile
from html.parser import HTMLParser
from io import BytesIO
from pathlib import Path

from PIL import Image, ImageChops, ImageStat
from resvg_py import svg_to_bytes


EXAMPLE = Path(__file__).resolve().parents[1] / "example"
with (EXAMPLE / "tessera-app.toml").open("rb") as file:
    APP = tomllib.load(file)
with (EXAMPLE / "Cargo.toml").open("rb") as file:
    CARGO = tomllib.load(file)
NAME = APP["name"]
IDENTIFIER = APP["identifier"]
VERSION = CARGO["package"]["version"]
NUMERIC_VERSION = re.split(r"[-+]", VERSION, maxsplit=1)[0]
ICON = EXAMPLE / APP["icon"]


def run(*args):
    return subprocess.check_output(args, encoding="utf-8", timeout=120).strip()


def equal(field, actual, expected):
    assert actual == expected, f"{field}: expected {expected!r}, got {actual!r}"


def one(paths):
    paths = list(paths)
    assert len(paths) == 1, f"Expected one artifact, found {paths}"
    return paths[0]


def icon_matches(source, *, inset=False, monochrome=False, size=None):
    if isinstance(source, bytes):
        source = BytesIO(source)
    with Image.open(source) as image:
        actual = image.convert("RGBA")
    width, height = actual.size
    assert width == height and width >= 16, f"Invalid icon dimensions: {actual.size}"
    if size is not None:
        equal("icon size", width, size)
    artwork_size = width * 44 // 108 if inset else width
    if ICON.suffix.lower() == ".svg":
        rendered = BytesIO(svg_to_bytes(svg_path=str(ICON), width=artwork_size, height=artwork_size))
        with Image.open(rendered) as image:
            artwork = image.convert("RGBA")
    else:
        with Image.open(ICON) as image:
            artwork = image.convert("RGBA").resize((artwork_size, artwork_size), Image.Resampling.LANCZOS)
    if monochrome:
        alpha = artwork.getchannel("A")
        artwork = Image.new("RGBA", artwork.size)
        artwork.putalpha(alpha)
    expected = Image.new("RGBA", actual.size)
    offset = (width - artwork_size) // 2
    expected.paste(artwork, (offset, offset))

    def composite(image):
        return Image.alpha_composite(Image.new("RGBA", image.size, "white"), image).convert("RGB")

    expected = composite(expected)
    contrast = sum(ImageStat.Stat(ImageChops.difference(expected, Image.new("RGB", expected.size, "white"))).mean)
    error = sum(ImageStat.Stat(ImageChops.difference(composite(actual), expected)).mean)
    assert contrast > 0 and error < contrast * 0.15, f"Packaged icon differs from {ICON} (error {error:.2f}, contrast {contrast:.2f})"


def deb(directory, scratch):
    package = one(directory.glob("*.deb"))
    equal("Debian Package", run("dpkg-deb", "--field", package, "Package"), IDENTIFIER.lower())
    equal("Debian Version", run("dpkg-deb", "--field", package, "Version"), VERSION)
    run("dpkg-deb", "--extract", package, scratch)
    launcher = IDENTIFIER.replace(".", "-")
    desktop = configparser.ConfigParser(interpolation=None)
    desktop.read(scratch / "usr/share/applications" / f"{launcher}.desktop", encoding="utf-8")
    entry = desktop["Desktop Entry"]
    equal("desktop Name", entry["Name"], NAME)
    equal("desktop executable", entry["Exec"].split()[0], launcher)
    assert (scratch / "usr/bin" / launcher).is_file()
    icons = list((scratch / "usr/share/icons/hicolor").glob(f"*x*/apps/{entry['Icon']}.png"))
    assert icons, f"Missing desktop icon {entry['Icon']}"
    for icon in icons:
        icon_matches(icon)


def pe_metadata(path, *, application=False):
    import pefile

    with pefile.PE(str(path)) as pe:
        strings = {key.decode(): value.decode() for group in pe.FileInfo for info in group
                   for table in getattr(info, "StringTable", []) for key, value in table.entries.items()}
        equal(f"{path.name} ProductName", strings.get("ProductName"), NAME)
        equal(f"{path.name} ProductVersion", strings.get("ProductVersion"), VERSION)
        if application:
            equal(f"{path.name} InternalName", strings.get("InternalName"), IDENTIFIER)

        def resources(kind):
            types = pe.DIRECTORY_ENTRY_RESOURCE.entries
            entries = next(entry.directory.entries for entry in types if entry.id == kind)
            return {entry.id: pe.get_data(data.OffsetToData, data.Size) for entry in entries
                    for data in [entry.directory.entries[0].data.struct]}

        group = next(iter(resources(14).values()))
        _, _, count = struct.unpack_from("<HHH", group)
        frames = list(struct.iter_unpack("<BBBBHHIH", group[6:]))
        equal("PE icon frame count", len(frames), count)
        frame = max(frames, key=lambda entry: (entry[0] or 256) * (entry[1] or 256))
        pixels = resources(3)[frame[-1]]
        header = struct.pack("<HHHBBBBHHII", 0, 1, 1, *frame[:-1], 22)
        icon_matches(header + pixels)


def windows(directory, scratch):
    installer = one(directory.glob("*-setup.exe"))
    pe_metadata(installer)
    seven_zip = shutil.which("7z") or Path(os.environ["ProgramFiles"]) / "7-Zip/7z.exe"
    executable = CARGO["bin"][0]["name"] + ".exe"
    run(seven_zip, "e", installer, f"-o{scratch}", "-y", executable)
    pe_metadata(scratch / executable, application=True)


def apple(bundle, scratch, *, ios):
    contents = bundle if ios else bundle / "Contents"
    with (contents / "Info.plist").open("rb") as file:
        info = plistlib.load(file)
    for key, expected in [("CFBundleName", NAME), ("CFBundleDisplayName", NAME),
                          ("CFBundleIdentifier", IDENTIFIER),
                          ("CFBundleShortVersionString", NUMERIC_VERSION if ios else VERSION)]:
        equal(key, info.get(key), expected)
    executable = (contents if ios else contents / "MacOS") / info["CFBundleExecutable"]
    assert executable.is_file(), f"Missing bundle executable {executable}"
    if ios:
        equal("iOS build version", info.get("CFBundleVersion"), NUMERIC_VERSION)
        equal("iOS bundle platform", info.get("CFBundleSupportedPlatforms"), ["iPhoneSimulator"])
        run("codesign", "--verify", "--strict", bundle)
        assert (contents / "Assets.car").stat().st_size > 0
        for key in ("CFBundleIcons", "CFBundleIcons~ipad"):
            primary = info[key]["CFBundlePrimaryIcon"]
            files = primary["CFBundleIconFiles"]
            assert files, f"{key} has no primary icon files"
            for name in files:
                stem = name.removesuffix(".png")
                icons = list(contents.glob(f"{stem}.png")) + list(contents.glob(f"{stem}@*.png"))
                assert icons, f"{key} points to missing icon {name}"
                for icon in icons:
                    decoded = scratch / "decoded.png"
                    run("sips", "-s", "format", "png", icon, "--out", decoded)
                    icon_matches(decoded)
    else:
        icon = Path(info["CFBundleIconFile"])
        icon_matches(contents / "Resources" / (icon if icon.suffix else icon.with_suffix(".icns")))


def android(directory, _scratch):
    sdk = Path(os.environ.get("ANDROID_HOME") or os.environ["ANDROID_SDK_ROOT"])
    tool = "aapt2.exe" if os.name == "nt" else "aapt2"
    aapt = max(sdk.glob(f"build-tools/*/{tool}"), key=lambda path: tuple(map(int, re.findall(r"\d+", path.parent.name))))
    apks = [directory] if directory.is_file() else sorted(directory.rglob("*.apk"))
    assert apks, f"No APKs under {directory}"
    major, minor, patch = map(int, NUMERIC_VERSION.split("."))
    version_code = str(major * 1_000_000 + minor * 1_000 + patch + 1)
    for apk in apks:
        print(f"Checking {apk}", flush=True)
        badging = run(aapt, "dump", "badging", apk)
        package = next(line for line in badging.splitlines() if line.startswith("package: "))
        fields = dict(re.findall(r"(\w+)='([^']*)'", package))
        equal("APK package", fields["name"], IDENTIFIER)
        equal("APK versionName", fields["versionName"], VERSION)
        equal("APK versionCode", fields["versionCode"], version_code)
        label = next(line for line in badging.splitlines() if line.startswith("application-label:"))
        equal("APK application label", label.split(":", 1)[1].strip("'"), NAME)
        manifest = run(aapt, "dump", "xmltree", apk, "--file", "AndroidManifest.xml")
        icon_id = re.search(r":icon\([^)]*\)=@(0x[0-9a-f]+)", manifest)[1]
        resources = {section.split()[0]: re.findall(r"\(file\) (\S+)", section)
                     for section in re.split(r"(?m)^ +resource ", run(aapt, "dump", "resources", apk))[1:]}
        with zipfile.ZipFile(apk) as archive:
            icons = resources[icon_id]
            assert any(path.endswith(".png") for path in icons), "APK has no legacy launcher icon"
            layers = set()
            for icon in icons:
                if icon.endswith(".png"):
                    icon_matches(archive.read(icon))
                    continue
                xml = run(aapt, "dump", "xmltree", apk, "--file", icon)
                for layer, resource in re.findall(r"E: (foreground|monochrome)[^\n]*\n\s+A: .*:drawable\([^)]*\)=@(0x[0-9a-f]+)", xml):
                    layers.add(layer)
                    assert resources[resource], f"Empty adaptive icon layer {layer}"
                    for path in resources[resource]:
                        icon_matches(archive.read(path), inset=True, monochrome=layer == "monochrome")
            equal("adaptive icon layers", layers, {"foreground", "monochrome"})


class WebHead(HTMLParser):
    def __init__(self):
        super().__init__()
        self.title = ""
        self.in_title = False
        self.links = {}

    def handle_starttag(self, tag, attrs):
        if tag == "title":
            self.in_title = True
        if tag == "link":
            attrs = dict(attrs)
            if "rel" in attrs and "href" in attrs:
                self.links[attrs["rel"]] = attrs["href"]

    def handle_endtag(self, tag):
        if tag == "title":
            self.in_title = False

    def handle_data(self, data):
        if self.in_title:
            self.title += data


def web(host, _scratch):
    head = WebHead()
    head.feed((host / "index.html").read_text(encoding="utf-8"))
    equal("Web title", head.title, NAME)
    path = host / head.links["manifest"]
    manifest = json.loads(path.read_text(encoding="utf-8"))
    for key, expected in [("name", NAME), ("id", IDENTIFIER), ("version", VERSION)]:
        equal(f"Web manifest {key}", manifest.get(key), expected)
    icon_matches(host / head.links["icon"])
    icon_matches(host / head.links["apple-touch-icon"], size=180)
    equal("Web manifest icon sizes", {icon["sizes"] for icon in manifest["icons"]}, {"192x192", "512x512"})
    for icon in manifest["icons"]:
        icon_matches(path.parent / icon["src"], size=int(icon["sizes"].split("x")[0]))
    with (host / "pkg-web" / f"{CARGO['lib']['name']}_bg.wasm").open("rb") as file:
        equal("WebAssembly header", file.read(8), b"\x00asm\x01\x00\x00\x00")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Verify the packaged example against its application configuration.")
    parser.add_argument("platform", choices=["deb", "windows", "macos", "ios", "android", "web"])
    parser.add_argument("artifact", type=Path)
    args = parser.parse_args()
    print(f"Checking {args.platform} artifacts at {args.artifact}", flush=True)
    with tempfile.TemporaryDirectory() as directory:
        scratch = Path(directory)
        if args.platform in ("macos", "ios"):
            apple(args.artifact, scratch, ios=args.platform == "ios")
        else:
            {"deb": deb, "windows": windows, "android": android, "web": web}[args.platform](args.artifact, scratch)
