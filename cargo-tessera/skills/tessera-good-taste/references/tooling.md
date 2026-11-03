# Tessera CLI tools

| Task | Command |
| --- | --- |
| Scaffold an app | `cargo tessera new my-app --template basic` or `--template blank` |
| Scaffold a plugin | `cargo tessera plugin new my-plugin --template basic` or `--template android` |
| Check compilation and Tessera call boundaries | `cargo tessera check -p my-app` |
| Develop on desktop | `cargo tessera dev -p my-app` |
| Build a native release | `cargo tessera build -p my-app --release` |
| Develop on web | `cargo tessera web dev` |
| Initialize Android | `cargo tessera android init` |
| Run on Android | `cargo tessera android dev --device DEVICE_ID` |

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
and submission CPU costs. Set `TESSERA_HEADLESS_GPU_TIMING=1` for supported
offscreen GPU timestamp capture; that mode waits for readback each frame.
