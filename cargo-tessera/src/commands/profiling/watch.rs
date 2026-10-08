//! Interactive, top-like profiler view for live Tessera frames.
//!
//! ## Usage
//!
//! Watch per-component build/measure costs while scrolling or interacting with
//! a running app.
use std::{
    collections::{HashMap, VecDeque},
    io::{self, Read, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender, channel},
    },
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use interprocess::local_socket::{
    Listener, ListenerNonblockingMode, ListenerOptions, Name, traits::Listener as _,
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    crossterm::{
        event::{
            self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
            KeyModifiers,
        },
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Sparkline, Table, TableState, Wrap},
};

use super::{FrameRecord, TraceEvent, parse_json_line};

/// Options for [`watch`].
pub struct WatchOptions {
    /// Refresh interval for reading new records and redrawing.
    pub refresh: Duration,
    /// Number of component rows to show.
    pub top: usize,
    /// Number of recent frames kept for the sparkline.
    pub window: usize,
    /// Minimum sample count for a component to be listed.
    pub min_count: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortKey {
    Build,
    Measure,
    Record,
    Input,
    Total,
    Count,
}

impl SortKey {
    fn next(self) -> Self {
        match self {
            SortKey::Build => SortKey::Measure,
            SortKey::Measure => SortKey::Record,
            SortKey::Record => SortKey::Input,
            SortKey::Input => SortKey::Total,
            SortKey::Total => SortKey::Count,
            SortKey::Count => SortKey::Build,
        }
    }

    fn label(self) -> &'static str {
        match self {
            SortKey::Build => "build",
            SortKey::Measure => "measure",
            SortKey::Record => "record",
            SortKey::Input => "input",
            SortKey::Total => "total",
            SortKey::Count => "count",
        }
    }

    fn value(self, agg: &Agg) -> f64 {
        match self {
            SortKey::Build => agg.build as f64,
            SortKey::Measure => agg.measure as f64,
            SortKey::Record => agg.record as f64,
            SortKey::Input => agg.input as f64,
            SortKey::Total => agg.total() as f64,
            SortKey::Count => agg.count as f64,
        }
    }
}

/// Per-component, per-phase time accumulated over the rolling window.
#[derive(Default, Clone, Copy)]
struct Agg {
    count: u64,
    build: u64,
    measure: u64,
    record: u64,
    input: u64,
}

impl Agg {
    fn total(&self) -> u64 {
        self.build + self.measure + self.record + self.input
    }

    fn add(&mut self, other: &Agg) {
        self.count += other.count;
        self.build += other.build;
        self.measure += other.measure;
        self.record += other.record;
        self.input += other.input;
    }
}

#[derive(Clone, Copy)]
struct FrameSample {
    index: u64,
    total_ns: u64,
    build_ns: u64,
    draw_ns: u64,
    record_ns: u64,
    measure_calls: u64,
    cache_hits: u64,
    cache_misses: u64,
    dirty_effective: u64,
}

struct WatchState {
    events: Receiver<StreamEvent>,
    connected: bool,
    buffer: Vec<u8>,
    logs: VecDeque<String>,
    logs_rx: Receiver<String>,
    stop: Arc<AtomicBool>,
    recording: bool,
    record_started: Option<Instant>,
    interval_elapsed: Duration,
    sort: SortKey,
    table: TableState,
    totals: HashMap<String, Agg>,
    frames: u64,
    nodes: u64,
    samples: VecDeque<FrameSample>,
    last: Option<FrameSample>,
    total_frame_ns: u128,
    total_build_ns: u128,
    last_error: Option<String>,
}

impl WatchState {
    fn new(
        events: Receiver<StreamEvent>,
        logs_rx: Receiver<String>,
        stop: Arc<AtomicBool>,
        options: &WatchOptions,
    ) -> Self {
        Self {
            events,
            connected: false,
            buffer: Vec::new(),
            logs: VecDeque::new(),
            logs_rx,
            stop,
            recording: false,
            record_started: None,
            interval_elapsed: Duration::ZERO,
            sort: SortKey::Build,
            table: TableState::default(),
            totals: HashMap::new(),
            frames: 0,
            nodes: 0,
            samples: VecDeque::with_capacity(options.window),
            last: None,
            total_frame_ns: 0,
            total_build_ns: 0,
            last_error: None,
        }
    }

    fn connected(&self) -> bool {
        self.connected
    }

