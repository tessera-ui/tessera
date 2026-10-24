//! Headless layout testing helpers for Tessera UI.
//!
//! ## Usage
//!
//! Assert positions and sizes of tagged nodes without creating a real renderer.

use std::{collections::BTreeSet, time::Duration};

use rustc_hash::FxHashMap as HashMap;

use crate::{
    Px, PxPosition, PxRect, PxSize,
    build_tree::build_component_tree,
    component_tree::{ComputeMode, ComputeParams, clear_layout_snapshots},
    context::{reset_component_context_tracking, reset_context_read_dependencies},
    cursor::PointerChange,
    focus::flush_pending_focus_callbacks,
    runtime::{
        TesseraRuntime, begin_frame_clock, clear_persistent_focus_handles, clear_redraw_waker,
        reset_build_invalidations, reset_component_replay_tracking, reset_focus_read_dependencies,
        reset_frame_clock, reset_layout_dirty_tracking, reset_render_slot_read_dependencies,
        reset_slots, reset_state_read_dependencies, take_layout_dirty_nodes,
        tick_frame_nanos_receivers,
    },
    time::Instant,
};

pub use crate::component_tree::LayoutFrameDiagnostics;

thread_local! {
    /// Layout diagnostics produced by the most recent `pump_layout_test_session`
    /// call, used by cache-reuse regression tests.
    static LAST_LAYOUT_FRAME_DIAGNOSTICS: std::cell::Cell<Option<LayoutFrameDiagnostics>> =
        const { std::cell::Cell::new(None) };
}

/// Returns the layout diagnostics captured by the most recent pumped frame.
#[cfg(test)]
pub(crate) fn last_layout_frame_diagnostics() -> Option<LayoutFrameDiagnostics> {
    LAST_LAYOUT_FRAME_DIAGNOSTICS.with(|cell| cell.get())
}

/// Create a headless layout test harness for the provided component content.
pub fn layout_test<F>(content: F) -> LayoutTestHarness<F>
where
    F: Fn(),
{
    LayoutTestHarness {
        content,
        viewport: (800, 600),
    }
}

/// Runs a component through build and layout without a real renderer.
pub struct LayoutTestHarness<F>
where
    F: Fn(),
{
    content: F,
    viewport: (u32, u32),
}

impl<F> LayoutTestHarness<F>
where
    F: Fn(),
{
    /// Set the test viewport size in physical pixels.
    pub fn viewport_px(mut self, width: u32, height: u32) -> Self {
        self.viewport = (width, height);
        self
    }

    /// Execute build and layout, then capture a snapshot for assertions.
    pub fn run(self) -> LayoutSnapshot {
        let mut session = start_layout_test_session(self);
        pump_layout_test_session(&mut session, 0)
    }
}

/// A headless Tessera layout test session that can be advanced frame by frame.
///
/// Build the harness with [`layout_test`], open a session with
/// [`start_layout_test_session`], then step frames with
/// [`pump_layout_test_session`] to assert snapshots or observe side effects
/// between frames.
pub struct LayoutTestSession<F>
where
    F: Fn(),
{
    content: F,
    viewport: (u32, u32),
    frame_origin: Instant,
    current_frame_nanos: u64,
    diagnostics: Option<LayoutFrameDiagnostics>,
}

impl<F: Fn()> LayoutTestSession<F> {
    /// Returns layout work recorded during the most recently pumped frame.
    pub fn diagnostics(&self) -> Option<LayoutFrameDiagnostics> {
        self.diagnostics
    }
}

/// Starts a headless layout test session for `harness`.
pub fn start_layout_test_session<F>(harness: LayoutTestHarness<F>) -> LayoutTestSession<F>
where
    F: Fn(),
{
    reset_runtime_for_layout_test(harness.viewport);
    LayoutTestSession {
        content: harness.content,
        viewport: harness.viewport,
        frame_origin: Instant::now(),
        current_frame_nanos: 0,
        diagnostics: None,
    }
}

/// Builds and lays out one frame at `frame_nanos`, returning a snapshot.
pub fn pump_layout_test_session<F>(
    session: &mut LayoutTestSession<F>,
    frame_nanos: u64,
) -> LayoutSnapshot
where
    F: Fn(),
{
    pump_layout_test_session_with_input(session, frame_nanos, Vec::new(), None)
}

/// Builds and lays out one frame while delivering pointer input to the tree.
///
/// `pointer_changes` and `cursor_position` are forwarded to the component tree
/// before layout, mirroring one renderer frame with injected input.
pub fn pump_layout_test_session_with_input<F>(
    session: &mut LayoutTestSession<F>,
    frame_nanos: u64,
    pointer_changes: Vec<PointerChange>,
    cursor_position: Option<PxPosition>,
) -> LayoutSnapshot
where
    F: Fn(),
{
    session.current_frame_nanos = frame_nanos;
    let frame_time = session.frame_origin + Duration::from_nanos(frame_nanos);
    begin_frame_clock(frame_time);
    // Match renderer frame order so frame callbacks update state before build.
    tick_frame_nanos_receivers();
    let _ = build_component_tree(&session.content);
    let layout_dirty_nodes = take_layout_dirty_nodes();
    let screen_size = PxSize::new(
        Px::new(session.viewport.0 as i32),
        Px::new(session.viewport.1 as i32),
    );

    let diagnostics = TesseraRuntime::with_mut(|runtime| {
        let (_graph, _requests, diagnostics, _record_cost, _move, _reveal, _keyboard) =
            runtime.component_tree.compute(
                ComputeParams {
                    screen_size,
                    cursor_position,
                    pointer_changes,
                    keyboard_events: Vec::new(),
                    ime_events: Vec::new(),
                    retry_focus_move: None,
                    retry_focus_reveal: false,
                    modifiers: winit::keyboard::ModifiersState::default(),
                    layout_dirty_nodes: &layout_dirty_nodes,
                },
                ComputeMode::LayoutOnly,
            );
        diagnostics
    });
    LAST_LAYOUT_FRAME_DIAGNOSTICS.with(|cell| cell.set(Some(diagnostics)));
    session.diagnostics = Some(diagnostics);
    flush_pending_focus_callbacks();
    crate::component_tree::flush_pending_globally_positioned_callbacks();

    LayoutSnapshot::capture()
}

/// Advances the session by `delta_nanos` and lays out the resulting frame.
pub fn advance_layout_test_session_by_nanos<F>(
    session: &mut LayoutTestSession<F>,
    delta_nanos: u64,
) -> LayoutSnapshot
where
    F: Fn(),
{
    pump_layout_test_session(
        session,
        session.current_frame_nanos.saturating_add(delta_nanos),
    )
}

/// Returns the `frame_nanos` of the session's most recent frame.
pub fn current_layout_test_frame_nanos<F>(session: &LayoutTestSession<F>) -> u64
where
    F: Fn(),
{
    session.current_frame_nanos
}

/// Captured layout information for a single headless test run.
pub struct LayoutSnapshot {
    root: LayoutNodeSnapshot,
    nodes_by_selector: HashMap<String, LayoutNodeSnapshot>,
    nodes_by_fn_name: HashMap<String, Vec<LayoutNodeSnapshot>>,
}

