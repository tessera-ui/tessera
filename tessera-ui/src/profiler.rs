//! # Performance Profiler
//!
//! ## Usage
//!
//! Stream JSONL frame timing for external analyzer tools.
//!
//! ## Format
//!
//! Here is an example of the output file:
//!
//! Note that the frame record shown here is formatted for readability. The
//! actual output file contains one JSON object per line to support streaming
//! processing and parsing.
//!
//! ```jsonl
//! {"version":3,"format":"tessera-profiler","generated_at":"1767008877"}
//! {
//!   "type": "frame",
//!   "frame": 0,
//!   "build_mode": "partial_replay",
//!   "redraw_reasons": ["mouse_input"],
//!   "inter_frame_wait_ns": 16564000,
//!   "render_time_ns": 1349800,
//!   "frame_total_ns": 57486200,
//!   "components": [
//!     {
//!       "id": "1",
//!       "fn_name": "entry_wrapper",
//!       "abs_pos": { "x": 0, "y": 0 },
//!       "size": { "w": 51, "h": 48 },
//!       "layout_cache_hit": true,
//!       "phases": { "build_ns": 55548600, "measure_ns": 55549000 },
//!       "children": [
//!         {
//!           "id": "2",
//!           "fn_name": "app",
//!           "abs_pos": { "x": 0, "y": 0 },
//!           "size": { "w": 51, "h": 48 },
//!           "layout_cache_hit": false,
//!           "phases": { "build_ns": 54978700, "measure_ns": 54979600 },
//!           "children": [
//!             {
//!               "id": "3",
//!               "fn_name": "text",
//!               "abs_pos": { "x": 0, "y": 0 },
//!               "size": { "w": 51, "h": 48 },
//!               "layout_cache_hit": true,
//!               "phases": { "build_ns": 54976500, "measure_ns": 54977000 },
//!               "children": [
//!                 {
//!                   "id": "4",
//!                   "fn_name": "modifier_semantics",
//!                   "abs_pos": { "x": 0, "y": 0 },
//!                   "size": { "w": 51, "h": 48 },
//!                   "phases": {
//!                     "build_ns": 8500,
//!                     "measure_ns": 54974800,
//!                     "input_ns": 7300
//!                   },
//!                   "children": [
//!                     {
//!                       "id": "5",
//!                       "fn_name": "text_inner",
//!                       "abs_pos": { "x": 0, "y": 0 },
//!                       "size": { "w": 51, "h": 48 },
//!                       "phases": {
//!                         "build_ns": 54948400,
//!                         "measure_ns": 54960100
//!                       },
//!                       "children": []
//!                     }
//!                   ]
//!                 }
//!               ]
//!             }
//!           ]
//!         }
//!       ]
//!     }
//!   ]
//! }
//! ```
//!
//! ### Frame Header
//!
//! The first line is a header that describes the profiler output format,
//! including version and generation timestamp.
//!
//! See [`FrameHeader`] for equivalent Rust structure.
//!
//! ### Trace Events
//!
//! Each subsequent line is a tagged event record.
//!
//! `layout_cache_hit` is an optional boolean on each component record. When
//! present, it indicates whether the layout cache was hit for that node in the
//! current frame.
//!
//! See [`FrameEventRecord`] and [`ComponentRecord`] for equivalent Rust
//! structures.
use std::{
    collections::HashMap,
    fs::{File, OpenOptions, create_dir_all},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    thread,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use indextree::NodeId;
use serde::Serialize;
use serde_json;
use tracing::error;

use crate::component_tree::LayoutFrameDiagnostics;

/// Profiling phases that can be emitted.
#[derive(Clone, Copy)]
pub enum Phase {
    /// Component build stage.
    Build,
    /// Layout measurement stage.
    Measure,
    /// Layout record stage (generate draw/compute commands).
    Record,
    /// Input handling stage.
    Input,
    /// GPU render stage (frame-level).
    RenderFrame,
}

/// Component tree build strategy used for a frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildMode {
    /// Full tree build because no previous tree exists.
    FullInitial,
    /// Partial dirty-subtree replay succeeded.
    PartialReplay,
    /// Build stage skipped because there were no invalidations.
    SkipNoInvalidation,
}

