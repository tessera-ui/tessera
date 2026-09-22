//! Headless, session-based rendering support for Tessera applications.
//!
//! ## Usage
//!
//! Drive a Tessera application without a window: render frames into an
//! offscreen target, inject synthetic pointer/IME input, and read the result
//! back as RGBA8 pixels or a PNG file. This powers `cargo tessera headless`,
//! which supervises one or more of these sessions over a JSONL stdio protocol.
//!
//! [`HeadlessRenderer`] is the library entry point; [`run_headless`] implements
//! the worker side of the `cargo tessera headless` protocol and is invoked by
//! [`crate::entry_point::EntryPoint::run_desktop`] when `TESSERA_HEADLESS` is set.

use std::{
    io::{BufRead, BufReader, Write},
    num::NonZero,
    path::Path,
    time::Duration,
};

use parking_lot::RwLock;
use serde::Deserialize;
use serde_json::{Value, json};
use tracing::info;

use crate::{
    NodeId, Px, PxPosition, PxSize,
    build_tree::build_component_tree,
    component_tree::{ComponentNodeMetaDatas, ComponentNodeTree, ComputeMode, ComputeParams},
    context::{reset_component_context_tracking, reset_context_read_dependencies},
    cursor::{
        CursorEventContent, CursorState, GestureState, MOUSE_POINTER_ID, PointerChange, PointerId,
        PressKeyEventType, ScrollDeltaUnit, ScrollEventContent, ScrollEventSource,
    },
    dp::SCALE_FACTOR,
    focus::flush_pending_focus_callbacks,
    ime_state::ImeState,
    keyboard_state::KeyboardState,
    pipeline_context::PipelineContext,
    render_module::RenderModule,
    renderer::{RenderCore, TesseraConfig, composite::expand_composites, core::OffscreenReadbackError},
    runtime::{
        TesseraRuntime, begin_frame_clock, clear_persistent_focus_handles, clear_redraw_waker,
        reset_build_invalidations, reset_component_replay_tracking, reset_focus_read_dependencies,
        reset_frame_clock, reset_layout_dirty_tracking, reset_render_slot_read_dependencies,
        reset_slots, reset_state_read_dependencies, take_layout_dirty_nodes,
        tick_frame_nanos_receivers,
    },
    time::Instant,
};

/// Environment variable that switches a Tessera application into headless mode.
pub const HEADLESS_ENV: &str = "TESSERA_HEADLESS";
/// Environment variable overriding the initial headless surface width.
pub const HEADLESS_WIDTH_ENV: &str = "TESSERA_HEADLESS_WIDTH";
/// Environment variable overriding the initial headless surface height.
pub const HEADLESS_HEIGHT_ENV: &str = "TESSERA_HEADLESS_HEIGHT";
/// Environment variable overriding the virtual frame step in milliseconds.
pub const HEADLESS_FRAME_TIME_MS_ENV: &str = "TESSERA_HEADLESS_FRAME_TIME_MS";

/// Returns whether the current process should run in headless mode.
pub fn headless_mode_requested() -> bool {
    match std::env::var(HEADLESS_ENV) {
        Ok(value) => {
            let value = value.trim();
            !value.is_empty() && value != "0" && !value.eq_ignore_ascii_case("false")
        }
        Err(_) => false,
    }
}

/// Errors produced by headless session setup and the JSONL worker loop.
#[derive(Debug)]
pub enum HeadlessError {
    /// An I/O error while reading commands or writing responses.
    Io(std::io::Error),
    /// A malformed JSONL command.
    Json(serde_json::Error),
    /// The GPU render core could not be initialized.
    GpuInit(String),
    /// Reading pixels back from the offscreen target failed.
    Readback(OffscreenReadbackError),
}

impl std::fmt::Display for HeadlessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "headless io error: {err}"),
            Self::Json(err) => write!(f, "headless protocol error: {err}"),
            Self::GpuInit(err) => write!(f, "headless gpu init failed: {err}"),
            Self::Readback(err) => write!(f, "headless readback failed: {err}"),
        }
    }
}

impl std::error::Error for HeadlessError {}

