# example

A minimal Tessera workspace app used to verify the current framework and
component stack.

## Running

```bash
cargo run -p example
```

## Window rendering

The window background extends behind system areas while interactive content uses
`WindowInsets::SafeDrawing`. Android keeps its system bars transparent. macOS uses
`WindowConfig::integrated_titlebar` to retain native window controls over the same
background.

Run the safe-area rendering regression with a native GPU adapter (Metal on macOS):

```bash
cargo test -p example app_paints_safe_areas_and_applies_live_insets_once -- --ignored
```

It checks opaque, continuous edge pixels and navigation placement across desktop,
portrait, and landscape insets. Native checks should also cover Android rotation
and keyboard visibility, and macOS window dragging, resizing, and fullscreen.