    fn drain_logs(&mut self) {
        while let Ok(line) = self.logs_rx.try_recv() {
            if self.logs.len() >= 200 {
                self.logs.pop_front();
            }
            self.logs.push_back(line);
        }
    }

    /// Clears the current recording interval (totals, sparkline and counters).
    fn clear_interval(&mut self) {
        self.totals.clear();
        self.frames = 0;
        self.nodes = 0;
        self.samples.clear();
        self.last = None;
        self.total_frame_ns = 0;
        self.total_build_ns = 0;
        self.interval_elapsed = Duration::ZERO;
        self.record_started = None;
        self.table.select(Some(0));
    }

    fn ingest(&mut self, event: TraceEvent, window: usize) {
        match event {
            TraceEvent::Frame(frame) => self.ingest_frame(*frame, window),
            TraceEvent::Wake(_) | TraceEvent::Runtime(_) => {}
        }
    }

    fn ingest_frame(&mut self, frame: FrameRecord, window: usize) {
        // Only recorded frames contribute to the displayed interval.
        if !self.recording {
            return;
        }

        let mut frame_agg: HashMap<String, Agg> = HashMap::new();
        for component in &frame.components {
            accumulate_frame_component(component, &mut frame_agg);
        }

        let diagnostics = frame.layout_diagnostics.as_ref();
        let sample = FrameSample {
            index: frame.frame,
            total_ns: frame.frame_total_ns.unwrap_or(0),
            build_ns: frame.build_tree_time_ns.unwrap_or(0),
            draw_ns: frame.draw_time_ns.unwrap_or(0),
            record_ns: frame.record_time_ns.unwrap_or(0),
            measure_calls: diagnostics.map_or(0, |d| d.measure_node_calls),
            cache_hits: diagnostics.map_or(0, |d| d.cache_hits_direct + d.cache_hits_boundary),
            cache_misses: diagnostics.map_or(0, |d| {
                d.cache_miss_no_entry
                    + d.cache_miss_constraint
                    + d.cache_miss_dirty_self
                    + d.cache_miss_child_size
            }),
            dirty_effective: diagnostics.map_or(0, |d| d.dirty_nodes_with_ancestors),
        };

        self.frames += 1;
        self.nodes += frame.components.len() as u64;
        self.total_frame_ns += u128::from(sample.total_ns);
        self.total_build_ns += u128::from(sample.build_ns);
        for (name, agg) in &frame_agg {
            self.totals.entry(name.clone()).or_default().add(agg);
        }

        if window > 0 {
            while self.samples.len() >= window {
                self.samples.pop_front();
            }
            self.samples.push_back(sample);
        }
        self.last = Some(sample);
    }
}

/// Adds one component (and its descendants) into this frame's per-name totals.
fn accumulate_frame_component(component: &super::ComponentRecord, agg: &mut HashMap<String, Agg>) {
    let name = component
        .fn_name
        .clone()
        .unwrap_or_else(|| "<unknown>".to_string());
    let entry = agg.entry(name).or_default();
    entry.count += 1;
    if let Some(value) = component.phases.build_ns {
        entry.build += value;
    }
    if let Some(value) = component.phases.measure_ns {
        entry.measure += value;
    }
    if let Some(value) = component.phases.record_ns {
        entry.record += value;
    }
    if let Some(value) = component.phases.input_ns {
        entry.input += value;
    }
    for child in &component.children {
        accumulate_frame_component(child, agg);
    }
}

/// Runs the live view on a local IPC listener, consuming profiler records from
/// the app. Dev-server build/startup logs stream to the console until the app
/// connects, then the interactive view takes over the terminal.
pub fn run_view(
    listener: Listener,
    options: WatchOptions,
    logs_rx: Receiver<String>,
    stop: Arc<AtomicBool>,
) -> Result<()> {
    let events = spawn_reader(listener, Arc::clone(&stop));
    let mut state = WatchState::new(events, logs_rx, stop, &options);

    if !wait_for_connection(&mut state, &options)? {
        return Ok(());
    }

    let mut terminal = setup_terminal().context("failed to initialize terminal")?;
    // The guard restores the terminal on early return or panic unwinding.
    let _guard = TerminalGuard;
    run_loop(&mut terminal, &mut state, &options)
}

