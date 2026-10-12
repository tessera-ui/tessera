# cargo-tessera

CLI tool for the Tessera UI framework - streamline project scaffolding, development, and building.

## Installation

```bash
cargo install cargo-tessera
```

## Usage

### Create a new project

```bash
cargo tessera new my-app
cd my-app
```

### Start development server

```bash
cargo tessera dev
```
`cargo tessera dev` watches `src/`, `Cargo.toml`, and (if present) `build.rs`, then rebuilds and restarts the app whenever changes are saved. Pass `--verbose` to see the underlying `cargo` commands.

Enable profiler output during desktop dev:

```bash
cargo tessera dev -p example --profiling-output profiles/dev.jsonl
```

Enable dirty replay overlay during desktop dev:

```bash
cargo tessera dev -p example --debug-dirty-overlay
```

### Build for release

```bash
cargo tessera build --release
```

Cross-compile for a specific target:

```bash
cargo tessera build --release --target x86_64-pc-windows-msvc
```

Enable profiler instrumentation in desktop build artifacts:

```bash
cargo tessera build -p example --profiling-output profiles/build.jsonl
```

Enable dirty replay overlay in desktop build artifacts:

```bash
cargo tessera build -p example --debug-dirty-overlay
```

### Analyze profiler output

```bash
cargo tessera profiling analyze profiles/dev.jsonl --top 30
```

Optional CSV export:

```bash
cargo tessera profiling analyze profiles/dev.jsonl --csv profiles/components.csv
```

Android pull + analyze in one step:

```bash
cargo tessera profiling analyze-android \
  --package com.example.my_app \
  --device 8cd1353b \
  --remote-path files/tessera-profiler.jsonl \
  --pull-to profiles/android.jsonl
```

### Headless profiling

```bash
cargo tessera headless -p example --profiling-output target/headless-profile.jsonl
```

The JSONL session protocol supports synthetic input, frame rendering, snapshots,
and viewport resizing without opening a window. Profiler records include
component build, layout/draw calculation, render graph preparation
(`render_prepare_ns`), command encoding, and queue submission timings. These
measure CPU wall time, not GPU execution or desktop presentation.

Desktop `frame_total_ns` also includes publishing accessibility updates and
applying cursor/IME requests. Headless sessions omit these platform operations;
compare desktop traces when offscreen measurements do not reproduce a stutter.

Set `TESSERA_HEADLESS_GPU_TIMING=1` to add `render_gpu_ns` timestamp measurements
on supported native adapters. This mode waits for GPU readback after each frame;
the wait is excluded from CPU frame timings and changes CPU/GPU overlap. GPU
timestamps cover the main frame command buffer, including its render, compute,
and copy passes; separate submissions used to populate pipeline caches are
excluded. Use `WGPU_POWER_PREF=low` or `high` to compare integrated and discrete
adapters, and check the adapter name in the startup log.
Set `TESSERA_HEADLESS_SCALE_FACTOR=1.5` or `2` to reproduce high-DPI rendering;
the configured viewport dimensions remain physical pixels. Headless sessions
default to a scale factor of 1, and ready/snapshot responses include the factor.

Enable `RUST_LOG=tessera_ui::renderer::core::gpu_timing=debug` with GPU timing to
log GPU duration per render-graph pass and the final output blit. Draw stages
include any background capture needed by sampled effects; compute stages include
their input preservation and dispatches.

### Build for Android (experimental)

Make sure Android SDK/NDK are installed and `adb` is available in your PATH.

Android helpers live under the dedicated subcommand:

```bash
# Initialize the Android Gradle project (required once)
cargo tessera android init

# Build APK/AAB via Gradle
cargo tessera android build --release --format apk

# Run the app on a device/emulator (device id is required)
cargo tessera android dev --device 8cd1353b
```

`cargo tessera android dev` requires `--device <device_id>` (list devices with `adb devices`).

Enable profiler data on Android builds by passing a sandbox path:

```bash
cargo tessera android dev \
  --device 8cd1353b \
  --profiling-output files/tessera-profiler.jsonl
```

Enable dirty replay overlay on Android:

```bash
cargo tessera android dev \
  --device 8cd1353b \
  --debug-dirty-overlay
```

Then pull and analyze:

```bash
cargo tessera profiling analyze-android \
  --package com.example.my_app \
  --device 8cd1353b \
  --remote-path files/tessera-profiler.jsonl
```

## Commands

- `cargo tessera new <name>` - Create a new Tessera project
- `cargo tessera dev` - Start development server with automatic rebuild/restart
- `cargo tessera build` - Build desktop targets
- `cargo tessera profiling analyze <file>` - Analyze profiler JSONL output
- `cargo tessera profiling analyze-android` - Pull Android profiler JSONL via adb, then analyze
- `cargo tessera android <subcommand>` - Android helpers (`build`, `dev`)

## License

Licensed under either of [MIT](../LICENSE) or [Apache-2.0](../LICENSE) at your option.