/// Reason that caused a redraw request.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RedrawReason {
    /// Initial frame request during startup/resume.
    Startup,
    /// Window resize event.
    WindowResized,
    /// Cursor move event.
    CursorMoved,
    /// Cursor leave event.
    CursorLeft,
    /// Mouse button input event.
    MouseInput,
    /// Mouse wheel event.
    MouseWheel,
    /// Touch event.
    TouchInput,
    /// Scale factor change event.
    ScaleFactorChanged,
    /// Keyboard input event.
    KeyboardInput,
    /// Keyboard modifier state change event.
    ModifiersChanged,
    /// IME event.
    ImeEvent,
    /// Focus change event.
    FocusChanged,
    /// Runtime state invalidation requires next frame.
    RuntimeInvalidation,
    /// Frame awaiter callback requires next frame.
    RuntimeFrameAwaiter,
}

/// Source category for redraw wake events.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WakeSource {
    /// Triggered by the renderer lifecycle.
    Lifecycle,
    /// Triggered by a window/input event.
    WindowEvent,
    /// Triggered by runtime pending work.
    Runtime,
}

/// Runtime lifecycle event kind emitted by profiler.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeEventKind {
    /// Renderer resumed and resources were created.
    Resumed,
    /// Renderer suspended and resources were released.
    Suspended,
}

#[derive(Clone)]
struct Sample {
    phase: Phase,
    frame_idx: u64,
    node_id: Option<NodeId>,
    parent_node_id: Option<NodeId>,
    fn_name: Option<String>,
    abs_pos: Option<(i32, i32)>,
    start: Instant,
    end: Instant,
    computed_size: Option<(i32, i32)>,
}

/// Metadata about a component node collected after a frame is computed.
pub struct NodeMeta {
    /// Unique node identifier.
    pub node_id: String,
    /// Parent node identifier if present.
    pub parent: Option<String>,
    /// Human-readable function name.
    pub fn_name: Option<String>,
    /// Absolute position of the node.
    pub abs_pos: Option<(i32, i32)>,
    /// Computed size of the node.
    pub size: Option<(i32, i32)>,
    /// Whether the layout cache was hit for this node in the frame.
    pub layout_cache_hit: Option<bool>,
}

/// Frame-level metadata dispatched after the component tree has been built and
/// measured.
pub struct FrameMeta {
    /// Frame index.
    pub frame_idx: u64,
    /// Component tree build strategy.
    pub build_mode: BuildMode,
    /// Redraw reasons that woke this frame.
    pub redraw_reasons: Vec<RedrawReason>,
    /// Time spent waiting between this and the previous frame.
    pub inter_frame_wait_ns: Option<u128>,
    /// Number of nodes replayed by partial build in this frame.
    pub partial_replay_nodes: Option<u64>,
    /// Total component nodes before the build pass in this frame.
    pub total_nodes_before_build: Option<u64>,
    /// Render duration for the frame.
    pub render_time_ns: Option<u128>,
    /// Component tree build duration for the frame (wall time).
    pub build_tree_time_ns: Option<u128>,
    /// Draw/compute duration for the frame (wall time).
    pub draw_time_ns: Option<u128>,
    /// Layout record duration for the frame (wall time).
    pub record_time_ns: Option<u128>,
    /// Total duration for the frame.
    pub frame_total_ns: Option<u128>,
    /// Optional layout diagnostics for the frame.
    pub layout_diagnostics: Option<LayoutFrameDiagnostics>,
    /// All nodes observed in the frame.
    pub nodes: Vec<NodeMeta>,
}

/// Redraw wake metadata emitted when the renderer requests a frame.
pub struct WakeMeta {
    /// Frame index associated with the wake request.
    pub frame_idx: u64,
    /// Source category that requested the redraw.
    pub source: WakeSource,
    /// Reasons associated with this wake request.
    pub reasons: Vec<RedrawReason>,
}

/// Runtime lifecycle metadata emitted by the renderer.
pub struct RuntimeMeta {
    /// Runtime event kind.
    pub kind: RuntimeEventKind,
}