impl From<std::io::Error> for HeadlessError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serde_json::Error> for HeadlessError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

impl From<OffscreenReadbackError> for HeadlessError {
    fn from(err: OffscreenReadbackError) -> Self {
        Self::Readback(err)
    }
}

/// Configuration for a headless renderer session.
#[derive(Clone, Debug)]
pub struct HeadlessConfig {
    /// Offscreen target width in physical pixels.
    pub width: u32,
    /// Offscreen target height in physical pixels.
    pub height: u32,
    /// MSAA sample count for render pipelines.
    pub sample_count: u32,
    /// Offscreen target format; RGBA8 is required for readback and PNG output.
    pub format: wgpu::TextureFormat,
    /// Virtual time advanced per rendered frame.
    pub frame_time: Duration,
}

impl Default for HeadlessConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            sample_count: 1,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            frame_time: Duration::from_nanos(16_666_667),
        }
    }
}

/// A deterministic, non-interactive Tessera renderer session.
///
/// Frames advance on a virtual clock, so a fixed input script always produces
/// the same output regardless of wall-clock timing.
pub struct HeadlessRenderer {
    core: RenderCore,
    entry: Box<dyn Fn()>,
    cursor: CursorState,
    keyboard: KeyboardState,
    ime: ImeState,
    size: PxSize,
    physical_size: winit::dpi::PhysicalSize<u32>,
    frame_index: u64,
    frame_origin: Instant,
    current_frame_nanos: u64,
    frame_time: Duration,
}

impl HeadlessRenderer {
    /// Creates a headless renderer for the given entry point and render modules.
    pub fn new(
        entry: Box<dyn Fn()>,
        modules: Vec<Box<dyn RenderModule>>,
        config: HeadlessConfig,
    ) -> Result<Self, HeadlessError> {
        let width = config.width.max(1);
        let height = config.height.max(1);
        let physical_size = winit::dpi::PhysicalSize::new(width, height);

        let mut core = pollster::block_on(RenderCore::new_offscreen(
            physical_size,
            config.format,
            config.sample_count,
        ));

        // Headless sessions map logical pixels to physical pixels 1:1.
        let _ = SCALE_FACTOR.set(RwLock::new(1.0));

        {
            let mut context = PipelineContext::new(&mut core);
            for module in &modules {
                module.register_pipelines(&mut context);
            }
        }

        reset_headless_runtime(physical_size);

        info!(
            "Headless renderer initialized at {}x{} ({} samples, {:?})",
            width, height, config.sample_count, config.format
        );

        Ok(Self {
            core,
            entry,
            cursor: CursorState::default(),
            keyboard: KeyboardState::default(),
            ime: ImeState::default(),
            size: PxSize::new(Px(width as i32), Px(height as i32)),
            physical_size,
            frame_index: 0,
            frame_origin: Instant::now(),
            current_frame_nanos: 0,
            frame_time: config.frame_time.max(Duration::from_nanos(1)),
        })
    }