impl LayoutSnapshot {
    fn capture() -> Self {
        TesseraRuntime::with(|runtime| {
            let tree = runtime.component_tree.tree();
            let metadatas = runtime.component_tree.metadatas();
            let mut root = None;
            let mut nodes_by_selector = HashMap::default();
            let mut nodes_by_fn_name: HashMap<String, Vec<LayoutNodeSnapshot>> = HashMap::default();

            metadatas.with_entries(|entries| {
                for (&node_id, metadata) in entries {
                    let Some(computed_data) = metadata.computed_data else {
                        continue;
                    };
                    let Some(abs_position) = metadata.abs_position else {
                        continue;
                    };
                    let Some(node) = tree.get(node_id) else {
                        continue;
                    };

                    let snapshot = LayoutNodeSnapshot {
                        fn_name: node.get().fn_name.clone(),
                        position: abs_position,
                        size: PxSize::new(computed_data.width, computed_data.height),
                    };
                    nodes_by_fn_name
                        .entry(snapshot.fn_name.clone())
                        .or_default()
                        .push(snapshot.clone());

                    if node.parent().is_none() {
                        root = Some(snapshot.clone());
                    }

                    if let Some(selector) = metadata
                        .accessibility
                        .as_ref()
                        .and_then(|accessibility| accessibility.key.as_ref())
                    {
                        assert_ne!(
                            selector, "root",
                            "`root` is reserved for the root node selector in layout tests"
                        );
                        let replaced = nodes_by_selector.insert(selector.clone(), snapshot.clone());
                        assert!(
                            replaced.is_none(),
                            "duplicate layout test selector `{selector}`"
                        );
                    }
                }
            });

            Self {
                root: root.expect("layout test root node not found after layout"),
                nodes_by_selector,
                nodes_by_fn_name,
            }
        })
    }

    /// Start an assertion chain for the given selector.
    ///
    /// The selector `root` is reserved for the root node. Other selectors are
    /// resolved from `SemanticsArgs::test_tag`.
    pub fn node(&self, selector: &str) -> NodeAssert<'_> {
        let _ = self.resolve(selector);
        NodeAssert {
            snapshot: self,
            selector: selector.to_string(),
        }
    }

    fn resolve(&self, selector: &str) -> &LayoutNodeSnapshot {
        if selector == "root" {
            return &self.root;
        }

        if let Some(node) = self.nodes_by_selector.get(selector) {
            return node;
        }

        if let Some(nodes) = self.nodes_by_fn_name.get(selector) {
            return match nodes.as_slice() {
                [node] => node,
                _ => panic!(
                    "layout selector `{selector}` matched {} nodes by function name; add a test_tag to disambiguate",
                    nodes.len()
                ),
            };
        }

        let mut selectors = BTreeSet::new();
        selectors.extend(self.nodes_by_selector.keys().cloned());
        selectors.extend(self.nodes_by_fn_name.keys().cloned());
        panic!(
            "layout test selector `{selector}` not found; available selectors: {:?}",
            selectors
        )
    }
}

#[derive(Clone)]
struct LayoutNodeSnapshot {
    fn_name: String,
    position: PxPosition,
    size: PxSize,
}

impl LayoutNodeSnapshot {
    fn x(&self) -> i32 {
        self.position.x_px().raw()
    }

    fn y(&self) -> i32 {
        self.position.y_px().raw()
    }

    fn width(&self) -> i32 {
        self.size.width.raw()
    }

    fn height(&self) -> i32 {
        self.size.height.raw()
    }

    fn right(&self) -> i32 {
        self.x() + self.width()
    }

    fn bottom(&self) -> i32 {
        self.y() + self.height()
    }
}

/// Fluent assertion chain for a selected layout node.
pub struct NodeAssert<'a> {
    snapshot: &'a LayoutSnapshot,
    selector: String,
}

impl<'a> NodeAssert<'a> {
    /// Assert that the node exists.
    pub fn exists(self) -> Self {
        self
    }

    /// Returns the node's bounds in physical pixels for geometry comparisons.
    pub fn bounds(&self) -> PxRect {
        let node = self.current();
        PxRect::from_position_size(node.position, node.size)
    }

    /// Assert the node position in physical pixels.
    pub fn position(self, x: i32, y: i32) -> Self {
        let node = self.current();
        assert_eq!(
            (node.x(), node.y()),
            (x, y),
            "layout node `{}` ({}) expected position ({x}, {y}), got ({}, {})",
            self.selector,
            node.fn_name,
            node.x(),
            node.y()
        );
        self
    }

    /// Assert the node size in physical pixels.
    pub fn size(self, width: i32, height: i32) -> Self {
        let node = self.current();
        assert_eq!(
            (node.width(), node.height()),
            (width, height),
            "layout node `{}` ({}) expected size ({width}, {height}), got ({}, {})",
            self.selector,
            node.fn_name,
            node.width(),
            node.height()
        );
        self
    }

    /// Assert the node x position in physical pixels.
    pub fn x(self, value: i32) -> Self {
        let node = self.current();
        assert_eq!(
            node.x(),
            value,
            "layout node `{}` ({}) expected x {value}, got {}",
            self.selector,
            node.fn_name,
            node.x()
        );
        self
    }

    /// Assert the node y position in physical pixels.
    pub fn y(self, value: i32) -> Self {
        let node = self.current();
        assert_eq!(
            node.y(),
            value,
            "layout node `{}` ({}) expected y {value}, got {}",
            self.selector,
            node.fn_name,
            node.y()
        );
        self
    }

    /// Assert the node width in physical pixels.
    pub fn width(self, value: i32) -> Self {
        let node = self.current();
        assert_eq!(
            node.width(),
            value,
            "layout node `{}` ({}) expected width {value}, got {}",
            self.selector,
            node.fn_name,
            node.width()
        );
        self
    }

    /// Assert the node height in physical pixels.
    pub fn height(self, value: i32) -> Self {
        let node = self.current();
        assert_eq!(
            node.height(),
            value,
            "layout node `{}` ({}) expected height {value}, got {}",
            self.selector,
            node.fn_name,
            node.height()
        );
        self
    }

    /// Assert that this node is positioned below another node.
    pub fn below(self, other: &str) -> Self {
        let node = self.current();
        let other_node = self.snapshot.resolve(other);
        assert!(
            node.y() >= other_node.bottom(),
            "layout node `{}` ({}) expected to be below `{other}` ({}), got y={} while other bottom={}",
            self.selector,
            node.fn_name,
            other_node.fn_name,
            node.y(),
            other_node.bottom()
        );
        self
    }

    /// Assert that this node is positioned above another node.
    pub fn above(self, other: &str) -> Self {
        let node = self.current();
        let other_node = self.snapshot.resolve(other);
        assert!(
            node.bottom() <= other_node.y(),
            "layout node `{}` ({}) expected to be above `{other}` ({}), got bottom={} while other y={}",
            self.selector,
            node.fn_name,
            other_node.fn_name,
            node.bottom(),
            other_node.y()
        );
        self
    }

    /// Assert that this node is positioned to the left of another node.
    pub fn left_of(self, other: &str) -> Self {
        let node = self.current();
        let other_node = self.snapshot.resolve(other);
        assert!(
            node.right() <= other_node.x(),
            "layout node `{}` ({}) expected to be left of `{other}` ({}), got right={} while other x={}",
            self.selector,
            node.fn_name,
            other_node.fn_name,
            node.right(),
            other_node.x()
        );
        self
    }

