# Headless sessions

Enable the `headless` feature on the application's `tessera-ui` dependency. Use
the shared `EntryPoint::run_desktop()` startup path; headless workers select its
offscreen mode. Rendering requires a native GPU adapter.

Start the session supervisor from the application workspace:

```sh
cargo tessera headless -p my-app --width 800 --height 600 --frame-time-ms 16
```

After its `ready` event, send one JSON object per stdin line and read the matching
stdout response. Name a worker with `spawn`, then address it through `session`:

```jsonl
{"id":1,"cmd":"spawn","name":"preview"}
{"id":2,"cmd":"render","session":"preview","frames":1,"out":"target/preview.png"}
{"id":3,"cmd":"snapshot","session":"preview"}
{"id":4,"cmd":"input","session":"preview","kind":"press","x":100,"y":80,"button":"left"}
{"id":5,"cmd":"input","session":"preview","kind":"release","x":100,"y":80,"button":"left"}
{"id":6,"cmd":"render","session":"preview","duration_ms":160,"out":"target/after-click.png"}
{"id":7,"cmd":"snapshot","session":"preview"}
{"id":8,"cmd":"shutdown"}
```

Use `snapshot` for component bounds and accessibility labels, values, and state.
Apply queued input with `render` before reading updated output. Use `text` with a
`text` field for focused-editor input, and `input` with `kind: "scroll"` plus
`delta_x` / `delta_y` for scrolling. Input positions are window coordinates.

Use `resize` with physical-pixel `width` / `height` for another viewport;
`rebuild` restarts the selected worker after source changes. `list` enumerates
workers, `kill` stops one, and `shutdown` closes the supervisor.

Set `TESSERA_HEADLESS_SCALE_FACTOR` before starting the supervisor for DPI cases.
Each render advances the virtual clock by the configured frame step. See
[CLI tools](tooling.md) for profiling capture and analysis.
