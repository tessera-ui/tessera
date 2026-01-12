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

### Build for release

```bash
cargo tessera build --release
```

Cross-compile for a specific target:

```bash
cargo tessera build --release --target x86_64-pc-windows-msvc
```

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

## Commands

- `cargo tessera new <name>` - Create a new Tessera project
- `cargo tessera dev` - Start development server with automatic rebuild/restart
- `cargo tessera build` - Build desktop targets
- `cargo tessera android <subcommand>` - Android helpers (`build`, `dev`)

## License

Licensed under either of [MIT](../LICENSE) or [Apache-2.0](../LICENSE) at your option.