    /// Returns the current offscreen size in physical pixels.
    pub fn physical_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.physical_size
    }

    /// Returns the number of frames rendered so far.
    pub fn frame_index(&self) -> u64 {
        self.frame_index
    }

    /// Queues a synthetic pointer change.
    ///
    /// Movement changes also update the tracked cursor position so hit testing
    /// observes the new location on the next frame.
    pub fn inject_pointer(&mut self, change: PointerChange) {
        if let CursorEventContent::Moved(position) = &change.content {
            self.cursor.update_position(*position);
        }
        self.cursor.push_event(change);
    }

    /// Moves the given pointer to `position`.
    pub fn pointer_move(&mut self, pointer_id: PointerId, position: PxPosition) {
        self.cursor.update_position(position);
        self.cursor.push_event(PointerChange {
            timestamp: Instant::now(),
            pointer_id,
            content: CursorEventContent::Moved(position),
            gesture_state: GestureState::TapCandidate,
            consumed: false,
        });
    }

    /// Presses a pointer button at `position`.
    pub fn pointer_press(
        &mut self,
        pointer_id: PointerId,
        position: PxPosition,
        button: PressKeyEventType,
    ) {
        self.cursor.update_position(position);
        self.cursor.push_event(PointerChange {
            timestamp: Instant::now(),
            pointer_id,
            content: CursorEventContent::Pressed(button),
            gesture_state: GestureState::TapCandidate,
            consumed: false,
        });
    }

    /// Releases a pointer button at `position`.
    pub fn pointer_release(
        &mut self,
        pointer_id: PointerId,
        position: PxPosition,
        button: PressKeyEventType,
    ) {
        self.cursor.update_position(position);
        self.cursor.push_event(PointerChange {
            timestamp: Instant::now(),
            pointer_id,
            content: CursorEventContent::Released(button),
            gesture_state: GestureState::TapCandidate,
            consumed: false,
        });
    }

    /// Queues a mouse-wheel scroll at the current pointer position.
    pub fn pointer_scroll(&mut self, pointer_id: PointerId, delta_x: f32, delta_y: f32) {
        self.cursor.push_event(PointerChange {
            timestamp: Instant::now(),
            pointer_id,
            content: CursorEventContent::Scroll(ScrollEventContent {
                delta_x,
                delta_y,
                unit: ScrollDeltaUnit::Pixel,
                source: ScrollEventSource::Wheel,
            }),
            gesture_state: GestureState::TapCandidate,
            consumed: false,
        });
    }

    /// Queues a keyboard event.
    ///
    /// Note that winit does not allow constructing `KeyEvent` values outside of
    /// its own crates; callers must forward events they already received. For
    /// synthetic text entry use [`HeadlessRenderer::inject_text`].
    pub fn inject_key_event(&mut self, event: winit::event::KeyEvent) {
        self.keyboard.push_event(event);
    }

    /// Queues an IME event, used for synthetic text entry.
    pub fn inject_ime_event(&mut self, event: winit::event::Ime) {
        self.ime.push_event(event);
    }

    /// Commits `text` as IME input for the next frame.
    pub fn inject_text(&mut self, text: &str) {
        self.ime.push_event(winit::event::Ime::Commit(text.to_string()));
    }

    /// Advances the virtual clock by one frame and renders it.
    pub fn step_frame(&mut self) {
        self.current_frame_nanos = self
            .current_frame_nanos
            .saturating_add(self.frame_time.as_nanos().min(u64::MAX as u128) as u64);
        let frame_time = self.frame_origin + Duration::from_nanos(self.current_frame_nanos);
        begin_frame_clock(frame_time);
        // Tick frame-nanos receivers before the build so their state writes are
        // consumed by the current recomposition pass.
        tick_frame_nanos_receivers();

        let _ = build_component_tree(&self.entry);

        let cursor_position = self.cursor.position();
        let pointer_changes = self.cursor.take_events();
        let keyboard_events = self.keyboard.take_events();
        let ime_events = self.ime.take_events();
        let modifiers = self.keyboard.modifiers();
        let layout_dirty_nodes = take_layout_dirty_nodes();
        let screen_size = self.size;

        self.core.compute_resource_manager_mut().clear();

        let core = &mut self.core;
        let graph = TesseraRuntime::with_mut(|runtime| {
            let (gpu, compute_resource_manager) = core.record_resources();
            let (graph, _requests, _diagnostics, _record_cost, _move, _reveal) = runtime
                .component_tree
                .compute(
                    ComputeParams {
                        screen_size,
                        cursor_position,
                        pointer_changes,
                        keyboard_events,
                        ime_events,
                        retry_focus_move: None,
                        retry_focus_reveal: false,
                        modifiers,
                        layout_dirty_nodes: &layout_dirty_nodes,
                    },
                    ComputeMode::Full {
                        compute_resource_manager,
                        gpu,
                    },
                );
            graph
        });
        flush_pending_focus_callbacks();

        let frame_index = self.frame_index;
        let (composite_context, composite_registry) =
            self.core.composite_context_parts(screen_size, frame_index);
        let graph = expand_composites(graph, composite_context, composite_registry);
        let execution = graph.into_execution();
        #[cfg(feature = "debug-dirty-overlay")]
        #[cfg(feature = "debug-dirty-overlay")]
        self.core.render(execution, &[]);
        #[cfg(not(feature = "debug-dirty-overlay"))]
        self.core.render(execution);
        self.cursor.frame_cleanup();
        self.frame_index = self.frame_index.wrapping_add(1);
    }

    /// Renders exactly `frames` frames (at least one).
    pub fn render_frames(&mut self, frames: u32) {
        for _ in 0..frames.max(1) {
            self.step_frame();
        }
    }

    /// Renders frames until at least `duration` of virtual time has elapsed.
    pub fn render_for_duration(&mut self, duration: Duration) {
        let mut elapsed = Duration::ZERO;
        loop {
            self.step_frame();
            elapsed += self.frame_time;
            if elapsed >= duration {
                break;
            }
        }
    }

    /// Reads the last rendered frame back as tightly packed RGBA8 pixels.
    pub fn read_pixels(&self) -> Result<Vec<u8>, HeadlessError> {
        Ok(self.core.read_offscreen_rgba()?)
    }

    /// Resizes the offscreen target and updates the runtime window size.
    pub fn resize(&mut self, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);
        let physical_size = winit::dpi::PhysicalSize::new(width, height);
        self.physical_size = physical_size;
        self.size = PxSize::new(Px(width as i32), Px(height as i32));
        self.core.resize(physical_size);
        self.core.resize_surface();
        TesseraRuntime::with_mut(|runtime| runtime.window_size = [width, height]);
    }

    /// Captures a semantic snapshot of the component tree as JSON.
    ///
    /// Each node reports its component function name, role, absolute bounds and
    /// accessibility metadata (role, label, value, key and state flags), which
    /// is what an automated agent needs to locate and target widgets.
    pub fn snapshot(&self) -> Value {
        TesseraRuntime::with(|runtime| {
            let tree = runtime.component_tree.tree();
            let metadatas = runtime.component_tree.metadatas();
            let root = tree.get_node_id_at(NonZero::new(1).expect("root index is non-zero"));
            let mut counter = 0u64;
            let root_value = match root {
                Some(root) => snapshot_node(tree, metadatas, root, &mut counter),
                None => Value::Null,
            };
            json!({
                "width": self.physical_size.width,
                "height": self.physical_size.height,
                "frame_index": self.frame_index,
                "node_count": counter,
                "root": root_value,
            })
        })
    }
}