/// # Examples
/// A minimal frame event written to the profiler output:
///
/// ```json
/// {"type":"frame","frame":1,"build_mode":"full_initial","redraw_reasons":["startup"],"render_time_ns":1000000,"frame_total_ns":2000000,"components":[{"id":"1","fn_name":"root","abs_pos":{"x":0,"y":0},"size":{"w":100,"h":50},"layout_cache_hit":true,"phases":{"build_ns":5000},"children":[]}]}
/// ```

enum Message {
    Sample(Sample),
    FrameMeta(FrameMeta),
    WakeMeta(WakeMeta),
    RuntimeMeta(RuntimeMeta),
}

struct ProfilerRuntime {
    sender: mpsc::Sender<Message>,
}

struct WorkerState {
    frames: HashMap<u64, Vec<Sample>>,
    writer: BufWriter<File>,
    header_written: bool,
}

static RUNTIME: OnceLock<ProfilerRuntime> = OnceLock::new();
static FRAME_INDEX: AtomicU64 = AtomicU64::new(0);
static OUTPUT_PATH: OnceLock<PathBuf> = OnceLock::new();

fn output_path() -> PathBuf {
    OUTPUT_PATH
        .get()
        .cloned()
        .unwrap_or_else(|| PathBuf::from("tessera-profiler.jsonl"))
}

/// Set profiler output path. Must be called before any profiling begins.
pub fn set_output_path(path: impl AsRef<Path>) {
    let _ = OUTPUT_PATH.set(path.as_ref().to_path_buf());
}

fn profiler_runtime() -> &'static ProfilerRuntime {
    RUNTIME.get_or_init(|| {
        let (sender, receiver) = mpsc::channel::<Message>();
        let _ = thread::Builder::new()
            .name("tessera-profiler".to_string())
            .spawn(move || worker_loop(receiver))
            .expect("failed to spawn profiler worker");
        ProfilerRuntime { sender }
    })
}

fn worker_loop(receiver: mpsc::Receiver<Message>) {
    let mut options = OpenOptions::new();
    options.create(true).write(true).truncate(true);

    let output_path = output_path();
    if let Some(parent) = output_path.parent()
        && !parent.as_os_str().is_empty()
        && let Err(err) = create_dir_all(parent)
    {
        error!(
            "tessera profiler failed to create output directory {}: {err}",
            parent.display()
        );
        return;
    }
    let file = match options.open(&output_path) {
        Ok(file) => file,
        Err(err) => {
            error!(
                "tessera profiler failed to open output file {}: {err}",
                output_path.display()
            );
            return;
        }
    };
    let mut state = WorkerState {
        frames: HashMap::new(),
        writer: BufWriter::new(file),
        header_written: false,
    };

    for msg in receiver {
        match msg {
            Message::Sample(sample) => {
                state
                    .frames
                    .entry(sample.frame_idx)
                    .or_default()
                    .push(sample);
            }
            Message::FrameMeta(frame_meta) => {
                let samples = state
                    .frames
                    .remove(&frame_meta.frame_idx)
                    .unwrap_or_default();
                flush_frame(&mut state, frame_meta, samples);
            }
            Message::WakeMeta(wake_meta) => {
                let event = TraceEvent::Wake(WakeEventRecord {
                    frame: wake_meta.frame_idx,
                    source: wake_meta.source,
                    reasons: wake_meta.reasons,
                });
                write_event(&mut state, &event);
            }
            Message::RuntimeMeta(runtime_meta) => {
                let event = TraceEvent::Runtime(RuntimeEventRecord {
                    kind: runtime_meta.kind,
                });
                write_event(&mut state, &event);
            }
        };
    }
}

fn flush_frame(state: &mut WorkerState, frame_meta: FrameMeta, samples: Vec<Sample>) {
    if let Some(record) = build_frame_record(frame_meta, samples) {
        let event = TraceEvent::Frame(record);
        write_event(state, &event);
    }
}

fn write_header_if_needed(state: &mut WorkerState) {
    if state.header_written {
        return;
    }

    let header = FrameHeader::new();
    if serde_json::to_writer(&mut state.writer, &header).is_ok() {
        let _ = state.writer.write_all(b"\n");
        state.header_written = true;
    }
}

