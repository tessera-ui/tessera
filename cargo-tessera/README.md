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
Hot reload is built in—`cargo tessera dev` watches `src/`, `Cargo.toml`, and (if
present) `build.rs`. Pass `--verbose` to see the underlying `cargo` commands.

### Build for release

```bash
cargo tessera build --release
```

Cross-compile for a specific target:

```bash
cargo tessera build --release --target x86_64-pc-windows-msvc
```

### Build for Android (experimental)

Install [`xbuild`](https://github.com/rust-mobile/xbuild) (`cargo install xbuild --features vendored`) and add Tessera metadata to your `Cargo.toml`:

```toml
[package.metadata.tessera.android]
package = "com.example.myapp"
arch = "arm64"
format = "apk"
```

Android helpers live under the dedicated subcommand:

```bash
# Build APK/AAB via xbuild
cargo tessera android build --release --format apk

# Run the app on a device/emulator (set --device if multiple devices are connected)
cargo tessera android dev --device adb:1234
```

Use `--arch`, `--package`, or `--format` to override metadata; if the build fails, install `xbuild` or run `x doctor` for diagnostics.

## Commands

- `cargo tessera new <name>` - Create a new Tessera project
- `cargo tessera dev` - Start development server with built-in hot reload
- `cargo tessera build` - Build desktop targets
- `cargo tessera android <subcommand>` - Android helpers (`build`, `dev`)

## License

Licensed under either of [MIT](../LICENSE) or [Apache-2.0](../LICENSE) at your option.