/// Streams dev-server logs to the console until the app connects over IPC.
///
/// Returns `false` when the run was stopped before the app ever connected.
fn wait_for_connection(state: &mut WatchState, options: &WatchOptions) -> Result<bool> {
    while !state.connected {
        if state.stop.load(Ordering::Relaxed) {
            return Ok(false);
        }
        while let Ok(line) = state.logs_rx.try_recv() {
            println!("{line}");
        }
        poll_stream(state, options.window)?;
        if !state.connected {
            thread::sleep(Duration::from_millis(50));
        }
    }
    Ok(true)
}

/// Builds an IPC listener name appropriate for the current platform.
///
/// Windows uses a namespaced pipe name; other platforms use a socket path.
pub fn ipc_listener_name(raw: &str) -> Result<Name<'_>> {
    use interprocess::local_socket::prelude::*;
    #[cfg(windows)]
    {
        use interprocess::local_socket::GenericNamespaced;
        raw.to_ns_name::<GenericNamespaced>()
            .with_context(|| format!("invalid IPC pipe name `{raw}`"))
    }
    #[cfg(unix)]
    {
        use interprocess::local_socket::GenericFilePath;
        raw.to_fs_name::<GenericFilePath>()
            .with_context(|| format!("invalid IPC socket path `{raw}`"))
    }
}

/// Creates a nonblocking local IPC listener for the live view.
pub fn bind_ipc_listener(raw: &str) -> Result<Listener> {
    let name = ipc_listener_name(raw)?;
    let listener = ListenerOptions::new()
        .name(name)
        .nonblocking(ListenerNonblockingMode::Accept)
        .create_sync()
        .with_context(|| format!("failed to create IPC listener `{raw}`"))?;
    Ok(listener)
}

/// Restores the terminal to a usable state when dropped.
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = std::io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);
        // Show the cursor again in case it was hidden.
        let _ = stdout.write_all(b"\x1b[?25h");
        let _ = stdout.flush();
    }
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    state: &mut WatchState,
    options: &WatchOptions,
) -> Result<()> {
    loop {
        state.drain_logs();
        if state.stop.load(Ordering::Relaxed) {
            return Ok(());
        }
        terminal.draw(|frame| draw(frame, state, options))?;

        if event::poll(options.refresh)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        state.stop.store(true, Ordering::Relaxed);
                        return Ok(());
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        state.stop.store(true, Ordering::Relaxed);
                        return Ok(());
                    }
                    KeyCode::Char('s') => {
                        state.sort = state.sort.next();
                        state.table.select(Some(0));
                    }
                    KeyCode::Char('r') => state.clear_interval(),
                    KeyCode::Char(' ') => {
                        if state.recording {
                            if let Some(started) = state.record_started {
                                state.interval_elapsed = started.elapsed();
                            }
                            state.recording = false;
                        } else {
                            state.clear_interval();
                            state.recording = true;
                            state.record_started = Some(Instant::now());
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => scroll(state, options.top, 1),
                    KeyCode::Up | KeyCode::Char('k') => scroll(state, options.top, -1),
                    KeyCode::PageDown => scroll(state, options.top, 10),
                    KeyCode::PageUp => scroll(state, options.top, -10),
                    KeyCode::Home => state.table.select(Some(0)),
                    KeyCode::End => state.table.select(Some(usize::MAX)),
                    _ => {}
                },
                Event::Resize(_, _) => {}
                _ => {}
            }
        }

        poll_stream(state, options.window)?;
    }
}

fn scroll(state: &mut WatchState, top: usize, delta: i32) {
    let len = state.totals.len().min(top.max(1));
    if len == 0 {
        return;
    }
    let current = state.table.selected().unwrap_or(0) as i32;
    let next = (current + delta).clamp(0, len as i32 - 1);
    state.table.select(Some(next as usize));
}

/// Accepts a pending app connection and drains any buffered records.
/// Events pushed by the background IPC reader.
enum StreamEvent {
    Connected,
    Bytes(Vec<u8>),
    Disconnected,
}

/// Reads the app's profiler stream on a background thread so the UI never
/// blocks on accept/read, forwarding bytes to the view over a channel.
fn spawn_reader(listener: Listener, stop: Arc<AtomicBool>) -> Receiver<StreamEvent> {
    let (tx, rx) = channel();
    let _ = thread::Builder::new()
        .name("tessera-profiler-ipc".to_string())
        .spawn(move || {
            read_loop(listener, stop, tx);
        });
    rx
}