fn write_event(state: &mut WorkerState, event: &TraceEvent) {
    write_header_if_needed(state);
    if serde_json::to_writer(&mut state.writer, event).is_ok() {
        let _ = state.writer.write_all(b"\n");
    }
    let _ = state.writer.flush();
}

/// Reset the active frame index.
pub fn begin_frame(frame_idx: u64) {
    FRAME_INDEX.store(frame_idx, Ordering::Relaxed);
}

/// Samples are sent immediately; no-op for compatibility.
pub fn end_frame() {}

fn current_frame_idx() -> u64 {
    FRAME_INDEX.load(Ordering::Relaxed)
}

fn push_sample(sample: Sample) {
    if let Err(err) = profiler_runtime().sender.send(Message::Sample(sample)) {
        error!("tessera profiler sample send failed: {err}");
    }
}

/// Profiler output frame header.
#[derive(Serialize)]
pub struct FrameHeader {
    /// Profiler output format version.
    version: u32,
    /// File format identifier.
    format: &'static str,
    /// Timestamp when the profiler output was generated.
    generated_at: String,
}

impl FrameHeader {
    fn new() -> Self {
        let generated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| format!("{}", d.as_secs()))
            .unwrap_or_else(|_| String::from("unknown"));
        Self {
            version: 3,
            format: "tessera-profiler",
            generated_at,
        }
    }
}

/// Profiler output event stream record.
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum TraceEvent {
    Frame(FrameEventRecord),
    Wake(WakeEventRecord),
    Runtime(RuntimeEventRecord),
}

/// Profiler output frame event record.
#[derive(Serialize)]
pub struct FrameEventRecord {
    /// Frame index.
    frame: u64,
    /// Build strategy used in this frame.
    build_mode: BuildMode,
    /// Redraw reasons that woke this frame.
    redraw_reasons: Vec<RedrawReason>,
    /// Time spent waiting between this and the previous frame.
    #[serde(skip_serializing_if = "Option::is_none")]
    inter_frame_wait_ns: Option<u128>,
    /// Number of nodes replayed by partial build in this frame.
    #[serde(skip_serializing_if = "Option::is_none")]
    partial_replay_nodes: Option<u64>,
    /// Total component nodes before the build pass in this frame.
    #[serde(skip_serializing_if = "Option::is_none")]
    total_nodes_before_build: Option<u64>,
    /// Render duration for the frame.
    render_time_ns: Option<u128>,
    /// Component tree build duration for the frame (wall time).
    build_tree_time_ns: Option<u128>,
    /// Draw/compute duration for the frame (wall time).
    draw_time_ns: Option<u128>,
    /// Layout record duration for the frame (wall time).
    record_time_ns: Option<u128>,
    /// Total duration for the frame.
    frame_total_ns: Option<u128>,
    /// Optional per-frame layout diagnostics.
    #[serde(skip_serializing_if = "Option::is_none")]
    layout_diagnostics: Option<LayoutDiagnosticsRecord>,
    /// Component tree records.
    components: Vec<ComponentRecord>,
}

#[derive(Serialize)]
struct WakeEventRecord {
    /// Frame index associated with this wake request.
    frame: u64,
    /// Source category that requested the redraw.
    source: WakeSource,
    /// Wake reasons for this request.
    reasons: Vec<RedrawReason>,
}

#[derive(Serialize)]
struct RuntimeEventRecord {
    /// Runtime lifecycle event kind.
    kind: RuntimeEventKind,
}

#[derive(Serialize, Clone, Copy)]
struct LayoutDiagnosticsRecord {
    dirty_nodes_param: u64,
    dirty_nodes_structural: u64,
    dirty_nodes_with_ancestors: u64,
    dirty_expand_ns: u64,
    measure_node_calls: u64,
    cache_hits_direct: u64,
    cache_hits_boundary: u64,
    cache_miss_no_entry: u64,
    cache_miss_constraint: u64,
    cache_miss_dirty_self: u64,
    cache_miss_child_size: u64,
    cache_store_count: u64,
    cache_drop_non_cacheable_count: u64,
}