fn snapshot_node(
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    node_id: NodeId,
    counter: &mut u64,
) -> Value {
    *counter += 1;
    let node = tree.get(node_id);
    let fn_name = node
        .map(|node| node.get().fn_name.clone())
        .unwrap_or_else(|| "<unknown>".to_string());
    let role = node
        .map(|node| format!("{:?}", node.get().role))
        .unwrap_or_else(|| "<unknown>".to_string());

    let metadata = metadatas.get(&node_id);
    let bounds = metadata
        .and_then(|metadata| {
            let position = metadata.abs_position?;
            let size = metadata.computed_data?;
            Some(json!({
                "x": position.x.0,
                "y": position.y.0,
                "width": size.width.0,
                "height": size.height.0,
            }))
        })
        .unwrap_or(Value::Null);

    let accessibility = metadata
        .and_then(|metadata| metadata.accessibility.as_ref())
        .map(|accessibility| {
            json!({
                "role": accessibility.role.map(|role| format!("{role:?}")),
                "label": accessibility.label,
                "value": accessibility.value,
                "key": accessibility.key,
                "focusable": accessibility.focusable,
                "focused": accessibility.focused,
                "disabled": accessibility.disabled,
            })
        })
        .unwrap_or(Value::Null);

    let children: Vec<Value> = node_id
        .children(tree)
        .map(|child| snapshot_node(tree, metadatas, child, counter))
        .collect();

    json!({
        "fn_name": fn_name,
        "role": role,
        "bounds": bounds,
        "accessibility": accessibility,
        "children": children,
    })
}