fn read_loop(listener: Listener, stop: Arc<AtomicBool>, tx: Sender<StreamEvent>) {
    loop {
        if stop.load(Ordering::Relaxed) {
            return;
        }
        match listener.accept() {
            Ok(mut stream) => {
                if tx.send(StreamEvent::Connected).is_err() {
                    return;
                }
                let mut chunk = [0u8; 16 * 1024];
                loop {
                    if stop.load(Ordering::Relaxed) {
                        return;
                    }
                    match stream.read(&mut chunk) {
                        Ok(0) => break,
                        Ok(read) => {
                            if tx.send(StreamEvent::Bytes(chunk[..read].to_vec())).is_err() {
                                return;
                            }
                        }
                        Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
                        Err(_) => break,
                    }
                }
                let _ = tx.send(StreamEvent::Disconnected);
            }
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(_) => {
                thread::sleep(Duration::from_millis(50));
            }
        }
    }
}

fn poll_stream(state: &mut WatchState, window: usize) -> Result<()> {
    while let Ok(event) = state.events.try_recv() {
        match event {
            StreamEvent::Connected => {
                state.connected = true;
                state.buffer.clear();
                state.last_error = None;
            }
            StreamEvent::Bytes(bytes) => {
                state.connected = true;
                state.buffer.extend_from_slice(&bytes);
            }
            StreamEvent::Disconnected => {
                state.connected = false;
                state.buffer.clear();
                state.last_error =
                    Some("app disconnected; waiting for it to reconnect".to_string());
            }
        }
    }

    let mut lines: Vec<String> = Vec::new();
    let mut consumed = 0usize;
    while let Some(relative) = state.buffer[consumed..]
        .iter()
        .position(|&byte| byte == b'\n')
    {
        let end = consumed + relative;
        if let Ok(text) = std::str::from_utf8(&state.buffer[consumed..end]) {
            lines.push(text.to_string());
        }
        consumed = end + 1;
    }
    if consumed > 0 {
        state.buffer.drain(..consumed);
    }
    // Guard against an unbounded partial line from a corrupt sender.
    if state.buffer.len() > (1 << 20) {
        state.buffer.clear();
    }

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(event) = parse_json_line::<TraceEvent>(trimmed) {
            state.ingest(event, window);
        }
    }
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn draw(frame: &mut Frame, state: &mut WatchState, options: &WatchOptions) {
    let log_height = if state.logs.is_empty() { 0 } else { 6 };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(6),
            Constraint::Length(log_height),
            Constraint::Length(1),
        ])
        .split(frame.area());

    draw_title(frame, chunks[0], state);
    draw_frame_summary(frame, chunks[1], state);
    draw_sparkline(frame, chunks[2], state);
    draw_components(frame, chunks[3], state, options);
    if log_height > 0 {
        draw_logs(frame, chunks[4], state);
    }
    draw_footer(frame, chunks[5], state);
}

fn draw_logs(frame: &mut Frame, area: Rect, state: &WatchState) {
    let lines: Vec<Line> = state
        .logs
        .iter()
        .rev()
        .take(area.height.saturating_sub(2) as usize)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|line| Line::from(line.clone()))
        .collect();
    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("dev log")
            .style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(paragraph, area);
}