impl From<LayoutFrameDiagnostics> for LayoutDiagnosticsRecord {
    fn from(value: LayoutFrameDiagnostics) -> Self {
        Self {
            dirty_nodes_param: value.dirty_nodes_param,
            dirty_nodes_structural: value.dirty_nodes_structural,
            dirty_nodes_with_ancestors: value.dirty_nodes_with_ancestors,
            dirty_expand_ns: value.dirty_expand_ns,
            measure_node_calls: value.measure_node_calls,
            cache_hits_direct: value.cache_hits_direct,
            cache_hits_boundary: value.cache_hits_boundary,
            cache_miss_no_entry: value.cache_miss_no_entry,
            cache_miss_constraint: value.cache_miss_constraint,
            cache_miss_dirty_self: value.cache_miss_dirty_self,
            cache_miss_child_size: value.cache_miss_child_size,
            cache_store_count: value.cache_store_count,
            cache_drop_non_cacheable_count: value.cache_drop_non_cacheable_count,
        }
    }
}

/// Component record within a frame.
#[derive(Serialize)]
pub struct ComponentRecord {
    /// Unique(in this frame) node identifier.
    id: String,
    /// The name of the component function.
    fn_name: Option<String>,
    /// Absolute position of the component on window.
    abs_pos: Option<Pos>,
    /// Size of the component.
    size: Option<Size>,
    /// Whether the layout cache was hit for this component.
    #[serde(skip_serializing_if = "Option::is_none")]
    layout_cache_hit: Option<bool>,
    /// How long each phase took.
    phases: PhaseDurations,
    /// Child components' records.
    children: Vec<ComponentRecord>,
}

#[derive(Serialize)]
struct Pos {
    x: i32,
    y: i32,
}

#[derive(Serialize)]
struct Size {
    w: i32,
    h: i32,
}

#[derive(Serialize, Default)]
struct PhaseDurations {
    #[serde(skip_serializing_if = "Option::is_none")]
    build_ns: Option<u128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    measure_ns: Option<u128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    record_ns: Option<u128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_ns: Option<u128>,
}

struct ComponentRecordBuilder {
    id: String,
    parent: Option<String>,
    fn_name: Option<String>,
    abs_pos: Option<Pos>,
    size: Option<Size>,
    layout_cache_hit: Option<bool>,
    phases: PhaseDurations,
    children: Vec<String>,
}

fn mangle_component_fn_name(name: &str) -> String {
    name.strip_prefix("__")
        .and_then(|rest| rest.strip_suffix("_shard_component"))
        .filter(|inner| !inner.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| name.to_owned())
}

fn mangle_component_fn_name_opt(name: Option<&str>) -> Option<String> {
    name.map(mangle_component_fn_name)
}

