# example

A minimal Tessera workspace app used to verify the current framework and
component stack.

## Running

```bash
cargo run -p example
```

## Window rendering

The scaffold uses `window_insets(WindowInsets::SafeDrawing)` to keep controls
clear of system areas while extending its background to the window edges.
`WindowConfig::with_app_titlebar` and `top_app_bar().window_chrome(true)`
integrate the desktop title bar, including the native controls on macOS.

Run the safe-area rendering regression with a native GPU adapter (Metal on macOS):

```bash
cargo test -p example app_paints_safe_areas_and_applies_live_insets_once -- --ignored
```

It checks opaque, continuous edge pixels and navigation placement across desktop,
portrait, and landscape insets. Native checks should also cover Android rotation
and keyboard visibility, and macOS window dragging, resizing, and fullscreen.