fn reset_headless_runtime(size: winit::dpi::PhysicalSize<u32>) {
    TesseraRuntime::with_mut(|runtime| {
        runtime.component_tree.reset();
        runtime.cursor_icon_request = None;
        runtime.window_minimized = false;
        runtime.window_size = [size.width, size.height];
    });
    reset_layout_dirty_tracking();
    reset_component_replay_tracking();
    reset_focus_read_dependencies();
    reset_render_slot_read_dependencies();
    reset_state_read_dependencies();
    reset_component_context_tracking();
    reset_context_read_dependencies();
    reset_build_invalidations();
    reset_frame_clock();
    clear_redraw_waker();
    clear_persistent_focus_handles();
    reset_slots();
}

/// Runs the worker side of the `cargo tessera headless` protocol.
///
/// Reads one JSON command per line from stdin and writes one JSON response per
/// line to stdout. Diagnostics are written to stderr so stdout stays a clean
/// JSONL stream.
pub fn run_headless(
    entry: Box<dyn Fn()>,
    modules: Vec<Box<dyn RenderModule>>,
    config: TesseraConfig,
) -> Result<(), HeadlessError> {
    init_headless_tracing();

    let headless_config = HeadlessConfig {
        width: env_u32(HEADLESS_WIDTH_ENV).unwrap_or(800),
        height: env_u32(HEADLESS_HEIGHT_ENV).unwrap_or(600),
        sample_count: config.sample_count.max(1),
        ..HeadlessConfig::default()
    };
    let mut frame_time = headless_config.frame_time;
    if let Some(ms) = env_u32(HEADLESS_FRAME_TIME_MS_ENV) {
        frame_time = Duration::from_millis(ms.max(1) as u64);
    }

    let mut renderer = HeadlessRenderer::new(entry, modules, headless_config)?;
    renderer.frame_time = frame_time;

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    write_line(
        &mut out,
        json!({
            "event": "ready",
            "width": renderer.physical_size.width,
            "height": renderer.physical_size.height,
        }),
    )?;

    let stdin = std::io::stdin();
    let reader = BufReader::new(stdin.lock());
    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let request = match serde_json::from_str::<WorkerRequest>(line) {
            Ok(request) => request,
            Err(err) => {
                write_line(
                    &mut out,
                    json!({"ok": false, "error": format!("invalid command: {err}")}),
                )?;
                continue;
            }
        };

        let is_shutdown = matches!(&request, WorkerRequest::Shutdown { .. });
        let (id, mut response) = handle_request(&mut renderer, request)?;
        if let Some(map) = response.as_object_mut() {
            if let Some(id) = id {
                map.insert("id".to_string(), json!(id));
            }
        }
        write_line(&mut out, response)?;

        if is_shutdown {
            break;
        }
    }

    Ok(())
}

fn handle_request(
    renderer: &mut HeadlessRenderer,
    request: WorkerRequest,
) -> Result<(Option<u64>, Value), HeadlessError> {
    match request {
        WorkerRequest::Input {
            id,
            kind,
            pointer_id,
            x,
            y,
            delta_x,
            delta_y,
            button,
        } => {
            let pointer_id = pointer_id.unwrap_or(MOUSE_POINTER_ID);
            let position = PxPosition::new(Px(x.unwrap_or(0.0) as i32), Px(y.unwrap_or(0.0) as i32));
            let button = button.unwrap_or_default().to_press_key();
            match kind {
                InputKind::Press => renderer.pointer_press(pointer_id, position, button),
                InputKind::Release => renderer.pointer_release(pointer_id, position, button),
                InputKind::Move => renderer.pointer_move(pointer_id, position),
                InputKind::Scroll => renderer.pointer_scroll(
                    pointer_id,
                    delta_x.unwrap_or(0.0),
                    delta_y.unwrap_or(0.0),
                ),
            }
            Ok((id, json!({"ok": true, "kind": kind.as_str()})))
        }
        WorkerRequest::Text { id, text } => {
            renderer.inject_text(&text);
            Ok((id, json!({"ok": true, "chars": text.chars().count()})))
        }
        WorkerRequest::Render {
            id,
            frames,
            duration_ms,
            out,
        } => {
            let rendered = if let Some(duration_ms) = duration_ms {
                renderer.render_for_duration(Duration::from_millis(duration_ms.max(1)));
                None
            } else {
                let frames = frames.unwrap_or(1).max(1);
                renderer.render_frames(frames);
                Some(frames)
            };

            let size = renderer.physical_size();
            let mut response = json!({
                "ok": true,
                "width": size.width,
                "height": size.height,
                "frame_index": renderer.frame_index(),
            });
            if let Some(frames) = rendered {
                response["frames"] = json!(frames);
            }

            if let Some(out_path) = out {
                let pixels = renderer.read_pixels()?;
                let png = encode_png_rgba(size.width, size.height, &pixels);
                std::fs::write(&out_path, &png)?;
                response["path"] = json!(out_path);
                response["bytes"] = json!(png.len());
            }
            Ok((id, response))
        }
        WorkerRequest::Snapshot { id } => {
            let snapshot = renderer.snapshot();
            Ok((id, json!({"ok": true, "snapshot": snapshot})))
        }
        WorkerRequest::Resize { id, width, height } => {
            renderer.resize(width, height);
            let size = renderer.physical_size();
            Ok((
                id,
                json!({"ok": true, "width": size.width, "height": size.height}),
            ))
        }
        WorkerRequest::Shutdown { id } => {
            let _ = id;
            Ok((id, json!({"ok": true, "event": "shutdown"})))
        }
    }
}

