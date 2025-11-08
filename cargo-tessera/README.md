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

For hot reload support, install `cargo-watch`:

```bash
cargo install cargo-watch
```

### Build for release

```bash
cargo tessera build --release
```

Cross-compile for a specific target:

```bash
cargo tessera build --release --target x86_64-pc-windows-msvc
```

## Commands

- `cargo tessera new <name>` - Create a new Tessera project
- `cargo tessera dev` - Start development server with built-in hot reload
- `cargo tessera build` - Build the project

## License

Licensed under either of [MIT](../LICENSE) or [Apache-2.0](../LICENSE) at your option.