fn build_frame_record(frame_meta: FrameMeta, samples: Vec<Sample>) -> Option<FrameEventRecord> {
    let mut nodes: HashMap<String, ComponentRecordBuilder> = HashMap::new();

    for node in frame_meta.nodes {
        let id = node.node_id.clone();
        let parent = node.parent.clone();
        let node_fn_name = mangle_component_fn_name_opt(node.fn_name.as_deref());
        let entry = nodes
            .entry(id.clone())
            .or_insert_with(|| ComponentRecordBuilder {
                id: id.clone(),
                parent: parent.clone(),
                fn_name: node_fn_name.clone(),
                abs_pos: node.abs_pos.map(|(x, y)| Pos { x, y }),
                size: node.size.map(|(w, h)| Size { w, h }),
                layout_cache_hit: node.layout_cache_hit,
                phases: PhaseDurations::default(),
                children: Vec::new(),
            });

        if entry.fn_name.is_none() {
            entry.fn_name = node_fn_name.clone();
        }
        if entry.abs_pos.is_none() {
            entry.abs_pos = node.abs_pos.map(|(x, y)| Pos { x, y });
        }
        if entry.size.is_none() {
            entry.size = node.size.map(|(w, h)| Size { w, h });
        }
        if entry.layout_cache_hit.is_none() {
            entry.layout_cache_hit = node.layout_cache_hit;
        }

        if let Some(parent_id) = parent {
            let parent_entry =
                nodes
                    .entry(parent_id.clone())
                    .or_insert_with(|| ComponentRecordBuilder {
                        id: parent_id.clone(),
                        parent: None,
                        fn_name: None,
                        abs_pos: None,
                        size: None,
                        layout_cache_hit: None,
                        phases: PhaseDurations::default(),
                        children: Vec::new(),
                    });
            if !parent_entry.children.contains(&id) {
                parent_entry.children.push(id.clone());
            }
        }
    }

    for sample in samples {
        let duration_ns = sample.end.duration_since(sample.start).as_nanos();
        let Some(node_id) = sample.node_id else {
            continue;
        };
        let node_key = node_id.to_string();
        let sample_fn_name = mangle_component_fn_name_opt(sample.fn_name.as_deref());

        let entry = nodes
            .entry(node_key.clone())
            .or_insert_with(|| ComponentRecordBuilder {
                id: node_key.clone(),
                parent: sample.parent_node_id.map(|p| p.to_string()),
                fn_name: sample_fn_name.clone(),
                abs_pos: sample.abs_pos.map(|(x, y)| Pos { x, y }),
                size: sample.computed_size.map(|(w, h)| Size { w, h }),
                layout_cache_hit: None,
                phases: PhaseDurations::default(),
                children: Vec::new(),
            });

        if entry.fn_name.is_none() {
            entry.fn_name = sample_fn_name.clone();
        }
        if entry.abs_pos.is_none() {
            entry.abs_pos = sample.abs_pos.map(|(x, y)| Pos { x, y });
        }
        if entry.size.is_none() {
            entry.size = sample.computed_size.map(|(w, h)| Size { w, h });
        }

        match sample.phase {
            Phase::Build => {
                entry.phases.build_ns = Some(entry.phases.build_ns.unwrap_or(0) + duration_ns);
            }
            Phase::Measure => {
                entry.phases.measure_ns = Some(entry.phases.measure_ns.unwrap_or(0) + duration_ns);
            }
            Phase::Record => {
                entry.phases.record_ns = Some(entry.phases.record_ns.unwrap_or(0) + duration_ns);
            }
            Phase::Input => {
                entry.phases.input_ns = Some(entry.phases.input_ns.unwrap_or(0) + duration_ns);
            }
            Phase::RenderFrame => {}
        }
    }

    if nodes.is_empty() {
        return Some(FrameEventRecord {
            frame: frame_meta.frame_idx,
            build_mode: frame_meta.build_mode,
            redraw_reasons: frame_meta.redraw_reasons,
            inter_frame_wait_ns: frame_meta.inter_frame_wait_ns,
            partial_replay_nodes: frame_meta.partial_replay_nodes,
            total_nodes_before_build: frame_meta.total_nodes_before_build,
            render_time_ns: frame_meta.render_time_ns,
            build_tree_time_ns: frame_meta.build_tree_time_ns,
            draw_time_ns: frame_meta.draw_time_ns,
            record_time_ns: frame_meta.record_time_ns,
            frame_total_ns: frame_meta.frame_total_ns,
            layout_diagnostics: frame_meta.layout_diagnostics.map(Into::into),
            components: Vec::new(),
        });
    }

    // Determine roots: nodes whose parent is None or parent not present.
    let mut roots = Vec::new();
    for (id, builder) in nodes.iter() {
        let parent_missing = match &builder.parent {
            Some(pid) => !nodes.contains_key(pid),
            None => true,
        };
        if parent_missing {
            roots.push(id.clone());
        }
    }

    fn build_tree(id: &str, map: &mut HashMap<String, ComponentRecordBuilder>) -> ComponentRecord {
        let builder = map.remove(id).unwrap_or_else(|| ComponentRecordBuilder {
            id: id.to_string(),
            parent: None,
            fn_name: None,
            abs_pos: None,
            size: None,
            layout_cache_hit: None,
            phases: PhaseDurations::default(),
            children: Vec::new(),
        });
        let children_ids = builder.children.clone();
        let children = children_ids
            .iter()
            .map(|child_id| build_tree(child_id, map))
            .collect();

        ComponentRecord {
            id: builder.id,
            fn_name: builder.fn_name,
            abs_pos: builder.abs_pos,
            size: builder.size,
            layout_cache_hit: builder.layout_cache_hit,
            phases: builder.phases,
            children,
        }
    }

    let mut map_for_build = nodes;
    let components = roots
        .iter()
        .map(|id| build_tree(id, &mut map_for_build))
        .collect();

    Some(FrameEventRecord {
        frame: frame_meta.frame_idx,
        build_mode: frame_meta.build_mode,
        redraw_reasons: frame_meta.redraw_reasons,
        inter_frame_wait_ns: frame_meta.inter_frame_wait_ns,
        partial_replay_nodes: frame_meta.partial_replay_nodes,
        total_nodes_before_build: frame_meta.total_nodes_before_build,
        render_time_ns: frame_meta.render_time_ns,
        build_tree_time_ns: frame_meta.build_tree_time_ns,
        draw_time_ns: frame_meta.draw_time_ns,
        record_time_ns: frame_meta.record_time_ns,
        frame_total_ns: frame_meta.frame_total_ns,
        layout_diagnostics: frame_meta.layout_diagnostics.map(Into::into),
        components,
    })
}

