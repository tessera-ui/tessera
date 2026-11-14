# Tessera CLI tools

| Task | Command |
| --- | --- |
| Scaffold an app | `cargo tessera new my-app --template basic` or `--template blank` |
| Scaffold a plugin | `cargo tessera plugin new my-plugin --template basic` or `--template android` |
| Check compilation and Tessera call boundaries | `cargo tessera check -p my-app` |
| Apply compiler and safe Tessera fixes | `cargo tessera fix -p my-app` |
| Develop on desktop | `cargo tessera dev -p my-app` |
| Build a native release | `cargo tessera build -p my-app --release` |
| Package a desktop app | `cargo tessera package -p my-app --release` (requires cargo-packager) |
| Package for iOS | `cargo tessera ios build -p my-app --release` (macOS with Xcode; `--device` for IPA) |
| Develop on web | `cargo tessera web dev` |
| Build a web release | `cargo tessera web build --release` (requires Binaryen 110 or newer) |
| Initialize Android | `cargo tessera android init` |
| Run on Android | `cargo tessera android dev --device DEVICE_ID` |

Use `cargo tessera check -D warnings` to treat Tessera lint warnings as errors.
Review `MaybeIncorrect` suggestions manually.
Select individual lints by name, such as `-A tessera::redundant_render_slot`.
Use `#[tessera_ui::lint(allow(tessera::redundant_render_slot))]` on an item or
inline module tree for source overrides. Use `allow`, `warn`, `deny`, or `forbid`;
nested settings override inherited levels unless forbidden.
`--force-warn` and `--cap-lints` are command-line only.

Set `name`, `identifier`, and `icon` in `tessera-app.toml` for every platform.
Use a square SVG (recommended, with outlined text and embedded resources) or PNG
(prefer at least 1024 × 1024). Versions come from Cargo. Use `..res::application()`
as renderer defaults to apply the same name, icon, and launcher identity at runtime.

The app templates generate a shared library entrypoint and a thin desktop binary.
Use [headless sessions](headless.md) to render and interact without a window.

Capture frame profiling from desktop development or an offscreen session:

```sh
cargo tessera dev -p my-app --profiling-output target/desktop.jsonl
cargo tessera headless -p my-app --profiling-output target/headless.jsonl
cargo tessera profiling analyze target/headless.jsonl --top 30
```

Pass `--debug-dirty-overlay` to desktop `dev` or `build` to display dirty replay
regions. Profiling reports component build, layout, render preparation, encoding,
and submission CPU costs. Set `TESSERA_GPU_TIMING=1` for supported native windowed
or offscreen GPU timestamp capture. That mode waits for readback each frame, so measure frame cadence in a separate
run with GPU timing disabled.