fn write_line(out: &mut impl Write, value: Value) -> Result<(), HeadlessError> {
    let encoded = serde_json::to_string(&value)?;
    out.write_all(encoded.as_bytes())?;
    out.write_all(b"\n")?;
    out.flush()?;
    Ok(())
}

fn env_u32(key: &str) -> Option<u32> {
    std::env::var(key)
        .ok()
        .and_then(|value| value.trim().parse::<u32>().ok())
}

fn init_headless_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("error,tessera_ui=info"));
    // Keep stdout reserved for the JSONL protocol.
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum InputKind {
    #[default]
    Press,
    Release,
    Move,
    Scroll,
}

impl InputKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Press => "press",
            Self::Release => "release",
            Self::Move => "move",
            Self::Scroll => "scroll",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ButtonKind {
    #[default]
    Left,
    Right,
    Middle,
}

impl ButtonKind {
    fn to_press_key(self) -> PressKeyEventType {
        match self {
            Self::Left => PressKeyEventType::Left,
            Self::Right => PressKeyEventType::Right,
            Self::Middle => PressKeyEventType::Middle,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
enum WorkerRequest {
    Input {
        #[serde(default)]
        id: Option<u64>,
        kind: InputKind,
        #[serde(default)]
        pointer_id: Option<u64>,
        #[serde(default)]
        x: Option<f32>,
        #[serde(default)]
        y: Option<f32>,
        #[serde(default)]
        delta_x: Option<f32>,
        #[serde(default)]
        delta_y: Option<f32>,
        #[serde(default)]
        button: Option<ButtonKind>,
    },
    Text {
        #[serde(default)]
        id: Option<u64>,
        text: String,
    },
    Render {
        #[serde(default)]
        id: Option<u64>,
        #[serde(default)]
        frames: Option<u32>,
        #[serde(default)]
        duration_ms: Option<u64>,
        #[serde(default)]
        out: Option<String>,
    },
    Snapshot {
        #[serde(default)]
        id: Option<u64>,
    },
    Resize {
        #[serde(default)]
        id: Option<u64>,
        width: u32,
        height: u32,
    },
    Shutdown {
        #[serde(default)]
        id: Option<u64>,
    },
}

/// Encodes tightly packed RGBA8 pixels as a PNG image.
///
/// Uses stored (uncompressed) DEFLATE blocks so no compression dependency is
/// required; the resulting file is larger than a compressed PNG but fully
/// valid and readable by any PNG consumer.
pub fn encode_png_rgba(width: u32, height: u32, pixels: &[u8]) -> Vec<u8> {
    let row_bytes = width as usize * 4;
    let mut raw = Vec::with_capacity((height as usize) * (1 + row_bytes));
    for row in 0..height as usize {
        raw.push(0);
        let start = row * row_bytes;
        raw.extend_from_slice(&pixels[start..start + row_bytes]);
    }

    let mut out = Vec::with_capacity(raw.len() + 1024);
    out.extend_from_slice(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);

    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8); // bit depth
    ihdr.push(6); // color type: RGBA
    ihdr.push(0); // compression method
    ihdr.push(0); // filter method
    ihdr.push(0); // interlace method
    write_png_chunk(&mut out, b"IHDR", &ihdr);
    write_png_chunk(&mut out, b"IDAT", &zlib_store(&raw));
    write_png_chunk(&mut out, b"IEND", &[]);
    out
}

fn write_png_chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(data);

    let mut crc = Crc32::new();
    crc.update(kind);
    crc.update(data);
    out.extend_from_slice(&crc.finish().to_be_bytes());
}

fn zlib_store(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() + data.len() / 65_535 * 5 + 16);
    // zlib header: deflate, 32K window, default compression level.
    out.push(0x78);
    out.push(0x01);

