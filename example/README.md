# example

A Tessera app showcasing the component stack.

## Running

```bash
cargo run -p example
```

## Window rendering

The scaffold uses `window_insets(WindowInsets::SafeDrawing)` to keep controls
clear of system areas while extending its background to the window edges.
`WindowConfig::with_app_titlebar` and `top_app_bar().window_chrome(true)`
integrate the desktop title bar, including the native controls on macOS.