fn draw_title(frame: &mut Frame, area: Rect, state: &WatchState) {
    let (mode, mode_style) = if !state.connected() {
        ("waiting", Style::default().fg(Color::Yellow))
    } else if state.recording {
        ("recording", Style::default().fg(Color::Green))
    } else {
        ("stopped", Style::default().fg(Color::Yellow))
    };
    let elapsed = if state.recording {
        state
            .record_started
            .map(|started| started.elapsed())
            .unwrap_or(state.interval_elapsed)
    } else {
        state.interval_elapsed
    };
    let line = Line::from(vec![
        Span::styled(
            "Tessera profiler ",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw("local ipc"),
        Span::raw("  "),
        Span::styled(format!("[{mode}]"), mode_style),
        Span::raw(format!(
            "  interval {} frames   {:.1}s",
            state.frames,
            elapsed.as_secs_f64()
        )),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn draw_frame_summary(frame: &mut Frame, area: Rect, state: &WatchState) {
    let Some(last) = &state.last else {
        let text = if !state.connected() {
            state
                .last_error
                .clone()
                .unwrap_or_else(|| "waiting for the app to connect over local IPC".to_string())
        } else if state.recording {
            "recording - waiting for the first frame…".to_string()
        } else {
            "press [space] to start recording an interval".to_string()
        };
        frame.render_widget(
            Paragraph::new(text).style(Style::default().fg(Color::Yellow)),
            area,
        );
        return;
    };

    let avg_frame = if state.frames > 0 {
        state.total_frame_ns as f64 / state.frames as f64
    } else {
        0.0
    };
    let hit_total = last.cache_hits + last.cache_misses;
    let hit_rate = if hit_total > 0 {
        last.cache_hits as f64 / hit_total as f64
    } else {
        0.0
    };
    let lines = vec![
        Line::from(format!(
            "frame #{}  total {}  build {}  draw {}  record {}  (avg total {})",
            last.index,
            format_ms(last.total_ns),
            format_ms(last.build_ns),
            format_ms(last.draw_ns),
            format_ms(last.record_ns),
            format_ms(avg_frame as u64),
        )),
        Line::from(format!(
            "layout: dirty {}  measure calls {}  cache hit {:.0}%  misses {}",
            last.dirty_effective,
            last.measure_calls,
            hit_rate * 100.0,
            last.cache_misses,
        )),
    ];
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), area);
}

fn draw_sparkline(frame: &mut Frame, area: Rect, state: &WatchState) {
    let data: Vec<u64> = state.samples.iter().map(|s| s.build_ns / 1_000).collect();
    let max = data.iter().copied().max().unwrap_or(1).max(1);
    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" build time (µs)"),
        )
        .data(&data)
        .max(max)
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(sparkline, area);
}

fn draw_components(frame: &mut Frame, area: Rect, state: &mut WatchState, options: &WatchOptions) {
    let frames = state.frames.max(1) as f64;
    let mut rows: Vec<(&String, &Agg)> = state
        .totals
        .iter()
        .filter(|(_, agg)| agg.count >= options.min_count && agg.count > 0)
        .collect();
    rows.sort_by(|a, b| {
        state
            .sort
            .value(b.1)
            .partial_cmp(&state.sort.value(a.1))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let phase_total: f64 = rows.iter().map(|(_, agg)| state.sort.value(agg)).sum();
    let visible = rows.len().min(options.top.max(1));
    let selected = state
        .table
        .selected()
        .unwrap_or(0)
        .min(visible.saturating_sub(1));
    state.table.select(Some(selected));

    let header = Row::new(vec![
        Cell::from("Component"),
        Cell::from("/frame"),
        Cell::from("Build/f"),
        Cell::from("Measure/f"),
        Cell::from("Record/f"),
        Cell::from("Input/f"),
        Cell::from("%"),
    ])
    .style(
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Gray),
    );

    let per_frame = |value: u64| format_ms((value as f64 / frames) as u64);
    let table_rows = rows.iter().take(options.top.max(1)).map(|(name, agg)| {
        let share = if phase_total > 0.0 {
            state.sort.value(agg) / phase_total
        } else {
            0.0
        };
        Row::new(vec![
            Cell::from(name.as_str()),
            Cell::from(format!("{:.1}", agg.count as f64 / frames)),
            Cell::from(per_frame(agg.build)),
            Cell::from(per_frame(agg.measure)),
            Cell::from(per_frame(agg.record)),
            Cell::from(per_frame(agg.input)),
            Cell::from(format!("{:.0}%", share * 100.0)),
        ])
    });

    let widths = [
        Constraint::Min(16),
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Length(11),
        Constraint::Length(10),
        Constraint::Length(9),
        Constraint::Length(6),
    ];
    let title = format!(
        "components · interval {} frames · sorted by {} ({}/frame)",
        state.frames,
        state.sort.label(),
        format_ms((phase_total / frames) as u64),
    );
    let table = Table::new(table_rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title))
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(table, area, &mut state.table);
}

fn draw_footer(frame: &mut Frame, area: Rect, state: &WatchState) {
    let text = format!(
        "[s]ort: {}   [↑/↓] scroll   [space] {} recording   [r]eset   [q]uit",
        state.sort.label(),
        if state.recording { "stop" } else { "start" },
    );
    frame.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Left)
            .style(Style::default().fg(Color::DarkGray)),
        area,
    );
}

