//! Headless layout testing helpers for Tessera UI.
//!
//! ## Usage
//!
//! Assert positions and sizes of tagged nodes without creating a real renderer.

use std::{collections::BTreeSet, time::Duration};

use rustc_hash::FxHashMap as HashMap;

use crate::{
    Px, PxPosition, PxSize,
    build_tree::build_component_tree,
    component_tree::{ComputeMode, ComputeParams, clear_layout_snapshots},
    context::{reset_component_context_tracking, reset_context_read_dependencies},
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
        let mut session = __private::start_layout_test_session(self);
        __private::pump_layout_test_session(&mut session, 0)
    }
}

#[doc(hidden)]
pub mod __private {
    use super::*;

    /// Runs a headless Tessera test across multiple animation frames.
    pub struct LayoutTestSession<F>
    where
        F: Fn(),
    {
        content: F,
        viewport: (u32, u32),
        frame_origin: Instant,
        current_frame_nanos: u64,
    }

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
        }
    }

    pub fn pump_layout_test_session<F>(
        session: &mut LayoutTestSession<F>,
        frame_nanos: u64,
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

        TesseraRuntime::with_mut(|runtime| {
            let _ = runtime.component_tree.compute(
                ComputeParams {
                    screen_size,
                    cursor_position: None,
                    pointer_changes: Vec::new(),
                    keyboard_events: Vec::new(),
                    ime_events: Vec::new(),
                    retry_focus_move: None,
                    retry_focus_reveal: false,
                    modifiers: winit::keyboard::ModifiersState::default(),
                    layout_dirty_nodes: &layout_dirty_nodes,
                },
                ComputeMode::LayoutOnly,
            );
        });
        flush_pending_focus_callbacks();

        LayoutSnapshot::capture()
    }

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

    pub fn current_layout_test_frame_nanos<F>(session: &LayoutTestSession<F>) -> u64
    where
        F: Fn(),
    {
        session.current_frame_nanos
    }
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

            for metadata_entry in metadatas.iter() {
                let node_id = *metadata_entry.key();
                let metadata = metadata_entry.value();
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
        self.position.x.raw()
    }

    fn y(&self) -> i32 {
        self.position.y.raw()
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
#[doc(hidden)]
#[macro_export]
macro_rules! __assert_layout_frames {
    ($session:ident) => {};
    ($session:ident,) => {};
    (
        $session:ident,
        $frame_nanos:expr => { $($expect:tt)* }
        $(, $($rest:tt)*)?
    ) => {{
        let __tessera_layout_snapshot =
            $crate::testing::__private::pump_layout_test_session(&mut $session, $frame_nanos);
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
            $first_frame_nanos:expr => { $($first_expect:tt)* }
            $(, $frame_nanos:expr => { $($frame_expect:tt)* })* $(,)?
        }
    ) => {{
        let mut __tessera_layout_session = $crate::testing::__private::start_layout_test_session(
            $crate::testing::layout_test(|| $content).viewport_px($width, $height)
        );
        $crate::__assert_layout_frames!(
            __tessera_layout_session,
            $first_frame_nanos => { $($first_expect)* }
            $(, $frame_nanos => { $($frame_expect)* })*
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
    use crate::{
        AccessibilityActionHandler, AccessibilityNode, AxisConstraint, ComputedData, Constraint,
        FrameNanosControl, LayoutModifierChild, LayoutModifierInput, LayoutModifierNode,
        LayoutPolicy, LayoutResult, Modifier, NoopRenderPolicy, PlacementModifierNode, Px,
        PxPosition, RenderSlot, SemanticsModifierNode, layout::MeasureScope, receive_frame_nanos,
        remember, tessera,
    };
    #[derive(Clone, PartialEq)]
    struct FixedSizePolicy {
        width: i32,
        height: i32,
    }

    impl LayoutPolicy for FixedSizePolicy {
        fn measure(
            &self,
            input: &MeasureScope<'_>,
        ) -> Result<LayoutResult, crate::MeasurementError> {
            let mut result = LayoutResult::default();
            for child in input.children() {
                let _ = child.measure_in_parent_constraint(input.parent_constraint())?;
                result.place_child(child, PxPosition::ZERO);
            }

            Ok(result.with_size(ComputedData {
                width: Px::new(self.width),
                height: Px::new(self.height),
            }))
        }
    }

    #[derive(Clone, Default, PartialEq)]
    struct VerticalStackPolicy;

    impl LayoutPolicy for VerticalStackPolicy {
        fn measure(
            &self,
            input: &MeasureScope<'_>,
        ) -> Result<LayoutResult, crate::MeasurementError> {
            let mut result = LayoutResult::default();
            let mut current_y = Px::ZERO;
            let mut max_width = Px::ZERO;

            for child in input.children() {
                let child_size = child.measure_in_parent_constraint(input.parent_constraint())?;
                result.place_child(child, PxPosition::new(Px::ZERO, current_y));
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

    impl LayoutPolicy for OffsetChildPolicy {
        fn measure(
            &self,
            input: &MeasureScope<'_>,
        ) -> Result<LayoutResult, crate::MeasurementError> {
            let mut result = LayoutResult::default();
            let child = input
                .children()
                .first()
                .copied()
                .expect("offset child policy requires a child");
            let child_size = child.measure_in_parent_constraint(input.parent_constraint())?;
            result.place_child(child, PxPosition::new(Px::new(self.x), Px::ZERO));

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
        fn measure(
            &self,
            _input: &LayoutModifierInput<'_>,
            child: &mut dyn LayoutModifierChild,
        ) -> Result<crate::LayoutModifierOutput, crate::MeasurementError> {
            let child_size = child.measure(&Constraint::exact(
                Px::new(self.width),
                Px::new(self.height),
            ))?;
            child.place(PxPosition::ZERO);
            Ok(crate::LayoutModifierOutput { size: child_size })
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
        fn measure(
            &self,
            input: &LayoutModifierInput<'_>,
            child: &mut dyn LayoutModifierChild,
        ) -> Result<crate::LayoutModifierOutput, crate::MeasurementError> {
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
            Ok(crate::LayoutModifierOutput {
                size: ComputedData {
                    width: child_size.width + padding + padding,
                    height: child_size.height + padding + padding,
                },
            })
        }
    }

    #[tessera(crate)]
    fn tagged_box(tag: String, width: i32, height: i32) {
        crate::layout::layout()
            .layout_policy(FixedSizePolicy { width, height })
            .render_policy(NoopRenderPolicy)
            .modifier(Modifier::new().push_semantics(TestTagSemanticsModifier { tag }));
    }

    #[tessera(crate)]
    fn responsive_box(tag: String) {
        crate::layout::layout()
            .layout_policy(crate::layout::DefaultLayoutPolicy)
            .render_policy(NoopRenderPolicy)
            .modifier(Modifier::new().push_semantics(TestTagSemanticsModifier { tag }));
    }

    #[tessera(crate)]
    fn stack_content() {
        crate::layout::layout()
            .layout_policy(VerticalStackPolicy)
            .render_policy(NoopRenderPolicy)
            .modifier(Modifier::new())
            .child(|| {
                title_box();
                button_box();
            });
    }

    #[tessera(crate)]
    fn sample_layout() {
        crate::layout::layout()
            .layout_policy(FixedSizePolicy {
                width: 800,
                height: 600,
            })
            .render_policy(NoopRenderPolicy)
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
            .layout_policy(OffsetChildPolicy { x: offset.get() })
            .render_policy(NoopRenderPolicy)
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
            .layout_policy(crate::layout::DefaultLayoutPolicy)
            .render_policy(NoopRenderPolicy)
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
            .layout_policy(FixedSizePolicy {
                width: 20,
                height: 20,
            })
            .render_policy(NoopRenderPolicy)
            .modifier(
                Modifier::new()
                    .push_semantics(TestTagSemanticsModifier {
                        tag: "offset_box".to_string(),
                    })
                    .push_placement(AnimatedOffsetPlacementNode { x: x.get(), y: 0 }),
            );
    }

    #[tessera(crate)]
    fn slot_host(slot: RenderSlot) {
        crate::layout::layout()
            .layout_policy(VerticalStackPolicy)
            .render_policy(NoopRenderPolicy)
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
            .layout_policy(FixedSizePolicy {
                width: 200,
                height: 100,
            })
            .render_policy(NoopRenderPolicy)
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
            .layout_policy(crate::layout::DefaultLayoutPolicy)
            .render_policy(NoopRenderPolicy)
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
        let mut session = crate::testing::__private::start_layout_test_session(
            crate::testing::layout_test(|| {
                animated_layout_sample();
            })
            .viewport_px(200, 100),
        );

        let first = crate::testing::__private::pump_layout_test_session(&mut session, 0);
        first.node("moving").position(0, 0).size(20, 20);

        let mid = crate::testing::__private::advance_layout_test_session_by_nanos(
            &mut session,
            100_000_000,
        );
        mid.node("moving").position(50, 0).size(20, 20);

        let end = crate::testing::__private::advance_layout_test_session_by_nanos(
            &mut session,
            100_000_000,
        );
        end.node("moving").position(100, 0).size(20, 20);
        assert_eq!(
            crate::testing::__private::current_layout_test_frame_nanos(&session),
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
}