    /// Assert that this node is positioned to the right of another node.
    pub fn right_of(self, other: &str) -> Self {
        let node = self.current();
        let other_node = self.snapshot.resolve(other);
        assert!(
            node.x() >= other_node.right(),
            "layout node `{}` ({}) expected to be right of `{other}` ({}), got x={} while other right={}",
            self.selector,
            node.fn_name,
            other_node.fn_name,
            node.x(),
            other_node.right()
        );
        self
    }

    /// Assert that this node shares the same start x position as another node.
    pub fn align_start_with(self, other: &str) -> Self {
        let node = self.current();
        let other_node = self.snapshot.resolve(other);
        assert_eq!(
            node.x(),
            other_node.x(),
            "layout node `{}` ({}) expected to align start with `{other}` ({}), got x={} vs {}",
            self.selector,
            node.fn_name,
            other_node.fn_name,
            node.x(),
            other_node.x()
        );
        self
    }

    /// Assert that this node shares the same top y position as another node.
    pub fn align_top_with(self, other: &str) -> Self {
        let node = self.current();
        let other_node = self.snapshot.resolve(other);
        assert_eq!(
            node.y(),
            other_node.y(),
            "layout node `{}` ({}) expected to align top with `{other}` ({}), got y={} vs {}",
            self.selector,
            node.fn_name,
            other_node.fn_name,
            node.y(),
            other_node.y()
        );
        self
    }

    /// Assert that this node has the same width as another node.
    pub fn same_width_as(self, other: &str) -> Self {
        let node = self.current();
        let other_node = self.snapshot.resolve(other);
        assert_eq!(
            node.width(),
            other_node.width(),
            "layout node `{}` ({}) expected same width as `{other}` ({}), got {} vs {}",
            self.selector,
            node.fn_name,
            other_node.fn_name,
            node.width(),
            other_node.width()
        );
        self
    }

    /// Assert that this node has the same height as another node.
    pub fn same_height_as(self, other: &str) -> Self {
        let node = self.current();
        let other_node = self.snapshot.resolve(other);
        assert_eq!(
            node.height(),
            other_node.height(),
            "layout node `{}` ({}) expected same height as `{other}` ({}), got {} vs {}",
            self.selector,
            node.fn_name,
            other_node.fn_name,
            node.height(),
            other_node.height()
        );
        self
    }

    fn current(&self) -> &LayoutNodeSnapshot {
        self.snapshot.resolve(&self.selector)
    }
}