/// RAII guard that records a single scoped timing sample.
pub struct ScopeGuard {
    sample: Option<Sample>,
}

/// Submit frame-level metadata after the component tree has finished computing.
pub fn submit_frame_meta(frame_meta: FrameMeta) {
    if let Err(err) = profiler_runtime()
        .sender
        .send(Message::FrameMeta(frame_meta))
    {
        eprintln!("tessera profiler frame meta send failed: {err}");
    }
}

/// Submit redraw wake metadata.
pub fn submit_wake_meta(wake_meta: WakeMeta) {
    if let Err(err) = profiler_runtime().sender.send(Message::WakeMeta(wake_meta)) {
        eprintln!("tessera profiler wake meta send failed: {err}");
    }
}

/// Submit runtime lifecycle metadata.
pub fn submit_runtime_meta(runtime_meta: RuntimeMeta) {
    if let Err(err) = profiler_runtime()
        .sender
        .send(Message::RuntimeMeta(runtime_meta))
    {
        eprintln!("tessera profiler runtime meta send failed: {err}");
    }
}

/// Construct a build-phase scope guard using the provided function name.
pub fn make_build_scope_guard(
    node_id: NodeId,
    parent_node_id: Option<NodeId>,
    fn_name: &str,
) -> Option<ScopeGuard> {
    Some(ScopeGuard::new(
        Phase::Build,
        Some(node_id),
        parent_node_id,
        Some(fn_name),
    ))
}

impl ScopeGuard {
    /// Create a new profiling scope for the given phase and component metadata.
    pub fn new(
        phase: Phase,
        node_id: Option<NodeId>,
        parent_node_id: Option<NodeId>,
        fn_name: Option<&str>,
    ) -> Self {
        let frame_idx = current_frame_idx();
        let fn_name_owned = fn_name.map(ToOwned::to_owned);
        let sample = Sample {
            phase,
            frame_idx,
            node_id,
            parent_node_id,
            fn_name: fn_name_owned,
            abs_pos: None,
            start: Instant::now(),
            end: Instant::now(),
            computed_size: None,
        };
        Self {
            sample: Some(sample),
        }
    }

    /// Attach the measured size for layout samples.
    pub fn set_computed_size(&mut self, width: i32, height: i32) {
        if let Some(sample) = &mut self.sample {
            sample.computed_size = Some((width, height));
        }
    }

    /// Attach positional info for layout/debug.
    pub fn set_positions(&mut self, abs_pos: Option<(i32, i32)>) {
        if let Some(sample) = &mut self.sample {
            sample.abs_pos = abs_pos;
        }
    }
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        if let Some(mut sample) = self.sample.take() {
            sample.end = Instant::now();
            push_sample(sample);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{mangle_component_fn_name, mangle_component_fn_name_opt};

    #[test]
    fn mangle_shard_component_name() {
        assert_eq!(
            mangle_component_fn_name("__home_shard_component"),
            "home".to_string()
        );
    }

    #[test]
    fn keep_regular_component_name() {
        assert_eq!(mangle_component_fn_name("button"), "button".to_string());
        assert_eq!(
            mangle_component_fn_name_opt(Some("__settings_shard_component")),
            Some("settings".to_string())
        );
        assert_eq!(
            mangle_component_fn_name_opt(Some("text_input")),
            Some("text_input".to_string())
        );
    }
}
