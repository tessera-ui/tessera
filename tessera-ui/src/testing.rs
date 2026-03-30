//! Headless layout testing helpers for Tessera UI.
//!
//! ## Usage
//!
//! Assert positions and sizes of tagged nodes without creating a real renderer.

use std::{collections::BTreeSet, time::Instant};

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
    },
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
        reset_runtime_for_layout_test(self.viewport);
        begin_frame_clock(Instant::now());
        let _ = build_component_tree(&self.content);
        let layout_dirty_nodes = take_layout_dirty_nodes();
        let screen_size = PxSize::new(
            Px::new(self.viewport.0 as i32),
            Px::new(self.viewport.1 as i32),
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

/// Assert layout relationships in a headless Tessera layout test.
#[macro_export]
macro_rules! assert_layout {
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
        AccessibilityActionHandler, AccessibilityNode, ComputedData, LayoutInput, LayoutOutput,
        LayoutPolicy, Modifier, NoopRenderPolicy, Px, PxPosition, SemanticsModifierNode, tessera,
    };

    #[derive(Clone, PartialEq)]
    struct FixedSizePolicy {
        width: i32,
        height: i32,
    }

    impl LayoutPolicy for FixedSizePolicy {
        fn measure(
            &self,
            input: &LayoutInput<'_>,
            output: &mut LayoutOutput<'_>,
        ) -> Result<ComputedData, crate::MeasurementError> {
            for child_id in input.children_ids() {
                let _ = input.measure_child_in_parent_constraint(*child_id)?;
                output.place_child(*child_id, PxPosition::ZERO);
            }

            Ok(ComputedData {
                width: Px::new(self.width),
                height: Px::new(self.height),
            })
        }
    }

    #[derive(Clone, Default, PartialEq)]
    struct VerticalStackPolicy;

    impl LayoutPolicy for VerticalStackPolicy {
        fn measure(
            &self,
            input: &LayoutInput<'_>,
            output: &mut LayoutOutput<'_>,
        ) -> Result<ComputedData, crate::MeasurementError> {
            let mut current_y = Px::ZERO;
            let mut max_width = Px::ZERO;

            for child_id in input.children_ids() {
                let child = input.measure_child_in_parent_constraint(*child_id)?;
                output.place_child(*child_id, PxPosition::new(Px::ZERO, current_y));
                current_y += child.height;
                max_width = max_width.max(child.width);
            }

            Ok(ComputedData {
                width: max_width,
                height: current_y,
            })
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

    #[tessera(crate)]
    fn tagged_box(tag: String, width: i32, height: i32) {
        crate::layout::layout_primitive()
            .layout_policy(FixedSizePolicy { width, height })
            .render_policy(NoopRenderPolicy)
            .modifier(Modifier::new().push_semantics(TestTagSemanticsModifier { tag }));
    }

    #[tessera(crate)]
    fn stack_content() {
        crate::layout::layout_primitive()
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
        crate::layout::layout_primitive()
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

    #[test]
    fn assert_layout_macro_smoke() {
        crate::assert_layout! {
            viewport: (800, 600),
            content: {
                sample_layout();
            },
            expect: {
                node("root").size(800, 600).position(0, 0);
                node("title_box").x(0).y(0).size(200, 40);
                node("button_box").below("title_box").align_start_with("title_box").size(120, 32);
            }
        }
    }
}