fn reset_runtime_for_layout_test(viewport: (u32, u32)) {
    TesseraRuntime::with_mut(|runtime| {
        runtime.component_tree.reset();
        runtime.cursor_icon_request = None;
        runtime.window_minimized = false;
        runtime.window_size = [viewport.0, viewport.1];
    });
    clear_layout_snapshots();
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

/// Assert layout relationships in a headless Tessera layout test.
#[doc(hidden)]
#[macro_export]
macro_rules! __assert_layout_expect {
    ($snapshot:ident,) => {};
    (
        $snapshot:ident,
        node($selector:expr) $(.$method:ident($($args:tt)*))*;
        $($rest:tt)*
    ) => {
        $snapshot.node($selector)$(.$method($($args)*))*;
        $crate::__assert_layout_expect!($snapshot, $($rest)*);
    };
}

/// Assert layout relationships across multiple animation frames.
///
/// Frames are listed as `frame_nanos => { assertions }`, separated by commas.
#[doc(hidden)]
#[macro_export]
macro_rules! __assert_layout_frames {
    ($session:ident) => {};
    ($session:ident,) => {};
    (
        $session:ident,
        $frame_nanos:tt => { $($expect:tt)* }
        $(, $($rest:tt)*)?
    ) => {{
        let __tessera_layout_snapshot =
            $crate::testing::pump_layout_test_session(&mut $session, $frame_nanos);
        $crate::__assert_layout_expect!(__tessera_layout_snapshot, $($expect)*);
        $crate::__assert_layout_frames!($session $(, $($rest)*)?);
    }};
}

/// Assert layout relationships in a headless Tessera layout test.
#[macro_export]
macro_rules! assert_layout {
    (
        viewport: ($width:expr, $height:expr),
        content: $content:block,
        expect: {
            $($frame_nanos:tt => { $($frame_expect:tt)* }),+ $(,)?
        }
    ) => {{
        let mut __tessera_layout_session = $crate::testing::start_layout_test_session(
            $crate::testing::layout_test(|| $content).viewport_px($width, $height)
        );
        $crate::__assert_layout_frames!(
            __tessera_layout_session,
            $($frame_nanos => { $($frame_expect)* }),+
        );
    }};
    (
        viewport: ($width:expr, $height:expr),
        content: $content:block,
        expect: { $($expect:tt)* }
    ) => {{
        let __tessera_layout_snapshot = $crate::testing::layout_test(|| $content)
            .viewport_px($width, $height)
            .run();
        $crate::__assert_layout_expect!(__tessera_layout_snapshot, $($expect)*);
    }};
}

#[cfg(test)]
mod tests {
    use std::{
        cell::Cell,
        num::NonZero,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
    };

    use crate::{
        AccessibilityActionHandler, AccessibilityNode, AxisConstraint, ComputedData, Constraint,
        FrameNanosControl, LayoutModifierChild, LayoutModifierInput, LayoutModifierNode,
        LayoutResult, Modifier, NoopDrawModifier, PlacementModifierNode, Px, PxPosition,
        RenderSlot, SemanticsModifierNode, State,
        component_tree::{NodeRole, direct_layout_children},
        layout::MeasureScope,
        receive_frame_nanos, remember,
        runtime::TesseraRuntime,
        tessera,
    };

    use super::{
        last_layout_frame_diagnostics, layout_test, pump_layout_test_session,
        reset_runtime_for_layout_test, start_layout_test_session,
    };

    thread_local! { static MEASURE_ONLY_WIDTH: Cell<Option<State<i32>>> = const { Cell::new(None) }; }

    #[derive(Clone, PartialEq)]
    struct MeasureOnlyWidthPolicy {
        width: State<i32>,
    }

    impl LayoutModifierNode for MeasureOnlyWidthPolicy {
        fn measure(
            &self,
            _input: &mut MeasureScope<'_>,
        ) -> Result<LayoutResult, crate::MeasurementError> {
            Ok(LayoutResult::new(ComputedData {
                width: Px(self.width.get()),
                height: Px(20),
            }))
        }
    }

    #[tessera(crate)]
    fn measure_only_width_fixture() {
        let width = remember(|| 40);
        MEASURE_ONLY_WIDTH.set(Some(width));
        crate::layout::layout()
            .policy(MeasureOnlyWidthPolicy { width })
            .modifier(Modifier::new().push_semantics(TestTagSemanticsModifier {
                tag: "measure-only".into(),
            }));
    }

    #[test]
    fn layout_state_updates_apply_when_component_build_is_skipped() {
        let mut session = start_layout_test_session(
            layout_test(|| {
                measure_only_width_fixture();
            })
            .viewport_px(200, 100),
        );
        pump_layout_test_session(&mut session, 0)
            .node("measure-only")
            .width(40);
        let width = MEASURE_ONLY_WIDTH.get().expect("width state");
        width.set(80);
        assert!(!crate::runtime::has_pending_build_invalidations());
        pump_layout_test_session(&mut session, 16_666_667)
            .node("measure-only")
            .width(80);
        width.set(120);
        pump_layout_test_session(&mut session, 33_333_334)
            .node("measure-only")
            .width(120);
    }
    #[derive(Clone, PartialEq)]
    struct FixedSizePolicy {
        width: i32,
        height: i32,
    }

    thread_local! {
        static POSITION_ATTACHED: Cell<Option<State<bool>>> = const { Cell::new(None) };
        static POSITION_COUNTS: Cell<Option<State<(u32, u32)>>> = const { Cell::new(None) };
        static POSITION_WIDTH: Cell<Option<State<i32>>> = const { Cell::new(None) };
        static POSITION_GEOMETRY_COUNTS: Cell<Option<State<(u32, u32, u32)>>> = const { Cell::new(None) };
    }

    #[tessera(crate)]
    fn position_handler_fixture() {
        use crate::modifier::FocusModifierExt as _;

        let attached = remember(|| false);
        let counts = remember(|| (0_u32, 0_u32));
        let width = remember(|| 40);
        POSITION_ATTACHED.set(Some(attached));
        POSITION_COUNTS.set(Some(counts));
        POSITION_WIDTH.set(Some(width));
        let geometry_counts = remember(|| (0_u32, 0_u32, 0_u32));
        POSITION_GEOMETRY_COUNTS.set(Some(geometry_counts));
        let mut modifier = Modifier::new().on_globally_positioned(move |_| {
            counts.with_mut(|counts| counts.0 += 1);
        });
        if attached.get() {
            modifier = modifier
                .on_globally_positioned(move |_| {
                    counts.with_mut(|counts| counts.1 += 1);
                })
                .on_globally_positioned_precise(move |_: crate::FloatRect| {
                    geometry_counts.with_mut(|counts| counts.0 += 1);
                })
                .on_layout_coordinates(move |coordinates: crate::LayoutCoordinates| {
                    assert_eq!(coordinates.local_size.height, Px(20));
                    geometry_counts.with_mut(|counts| counts.1 += 1);
                })
                .on_local_to_window(move |transform: crate::LocalToWindow| {
                    assert_eq!(transform, crate::LocalToWindow::IDENTITY);
                    geometry_counts.with_mut(|counts| counts.2 += 1);
                });
        }
        crate::layout::layout()
            .modifier(modifier)
            .policy(FixedSizePolicy {
                width: width.get(),
                height: 20,
            });
    }

    #[test]
    fn newly_attached_position_handlers_receive_unchanged_bounds_once() {
        let mut session = start_layout_test_session(
            layout_test(|| {
                position_handler_fixture();
            })
            .viewport_px(200, 100),
        );
        pump_layout_test_session(&mut session, 0);
        let counts = POSITION_COUNTS.get().expect("counts");
        let attached = POSITION_ATTACHED.get().expect("attachment");
        let geometry_counts = POSITION_GEOMETRY_COUNTS.get().expect("geometry counts");
        assert_eq!(geometry_counts.get(), (0, 0, 0));
        assert_eq!(counts.get(), (1, 0));
        attached.set(true);
        pump_layout_test_session(&mut session, 16_666_667);
        assert_eq!(
            counts.get(),
            (1, 1),
            "only the newly attached handler receives the unchanged rectangle"
        );
        assert_eq!(
            geometry_counts.get(),
            (1, 1, 1),
            "new observer kinds must receive unchanged valid geometry"
        );
        pump_layout_test_session(&mut session, 33_333_334);
        assert_eq!(counts.get(), (1, 1), "idle layouts do not repeat callbacks");
        assert_eq!(geometry_counts.get(), (1, 1, 1));
        attached.set(false);
        pump_layout_test_session(&mut session, 50_000_001);
        attached.set(true);
        pump_layout_test_session(&mut session, 66_666_668);
        assert_eq!(
            counts.get(),
            (1, 2),
            "reattachment receives an initial rectangle"
        );
        POSITION_WIDTH.get().expect("width").set(80);
        pump_layout_test_session(&mut session, 83_333_335);
        assert_eq!(
            counts.get(),
            (2, 3),
            "changed bounds notify both attached handlers"
        );
        assert_eq!(
            geometry_counts.get(),
            (3, 3, 2),
            "size changes notify size observers while an unchanged transform remains quiet"
        );
        pump_layout_test_session(&mut session, 100_000_002);
        assert_eq!(counts.get(), (2, 3));
        assert_eq!(geometry_counts.get(), (3, 3, 2));
    }

    struct ResourceLease {
        owner: State<u32>,
        drops: Arc<AtomicUsize>,
    }

    impl Drop for ResourceLease {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::SeqCst);
            if self.owner.is_alive() {
                self.owner.with_mut(|value| *value += 1);
            }
        }
    }

    thread_local! {
        static RESOURCE_OWNER: Cell<Option<State<u32>>> = const { Cell::new(None) };
        static RESOURCE_LOCAL: Cell<Option<State<bool>>> = const { Cell::new(None) };
        static RESOURCE_CHILD: Cell<Option<State<bool>>> = const { Cell::new(None) };
    }

    #[tessera(crate)]
    fn remembered_resource_child(owner: State<u32>, drops: Arc<AtomicUsize>) {
        let _lease = remember(|| ResourceLease { owner, drops });
        crate::layout::layout();
    }

    #[tessera(crate)]
    fn remembered_resource_fixture(drops: Arc<AtomicUsize>) {
        let owner = remember(|| 0_u32);
        let local = remember(|| true);
        let child = remember(|| true);
        RESOURCE_OWNER.set(Some(owner));
        RESOURCE_LOCAL.set(Some(local));
        RESOURCE_CHILD.set(Some(child));
        crate::layout::layout().child(move || {
            if local.get() {
                let drops = drops.clone();
                let _lease = remember(|| ResourceLease { owner, drops });
            }
            if child.get() {
                remembered_resource_child(owner, drops.clone());
            }
        });
    }

    #[test]
    fn remembered_resource_destructors_update_owners_after_unmount_and_handle_shutdown() {
        let drops = Arc::new(AtomicUsize::new(0));
        let initial = drops.clone();
        let mut session = start_layout_test_session(
            layout_test(move || {
                remembered_resource_fixture(initial.clone());
            })
            .viewport_px(20, 20),
        );
        pump_layout_test_session(&mut session, 0);
        let owner = RESOURCE_OWNER.get().expect("owner");
        RESOURCE_LOCAL.get().expect("local resource").set(false);
        pump_layout_test_session(&mut session, 16_666_667);
        assert_eq!(
            owner.get(),
            1,
            "conditional remembered resources can update their owner during recycling"
        );
        RESOURCE_CHILD.get().expect("child resource").set(false);
        pump_layout_test_session(&mut session, 33_333_334);
        assert_eq!(
            owner.get(),
            2,
            "removed child components release resources outside the slot-table borrow"
        );
        assert_eq!(drops.load(Ordering::SeqCst), 2);
        RESOURCE_LOCAL.get().expect("local resource").set(true);
        RESOURCE_CHILD.get().expect("child resource").set(true);
        pump_layout_test_session(&mut session, 50_000_001);
        reset_runtime_for_layout_test((20, 20));
        assert!(!owner.is_alive());
        assert_eq!(
            drops.load(Ordering::SeqCst),
            4,
            "runtime reset invalidates owners before resource destruction"
        );

        let thread_drops = drops.clone();
        std::thread::spawn(move || {
            let mut session = start_layout_test_session(
                layout_test(move || {
                    remembered_resource_fixture(thread_drops.clone());
                })
                .viewport_px(20, 20),
            );
            pump_layout_test_session(&mut session, 0);
        })
        .join()
        .expect("thread-local resource teardown must permit liveness checks");
        assert_eq!(drops.load(Ordering::SeqCst), 6);
    }

    impl crate::modifier::LayoutModifierNode for FixedSizePolicy {
        fn measure(
            &self,
            input: &mut MeasureScope<'_>,
        ) -> Result<LayoutResult, crate::MeasurementError> {
            let mut result = LayoutResult::default();
            let child_constraint = input.parent_constraint().without_min();
            let child_count = input.children().len();
            for index in 0..child_count {
                let measured = input.children()[index].measure(input, &child_constraint)?;
                result.place_child(measured, PxPosition::ZERO);
            }

            Ok(result.with_size(ComputedData {
                width: Px::new(self.width),
                height: Px::new(self.height),
            }))
        }
    }

    #[derive(Clone, Default, PartialEq)]
    struct VerticalStackPolicy;

    impl crate::modifier::LayoutModifierNode for VerticalStackPolicy {
        fn measure(
            &self,
            input: &mut MeasureScope<'_>,
        ) -> Result<LayoutResult, crate::MeasurementError> {
            let mut result = LayoutResult::default();
            let mut current_y = Px::ZERO;
            let mut max_width = Px::ZERO;
            let child_constraint = input.parent_constraint().without_min();

            let child_count = input.children().len();
            for index in 0..child_count {
                let child_size = input.children()[index].measure(input, &child_constraint)?;
                result.place_child(child_size, PxPosition::new(Px::ZERO, current_y));
                current_y += child_size.height;
                max_width = max_width.max(child_size.width);
            }

            Ok(result.with_size(ComputedData {
                width: max_width,
                height: current_y,
            }))
        }
    }

    #[derive(Clone, PartialEq)]
    struct OffsetChildPolicy {
        x: i32,
    }

    impl crate::modifier::LayoutModifierNode for OffsetChildPolicy {
        fn measure(
            &self,
            input: &mut MeasureScope<'_>,
        ) -> Result<LayoutResult, crate::MeasurementError> {
            let mut result = LayoutResult::default();
            let child_constraint = input.parent_constraint().without_min();
            let child_size = input
                .children()
                .first()
                .copied()
                .expect("offset child policy requires a child")
                .measure(input, &child_constraint)?;
            result.place_child(child_size, PxPosition::new(Px::new(self.x), Px::ZERO));

            Ok(result.with_size(ComputedData {
                width: Px::new(self.x + child_size.width.raw()),
                height: child_size.height,
            }))
        }
    }

    struct TestTagSemanticsModifier {
        tag: String,
    }

    impl SemanticsModifierNode for TestTagSemanticsModifier {
        fn apply(
            &self,
            accessibility: &mut AccessibilityNode,
            _action_handler: &mut Option<AccessibilityActionHandler>,
        ) {
            *accessibility = accessibility.clone().with_key(self.tag.clone());
        }
    }

    #[derive(Clone, Copy, PartialEq)]
    struct AnimatedWidthModifierNode {
        width: i32,
        height: i32,
    }

    impl LayoutModifierNode for AnimatedWidthModifierNode {
        fn measure_content(
            &self,
            _input: &LayoutModifierInput<'_>,
            child: &mut dyn LayoutModifierChild,
        ) -> Result<crate::LayoutResult, crate::MeasurementError> {
            let child_size = child.measure(&Constraint::exact(
                Px::new(self.width),
                Px::new(self.height),
            ))?;
            child.place(PxPosition::ZERO);
            Ok(crate::LayoutResult::new(child_size))
        }
    }

    #[derive(Clone, Copy, PartialEq)]
    struct AnimatedOffsetPlacementNode {
        x: i32,
        y: i32,
    }

    impl PlacementModifierNode for AnimatedOffsetPlacementNode {
        fn transform_position(&self, position: PxPosition) -> PxPosition {
            position.offset(Px::new(self.x), Px::new(self.y))
        }
    }

    #[derive(Clone, Copy, PartialEq)]
    struct TestPaddingModifierNode {
        padding: i32,
    }

    impl LayoutModifierNode for TestPaddingModifierNode {
        fn measure_content(
            &self,
            input: &LayoutModifierInput<'_>,
            child: &mut dyn LayoutModifierChild,
        ) -> Result<crate::LayoutResult, crate::MeasurementError> {
            let padding = Px::new(self.padding);
            let parent = input.layout_input.parent_constraint();
            let constraint = Constraint::new(
                AxisConstraint::new(
                    (parent.width().min - padding - padding).max(Px::ZERO),
                    parent
                        .width()
                        .max
                        .map(|value| (value - padding - padding).max(Px::ZERO)),
                ),
                AxisConstraint::new(
                    (parent.height().min - padding - padding).max(Px::ZERO),
                    parent
                        .height()
                        .max
                        .map(|value| (value - padding - padding).max(Px::ZERO)),
                ),
            );
            let child_size = child.measure(&constraint)?;
            child.place(PxPosition::new(padding, padding));
            Ok(crate::LayoutResult::new(ComputedData {
                width: child_size.width + padding + padding,
                height: child_size.height + padding + padding,
            }))
        }
    }

    #[tessera(crate)]
    fn tagged_box(tag: Option<String>, width: Option<i32>, height: Option<i32>) {
        let tag = tag.unwrap_or_default();
        let width = width.unwrap_or_default();
        let height = height.unwrap_or_default();
        crate::layout::layout()
            .policy(FixedSizePolicy { width, height })
            .draw(NoopDrawModifier)
            .modifier(Modifier::new().push_semantics(TestTagSemanticsModifier { tag }));
    }

    #[tessera(crate)]
    fn responsive_box(tag: Option<String>) {
        let tag = tag.unwrap_or_default();
        crate::layout::layout()
            .policy(crate::layout::DefaultLayoutNode)
            .draw(NoopDrawModifier)
            .modifier(Modifier::new().push_semantics(TestTagSemanticsModifier { tag }));
    }

    #[tessera(crate)]
    fn stack_content() {
        crate::layout::layout()
            .policy(VerticalStackPolicy)
            .draw(NoopDrawModifier)
            .modifier(Modifier::new())
            .child(|| {
                title_box();
                button_box();
            });
    }

    #[tessera(crate)]
    fn sample_layout() {
        crate::layout::layout()
            .policy(FixedSizePolicy {
                width: 800,
                height: 600,
            })
            .draw(NoopDrawModifier)
            .modifier(Modifier::new())
            .child(|| {
                stack_content();
            });
    }

    #[tessera(crate)]
    fn title_box() {
        tagged_box().tag("title".to_string()).width(200).height(40);
    }

    #[tessera(crate)]
    fn button_box() {
        tagged_box().tag("button".to_string()).width(120).height(32);
    }

    #[tessera(crate)]
    fn animated_layout_sample() {
        let offset = remember(|| 0_i32);

        receive_frame_nanos(move |frame_nanos| {
            let next_offset = if frame_nanos >= 200_000_000 {
                100
            } else if frame_nanos >= 100_000_000 {
                50
            } else {
                0
            };
            offset.set(next_offset);
            if frame_nanos >= 200_000_000 {
                FrameNanosControl::Stop
            } else {
                FrameNanosControl::Continue
            }
        });

        crate::layout::layout()
            .policy(OffsetChildPolicy { x: offset.get() })
            .draw(NoopDrawModifier)
            .modifier(Modifier::new())
            .child(|| {
                tagged_box().tag("moving".to_string()).width(20).height(20);
            });
    }

    #[tessera(crate)]
    fn animated_modifier_layout_sample() {
        let width = remember(|| 20_i32);

        receive_frame_nanos(move |frame_nanos| {
            let next_width = if frame_nanos >= 200_000_000 {
                100
            } else if frame_nanos >= 100_000_000 {
                50
            } else {
                20
            };
            width.set(next_width);
            if frame_nanos >= 200_000_000 {
                FrameNanosControl::Stop
            } else {
                FrameNanosControl::Continue
            }
        });

        crate::layout::layout()
            .policy(crate::layout::DefaultLayoutNode)
            .draw(NoopDrawModifier)
            .modifier(
                Modifier::new()
                    .push_semantics(TestTagSemanticsModifier {
                        tag: "modifier_box".to_string(),
                    })
                    .push_layout(AnimatedWidthModifierNode {
                        width: width.get(),
                        height: 20,
                    }),
            )
            .child(|| {
                responsive_box().tag("modifier_box_child".to_string());
            });
    }

    #[tessera(crate)]
    fn animated_offset_modifier_sample() {
        let x = remember(|| 0_i32);

        receive_frame_nanos(move |frame_nanos| {
            let next_x = if frame_nanos >= 200_000_000 {
                100
            } else if frame_nanos >= 100_000_000 {
                50
            } else {
                0
            };
            x.set(next_x);
            if frame_nanos >= 200_000_000 {
                FrameNanosControl::Stop
            } else {
                FrameNanosControl::Continue
            }
        });

        crate::layout::layout()
            .policy(FixedSizePolicy {
                width: 20,
                height: 20,
            })
            .draw(NoopDrawModifier)
            .modifier(
                Modifier::new()
                    .push_semantics(TestTagSemanticsModifier {
                        tag: "offset_box".to_string(),
                    })
                    .push_placement(AnimatedOffsetPlacementNode { x: x.get(), y: 0 }),
            );
    }

    #[tessera(crate)]
    fn slot_host(slot: Option<RenderSlot>) {
        let slot = slot.unwrap_or_default();
        crate::layout::layout()
            .policy(VerticalStackPolicy)
            .draw(NoopDrawModifier)
            .modifier(Modifier::new())
            .child(move || {
                slot.render();
            });
    }

    #[tessera(crate)]
    fn animated_nested_slot_sample() {
        let width = remember(|| 20_i32);

        receive_frame_nanos(move |frame_nanos| {
            let next_width = if frame_nanos >= 200_000_000 {
                100
            } else if frame_nanos >= 100_000_000 {
                50
            } else {
                20
            };
            width.set(next_width);
            if frame_nanos >= 200_000_000 {
                FrameNanosControl::Stop
            } else {
                FrameNanosControl::Continue
            }
        });

        crate::layout::layout()
            .policy(FixedSizePolicy {
                width: 200,
                height: 100,
            })
            .draw(NoopDrawModifier)
            .modifier(Modifier::new())
            .child(move || {
                slot_host().slot(move || {
                    slot_host().slot(move || {
                        tagged_box()
                            .tag("nested_moving".to_string())
                            .width(width.get())
                            .height(20);
                    });
                });
            });
    }

    #[tessera(crate)]
    fn ordered_layout_modifier_sample() {
        crate::layout::layout()
            .policy(crate::layout::DefaultLayoutNode)
            .draw(NoopDrawModifier)
            .modifier(
                Modifier::new()
                    .push_semantics(TestTagSemanticsModifier {
                        tag: "ordered_parent".to_string(),
                    })
                    .push_layout(TestPaddingModifierNode { padding: 10 })
                    .push_layout(AnimatedWidthModifierNode {
                        width: 50,
                        height: 20,
                    }),
            )
            .child(|| {
                responsive_box().tag("ordered_child".to_string());
            });
    }

    #[tessera(crate)]
    fn explicit_layout_boundary_sample() {
        crate::layout::layout().policy(FixedSizePolicy {
            width: 24,
            height: 12,
        });
    }

    #[test]
    fn assert_layout_macro_smoke() {
        crate::assert_layout! {
            viewport: (800, 600),
            content: {
                sample_layout();
            },
            expect: {
                node("root").size(800, 600).position(0, 0);
                node("title").x(0).y(0).size(200, 40);
                node("button").below("title").align_start_with("title").size(120, 32);
            }
        }
    }

    #[derive(Clone, Copy, PartialEq)]
    struct IntrinsicAwarePolicy;

    impl LayoutModifierNode for IntrinsicAwarePolicy {
        fn measure(
            &self,
            scope: &mut MeasureScope<'_>,
        ) -> Result<LayoutResult, crate::MeasurementError> {
            let offset = if scope.is_intrinsic() { 0 } else { 20 };
            Ok(LayoutResult::new(ComputedData {
                width: Px(10 + offset),
                height: Px(14 + offset),
            }))
        }
    }

    #[derive(Clone, Copy, PartialEq)]
    struct IntrinsicProbePolicy {
        intrinsic_first: bool,
    }

    impl LayoutModifierNode for IntrinsicProbePolicy {
        fn measure(
            &self,
            scope: &mut MeasureScope<'_>,
        ) -> Result<LayoutResult, crate::MeasurementError> {
            let child = scope.children()[0];
            let constraint = scope.parent_constraint().without_min();
            if !self.intrinsic_first {
                assert_eq!(
                    child.measure(scope, &constraint)?.size(),
                    ComputedData {
                        width: Px(38),
                        height: Px(42)
                    }
                );
            }
            assert_eq!(
                child.probe_intrinsic(scope, &constraint)?.size(),
                ComputedData {
                    width: Px(18),
                    height: Px(22)
                }
            );
            assert!(!scope.is_intrinsic());
            let normal = child.measure(scope, &constraint)?;
            assert_eq!(
                normal.size(),
                ComputedData {
                    width: Px(38),
                    height: Px(42)
                }
            );
            let mut result = LayoutResult::new(normal.size());
            result.place_child(child, PxPosition::new(Px(7), Px(9)));
            Ok(result)
        }
    }

    #[tessera(crate)]
    fn intrinsic_probe_fixture(intrinsic_first: bool) {
        crate::layout::layout()
            .policy(IntrinsicProbePolicy { intrinsic_first })
            .child(|| {
                crate::layout::layout()
                    .policy(IntrinsicAwarePolicy)
                    .modifier(
                        Modifier::new()
                            .push_layout(TestPaddingModifierNode { padding: 4 })
                            .push_semantics(TestTagSemanticsModifier {
                                tag: "intrinsic-child".to_string(),
                            }),
                    );
            });
    }

    #[test]
    fn intrinsic_probes_preserve_normal_measurement_cache_and_placements() {
        for intrinsic_first in [false, true] {
            let mut session = start_layout_test_session(
                layout_test(move || {
                    intrinsic_probe_fixture(intrinsic_first);
                })
                .viewport_px(200, 200),
            );
            for now in [0, 16_666_667, 33_333_334] {
                let snapshot = pump_layout_test_session(&mut session, now);
                snapshot.node("intrinsic-child").size(38, 42).position(7, 9);
            }
            assert!(
                last_layout_frame_diagnostics()
                    .expect("diagnostics")
                    .cache_hits_direct
                    > 0
            );
            assert_eq!(
                last_layout_frame_diagnostics()
                    .expect("diagnostics")
                    .cache_store_count,
                0
            );
        }
    }

    #[test]
    fn assert_layout_macro_pumps_animation_frames() {
        crate::assert_layout! {
            viewport: (200, 100),
            content: {
                animated_layout_sample();
            },
            expect: {
                0 => {
                    node("moving").position(0, 0).size(20, 20);
                },
                100_000_000 => {
                    node("moving").position(50, 0).size(20, 20);
                },
                200_000_000 => {
                    node("moving").position(100, 0).size(20, 20);
                }
            }
        }
    }

    #[test]
    fn layout_test_session_pumps_animation_frames() {
        let mut session = crate::testing::start_layout_test_session(
            crate::testing::layout_test(|| {
                animated_layout_sample();
            })
            .viewport_px(200, 100),
        );

        let first = crate::testing::pump_layout_test_session(&mut session, 0);
        first.node("moving").position(0, 0).size(20, 20);

        let mid = crate::testing::advance_layout_test_session_by_nanos(&mut session, 100_000_000);
        mid.node("moving").position(50, 0).size(20, 20);

        let end = crate::testing::advance_layout_test_session_by_nanos(&mut session, 100_000_000);
        end.node("moving").position(100, 0).size(20, 20);
        assert_eq!(
            crate::testing::current_layout_test_frame_nanos(&session),
            200_000_000
        );
    }

    #[test]
    fn assert_layout_macro_pumps_modifier_driven_animation_frames() {
        crate::assert_layout! {
            viewport: (200, 100),
            content: {
                animated_modifier_layout_sample();
            },
            expect: {
                0 => {
                    node("modifier_box").size(20, 20);
                },
                100_000_000 => {
                    node("modifier_box").size(50, 20);
                },
                200_000_000 => {
                    node("modifier_box").size(100, 20);
                }
            }
        }
    }

    #[test]
    fn assert_layout_macro_pumps_offset_modifier_frames() {
        crate::assert_layout! {
            viewport: (200, 100),
            content: {
                animated_offset_modifier_sample();
            },
            expect: {
                0 => {
                    node("offset_box").position(0, 0).size(20, 20);
                },
                100_000_000 => {
                    node("offset_box").position(50, 0).size(20, 20);
                },
                200_000_000 => {
                    node("offset_box").position(100, 0).size(20, 20);
                }
            }
        }
    }

    #[test]
    fn assert_layout_macro_pumps_nested_slot_animation_frames() {
        crate::assert_layout! {
            viewport: (200, 100),
            content: {
                animated_nested_slot_sample();
            },
            expect: {
                0 => {
                    node("nested_moving").size(20, 20);
                },
                100_000_000 => {
                    node("nested_moving").size(50, 20);
                },
                200_000_000 => {
                    node("nested_moving").size(100, 20);
                }
            }
        }
    }

    #[test]
    fn layout_modifier_order_follows_source_chain() {
        crate::assert_layout! {
            viewport: (200, 100),
            content: {
                ordered_layout_modifier_sample();
            },
            expect: {
                node("ordered_parent").position(0, 0).size(70, 40);
                node("ordered_child").position(10, 10).size(50, 20);
            }
        }
    }

    #[test]
    fn layout_keeps_composition_boundary_and_emits_layout_child() {
        reset_runtime_for_layout_test((100, 100));
        let _ = crate::build_tree::build_component_tree(&explicit_layout_boundary_sample);

        TesseraRuntime::with(|runtime| {
            let tree = runtime.component_tree.tree();
            let root = tree
                .get_node_id_at(NonZero::new(1).expect("root node index must be non-zero"))
                .expect("root node must exist after build");

            let mut layout_composition = None;
            let mut layout_node = None;

            for edge in root.traverse(tree) {
                let indextree::NodeEdge::Start(node_id) = edge else {
                    continue;
                };
                let Some(node_ref) = tree.get(node_id) else {
                    continue;
                };
                let node = node_ref.get();
                if node.fn_name != "layout" {
                    continue;
                }

                match node.role {
                    NodeRole::Composition => {
                        layout_composition =
                            Some((node_id, node.instance_key, node.replay.is_some()));
                    }
                    NodeRole::Layout => {
                        layout_node = Some((node_id, node.instance_key, node.replay.is_none()));
                    }
                }
            }

            let (layout_composition_id, composition_instance_key, has_replay) =
                layout_composition.expect("layout composition boundary must exist");
            let (layout_node_id, layout_instance_key, has_no_replay) =
                layout_node.expect("emitted layout node must exist");

            assert!(
                has_replay,
                "layout composition boundary must keep replay metadata"
            );
            assert!(
                has_no_replay,
                "emitted layout node must not become its own replay boundary"
            );
            assert_ne!(
                composition_instance_key, layout_instance_key,
                "layout node must keep a distinct node identity"
            );
            assert_eq!(
                layout_node_id.parent(tree),
                Some(layout_composition_id),
                "layout node must be emitted directly under the layout composition boundary"
            );
            assert_eq!(
                direct_layout_children(layout_composition_id, tree),
                vec![layout_node_id].into(),
                "direct layout children must resolve to the emitted layout node"
            );
        });
    }

    thread_local! { static INTRINSIC_WIDTH: Cell<Option<State<i32>>> = const { Cell::new(None) }; }

    #[derive(Clone, PartialEq)]
    struct IntrinsicStateLeaf {
        width: State<i32>,
    }

    impl LayoutModifierNode for IntrinsicStateLeaf {
        fn measure(
            &self,
            scope: &mut MeasureScope<'_>,
        ) -> Result<LayoutResult, crate::MeasurementError> {
            Ok(LayoutResult::new(ComputedData {
                width: Px(if scope.is_intrinsic() {
                    self.width.get()
                } else {
                    20
                }),
                height: Px(20),
            }))
        }
    }

    #[derive(Clone, PartialEq)]
    struct IntrinsicPositionPolicy;

    impl LayoutModifierNode for IntrinsicPositionPolicy {
        fn measure(
            &self,
            scope: &mut MeasureScope<'_>,
        ) -> Result<LayoutResult, crate::MeasurementError> {
            let child = scope.children()[0];
            let natural = child.probe_intrinsic(scope, &Constraint::NONE)?;
            let normal = child.measure(scope, &Constraint::NONE)?;
            let mut result = LayoutResult::new(ComputedData {
                width: natural.width + normal.width,
                height: normal.height,
            });
            result.place_child(child, PxPosition::new(natural.width, Px::ZERO));
            Ok(result)
        }
    }

    #[tessera(crate)]
    fn intrinsic_state_fixture() {
        let width = remember(|| 40);
        INTRINSIC_WIDTH.set(Some(width));
        crate::layout::layout()
            .policy(IntrinsicPositionPolicy)
            .child(move || {
                crate::layout::layout()
                    .policy(IntrinsicStateLeaf { width })
                    .modifier(Modifier::new().push_semantics(TestTagSemanticsModifier {
                        tag: "intrinsic-state".into(),
                    }));
            });
    }

    #[test]
    fn intrinsic_state_dependencies_survive_normal_measurement_and_invalidate_the_probe_owner() {
        let mut session = start_layout_test_session(
            layout_test(|| {
                intrinsic_state_fixture();
            })
            .viewport_px(240, 100),
        );
        pump_layout_test_session(&mut session, 0)
            .node("intrinsic-state")
            .size(20, 20)
            .position(40, 0);
        let width = INTRINSIC_WIDTH.get().unwrap();
        for (frame, value) in [(16_666_667, 70), (33_333_334, 30)] {
            width.set(value);
            assert!(!crate::runtime::has_pending_build_invalidations());
            pump_layout_test_session(&mut session, frame)
                .node("intrinsic-state")
                .size(20, 20)
                .position(value, 0);
        }
    }

    thread_local! {
        static PHASE_OFFSET: Cell<Option<State<i32>>> = const { Cell::new(None) };
        static PHASE_MEASURES: Cell<usize> = const { Cell::new(0) };
        static PHASE_BUILDS: Cell<usize> = const { Cell::new(0) };
        static PHASE_TRANSFORMS: Cell<usize> = const { Cell::new(0) };
    }

    #[derive(Clone, PartialEq)]
    struct PlacementStatePolicy {
        offset: State<i32>,
        transform: bool,
    }

    impl LayoutModifierNode for PlacementStatePolicy {
        fn measure(
            &self,
            scope: &mut MeasureScope<'_>,
        ) -> Result<LayoutResult, crate::MeasurementError> {
            PHASE_MEASURES.set(PHASE_MEASURES.get() + 1);
            let child = scope.children()[0];
            child.measure(scope, &Constraint::exact(Px(20), Px(20)))?;
            let offset = scope.placement(|| if self.transform { 0 } else { self.offset.get() });
            let mut result = LayoutResult::new(ComputedData {
                width: Px(200),
                height: Px(60),
            });
            result.place_child(child, PxPosition::new(Px(offset), Px::ZERO));
            Ok(result)
        }

        fn place_children(
            &self,
            scope: &crate::layout::PlacementScope<'_>,
            _inner: &crate::layout::PlacementResult,
        ) -> Option<crate::layout::PlacementResult> {
            let offset = if self.transform { 0 } else { self.offset.get() };
            let mut result = LayoutResult::new(scope.size());
            result.place_child(scope.children()[0], PxPosition::new(Px(offset), Px::ZERO));
            Some(result.into_placement_result())
        }
    }

    #[derive(PartialEq)]
    struct StateOffsetModifier {
        offset: State<i32>,
    }

    impl PlacementModifierNode for StateOffsetModifier {
        fn transform_position(&self, position: PxPosition) -> PxPosition {
            PHASE_TRANSFORMS.set(PHASE_TRANSFORMS.get() + 1);
            position.offset(Px(self.offset.get()), Px::ZERO)
        }
    }

    #[tessera(crate)]
    fn placement_state_fixture(transform: bool) {
        PHASE_BUILDS.set(PHASE_BUILDS.get() + 1);
        let offset = remember(|| 0);
        PHASE_OFFSET.set(Some(offset));
        let modifier = if transform {
            Modifier::new().push_placement(StateOffsetModifier { offset })
        } else {
            Modifier::new()
        };
        crate::layout::layout()
            .policy(PlacementStatePolicy { offset, transform })
            .modifier(modifier)
            .child(|| {
                tagged_box()
                    .tag("phase-child".to_string())
                    .width(20)
                    .height(20);
            });
    }

    #[test]
    fn placement_and_transform_state_move_descendants_without_build_or_measure() {
        for transform in [false, true] {
            PHASE_MEASURES.set(0);
            PHASE_BUILDS.set(0);
            PHASE_TRANSFORMS.set(0);
            let mut session = start_layout_test_session(
                layout_test(move || {
                    placement_state_fixture(transform);
                })
                .viewport_px(240, 100),
            );
            pump_layout_test_session(&mut session, 0)
                .node("phase-child")
                .position(0, 0);
            let measures = PHASE_MEASURES.get();
            let builds = PHASE_BUILDS.get();
            let transforms = PHASE_TRANSFORMS.get();
            pump_layout_test_session(&mut session, 16_666_667)
                .node("phase-child")
                .position(0, 0);
            assert_eq!(
                last_layout_frame_diagnostics()
                    .unwrap()
                    .geometry_nodes_updated,
                0
            );
            assert_eq!(PHASE_TRANSFORMS.get(), transforms);
            PHASE_OFFSET.get().unwrap().set(35);
            assert!(!crate::runtime::has_pending_build_invalidations());
            pump_layout_test_session(&mut session, 33_333_334)
                .node("phase-child")
                .position(35, 0);
            assert_eq!(PHASE_MEASURES.get(), measures);
            assert_eq!(PHASE_BUILDS.get(), builds);
            assert!(
                last_layout_frame_diagnostics()
                    .unwrap()
                    .geometry_nodes_updated
                    > 0
            );
            pump_layout_test_session(&mut session, 50_000_001)
                .node("phase-child")
                .position(35, 0);
            assert_eq!(
                last_layout_frame_diagnostics()
                    .unwrap()
                    .geometry_nodes_updated,
                0
            );
        }
    }

    #[tessera(crate)]
    fn inner_layout_cache_sample() {
        crate::layout::layout();
    }

    #[tessera(crate)]
    fn nested_composition_layout_sample() {
        crate::layout::layout().child(|| inner_layout_cache_sample());
    }

    #[test]
    fn stable_frame_reuses_layout_cache_across_composition_boundaries() {
        let mut session = start_layout_test_session(
            layout_test(|| nested_composition_layout_sample()).viewport_px(120, 80),
        );
        let _ = pump_layout_test_session(&mut session, 0);
        let _ = pump_layout_test_session(&mut session, 16_666_667);

        let diagnostics = last_layout_frame_diagnostics().expect("layout diagnostics");
        assert_eq!(
            diagnostics.geometry_nodes_updated, 0,
            "unchanged subtrees retain window geometry"
        );
        assert_eq!(
            diagnostics.dirty_nodes_structural, 0,
            "unchanged frames retain layout adjacency"
        );
        assert_eq!(
            diagnostics.cache_miss_no_entry, 0,
            "composition boundaries must not force descendant layout nodes to miss the cache"
        );
        assert_eq!(
            diagnostics.measure_node_calls, 1,
            "a stable frame must resolve from the root layout cache without re-measuring"
        );
    }
}