fn format_ms(ns: u64) -> String {
    let ms = ns as f64 / 1_000_000.0;
    if ms >= 1.0 {
        format!("{ms:.2} ms")
    } else {
        format!("{:.0} µs", ns as f64 / 1_000.0)
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use interprocess::local_socket::prelude::*;

    use super::*;

    fn options() -> WatchOptions {
        WatchOptions {
            refresh: Duration::from_millis(250),
            top: 15,
            window: 120,
            min_count: 1,
        }
    }

    fn frame_line(index: u64, root_build: u64, child_build: u64) -> String {
        serde_json::json!({
            "type": "frame",
            "frame": index,
            "build_mode": "skip_no_invalidation",
            "redraw_reasons": ["runtime_invalidation"],
            "frame_total_ns": 5_000_000u64,
            "build_tree_time_ns": 1_000_000u64,
            "components": [{
                "id": "1",
                "fn_name": "root",
                "phases": { "build_ns": root_build },
                "children": [{
                    "id": "2",
                    "fn_name": "child",
                    "phases": { "build_ns": child_build },
                    "children": []
                }]
            }]
        })
        .to_string()
    }

    fn unique_name() -> String {
        use std::sync::atomic::{AtomicU64 as Counter, Ordering as CounterOrdering};
        static COUNTER: Counter = Counter::new(0);
        format!(
            "tessera-profiler-test-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, CounterOrdering::Relaxed)
        )
    }

    fn listening_state() -> (WatchState, LocalSocketStream) {
        let raw = unique_name();
        let listener = bind_ipc_listener(&raw).unwrap();
        let client = LocalSocketStream::connect(ipc_listener_name(&raw).unwrap()).unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let events = spawn_reader(listener, Arc::clone(&stop));
        let (_logs_tx, logs_rx) = std::sync::mpsc::channel();
        let mut state = WatchState::new(events, logs_rx, stop, &options());
        state.recording = true;
        (state, client)
    }

    fn pump(state: &mut WatchState, options: &WatchOptions, iterations: usize) {
        for _ in 0..iterations {
            poll_stream(state, options.window).unwrap();
            thread::sleep(Duration::from_millis(2));
        }
    }

    #[test]
    fn aggregates_streamed_frames_over_ipc() {
        let (mut state, mut client) = listening_state();
        let options = options();

        client
            .write_all(format!("{}\n", frame_line(1, 1_000_000, 400_000)).as_bytes())
            .unwrap();
        client.flush().unwrap();
        pump(&mut state, &options, 100);

        assert_eq!(state.frames, 1);
        let root = state.totals.get("root").expect("root stats");
        assert_eq!(root.count, 1);
        // Per-component phase times are reported as-is and summed per frame.
        assert_eq!(root.build, 1_000_000);
        assert_eq!(
            state.totals.get("child").expect("child stats").build,
            400_000
        );

        client
            .write_all(format!("{}\n", frame_line(2, 2_000_000, 500_000)).as_bytes())
            .unwrap();
        client.flush().unwrap();
        pump(&mut state, &options, 100);

        assert_eq!(state.frames, 2);
        assert_eq!(
            state.totals.get("root").expect("root stats").build,
            3_000_000
        );
        assert_eq!(state.samples.len(), 2);
    }

    #[test]
    fn waits_for_complete_lines() {
        let (mut state, mut client) = listening_state();
        let options = options();

        client.write_all(frame_line(1, 100, 50).as_bytes()).unwrap();
        client.flush().unwrap();
        pump(&mut state, &options, 30);
        assert_eq!(state.frames, 0, "a chunk without a newline is incomplete");

        client.write_all(b"\n").unwrap();
        client.flush().unwrap();
        pump(&mut state, &options, 100);
        assert_eq!(state.frames, 1);
    }

    #[test]
    fn renders_with_and_without_frames() {
        let options = options();
        let listener = bind_ipc_listener(&unique_name()).unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let events = spawn_reader(listener, Arc::clone(&stop));
        let (_logs_tx, logs_rx) = std::sync::mpsc::channel();
        let mut state = WatchState::new(events, logs_rx, stop, &options);
        state.recording = true;

        let mut terminal = Terminal::new(ratatui::backend::TestBackend::new(100, 30)).unwrap();
        // No client connected yet: the view shows the listening hint.
        terminal
            .draw(|frame| draw(frame, &mut state, &options))
            .unwrap();

        let event: TraceEvent = parse_json_line(&frame_line(1, 1_000_000, 400_000)).unwrap();
        state.ingest(event, options.window);
        state.sort = state.sort.next();
        terminal
            .draw(|frame| draw(frame, &mut state, &options))
            .unwrap();
        assert_eq!(state.frames, 1);
    }
}