    if data.is_empty() {
        out.extend_from_slice(&[0x01, 0x00, 0x00, 0xFF, 0xFF]);
    }

    let mut offset = 0;
    while offset < data.len() {
        let remaining = data.len() - offset;
        let block_len = remaining.min(65_535);
        let is_final = offset + block_len >= data.len();
        out.push(if is_final { 0x01 } else { 0x00 });
        let len = block_len as u16;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(&data[offset..offset + block_len]);
        offset += block_len;
    }

    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

fn adler32(data: &[u8]) -> u32 {
    const MOD: u32 = 65_521;
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &byte in data {
        a = (a + byte as u32) % MOD;
        b = (b + a) % MOD;
    }
    (b << 16) | a
}

struct Crc32 {
    table: [u32; 256],
    state: u32,
}

impl Crc32 {
    fn new() -> Self {
        let mut table = [0u32; 256];
        for (index, entry) in table.iter_mut().enumerate() {
            let mut value = index as u32;
            for _ in 0..8 {
                value = if value & 1 != 0 {
                    0xEDB8_8320 ^ (value >> 1)
                } else {
                    value >> 1
                };
            }
            *entry = value;
        }
        Self {
            table,
            state: 0xFFFF_FFFF,
        }
    }

    fn update(&mut self, data: &[u8]) {
        for &byte in data {
            let index = ((self.state ^ byte as u32) & 0xFF) as usize;
            self.state = self.table[index] ^ (self.state >> 8);
        }
    }

    fn finish(self) -> u32 {
        self.state ^ 0xFFFF_FFFF
    }
}

/// Writes a PNG file containing `pixels` and returns the byte length written.
pub fn write_png_file(
    path: impl AsRef<Path>,
    width: u32,
    height: u32,
    pixels: &[u8],
) -> Result<usize, HeadlessError> {
    let png = encode_png_rgba(width, height, pixels);
    std::fs::write(path, &png)?;
    Ok(png.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adler32_matches_reference_values() {
        assert_eq!(adler32(b""), 1);
        assert_eq!(adler32(b"Wikipedia"), 0x11E6_0398);
    }

    #[test]
    fn crc32_matches_reference_value() {
        let mut crc = Crc32::new();
        crc.update(b"123456789");
        assert_eq!(crc.finish(), 0xCBF4_3926);
    }

    #[test]
    fn png_encoder_emits_valid_structure() {
        let width = 2;
        let height = 1;
        let pixels = vec![255, 0, 0, 255, 0, 255, 0, 255];
        let png = encode_png_rgba(width, height, &pixels);

        assert_eq!(&png[..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
        assert_eq!(&png[12..16], b"IHDR");
        assert_eq!(&png[16..20], &2u32.to_be_bytes());
        assert_eq!(&png[20..24], &1u32.to_be_bytes());
        assert_eq!(png[png.len() - 8..png.len() - 4], *b"IEND");
        assert!(png.windows(4).any(|window| window == b"IDAT"));
    }

    #[test]
    fn headless_mode_detects_env_flag() {
        // The default (unset) state must keep the desktop path untouched.
        // SAFETY: this test only reads the variable it manages.
        unsafe {
            std::env::remove_var(HEADLESS_ENV);
        }
        assert!(!headless_mode_requested());
    }
}
