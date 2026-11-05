# cargo-tessera

CLI tool for the Tessera UI framework - streamline project scaffolding, development, and building.

## Installation

Requires Rust 1.98 or newer.

```bash
cargo install cargo-tessera
```

## Usage

### Create a new project

```bash
cargo tessera new my-app
cd my-app
```

App and plugin scaffolds include an `AGENTS.md` explaining how to install the
optional `tessera-good-taste` skill. Project creation does not install the skill.

### Install the agent skill

Run this command in a project when you want Tessera application guidance:

```bash
cargo tessera skill
```

The command shows a compact selector below your shell output, with **Local**
selected initially and the destination of each scope previewed. Use Up/Down or
Tab to select and Enter to confirm. Esc, q, or Ctrl-C cancels without
installing. The selector rows are removed when it exits and the terminal is
restored, including after cancellation and errors.

- Local: `<project>/.agents/skills/tessera-good-taste/` (current directory by default).
- Global: `~/.agents/skills/tessera-good-taste/`, using your operating-system home directory.

For scripts or non-interactive terminals, explicitly select one scope:

```bash
cargo tessera skill --local
cargo tessera skill --local --path my-app
cargo tessera skill --global
```

`--local` and `--global` are mutually exclusive and bypass the selector entirely;
they do not require a TTY. Without either flag, both stdin and stdout must be
terminals; otherwise the command fails with a scope-selection hint. `--path` is
local-only: it conflicts with `--global`, and the selector shows Global as
unavailable when `--path` is supplied.

Both scopes leave `AGENTS.md`, other skills, and unrelated files unchanged.
Matching files are left untouched; use `--force` to replace locally modified
bundled skill files. Invoke the installed skill as `$tessera-good-taste` in an
agent that supports its installation location, or read its `SKILL.md` directly.
Project creation still only generates guidance; it does not install the skill.

For agent-specific installation through the Skills CLI, select the skill from
this repository:

```bash
npx skills add tessera-ui/tessera --skill tessera-good-taste
```

The repository's `skills.sh.json` exposes `crates/cargo-tessera/skills/` as a skill
container. `SKILL.md` routes tasks to focused references, which are read as needed.

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

### Fix diagnostics

```bash
cargo tessera fix -p my-app
```

Runs `cargo fix`, applies machine-applicable Tessera suggestions, and checks the
result. Review uncertain suggestions manually. Cargo fix options such as
`--allow-dirty`, `--allow-staged`, and `--allow-no-vcs` are supported.

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

Licensed under either of [MIT](../../LICENSE) or [Apache-2.0](../../LICENSE) at your option.
