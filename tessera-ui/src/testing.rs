//! Headless UI testing for Tessera.
//!
//! ## Usage
//!
//! Drive components frame by frame and assert layout, semantics, input, and pixels.

use std::{collections::BTreeSet, time::Duration};

use crate::{
    Px, PxPosition, PxRect, PxSize,
    accesskit::{Action, ActionRequest, Node, NodeId, TreeId},
    build_tree::build_component_tree,
    component_tree::{ComputeMode, ComputeParams, clear_layout_snapshots},
    context::reset_context_read_dependencies,
    cursor::PointerChange,
    focus::flush_pending_focus_callbacks,
    runtime::{
        ModelScope, TesseraRuntime, begin_frame_clock, clear_persistent_focus_handles,
        clear_redraw_waker, release_retired_replay_snapshots, reset_build_invalidations,
        reset_focus_read_dependencies, reset_frame_clock, reset_layout_dirty_tracking,
        reset_render_slot_read_dependencies, reset_state_read_dependencies,
        take_layout_dirty_nodes, tick_frame_clock,
    },
    time::Instant,
};

pub use crate::{
    component_tree::LayoutFrameDiagnostics,
    work::{FrameWork, WorkExecution, WorkPhase, WorkTotals},
};

mod probe;
mod query;

pub use probe::TestProbe;
pub use query::{
    AccessibilitySnapshot, ComponentSnapshot, ComponentSnapshotNode, text_matches, visible_text,
};

/// A deterministic UI test with session-owned state and a virtual clock.
///
/// Tests start layout-only with an 800 by 600 viewport at the current density.
/// Configure the viewport, density, frame step, and optional GPU rendering
/// before [`UiTest::content`], then step frames and assert the results.
///
/// ```
/// use tessera_ui::testing::UiTest;
///
/// let layout = UiTest::new().viewport(200, 100).content(|| {}).frame();
/// layout.node("root").rect(0, 0, 200, 100);
/// ```
pub struct UiTest<'a> {
    owner: ModelScope,
    #[cfg(all(feature = "headless", not(target_family = "wasm")))]
    renderer: Option<Box<crate::headless::HeadlessRenderer>>,
    content: Box<dyn Fn() + 'a>,
    session: crate::RuntimeSession,
    viewport: (u32, u32),
    samples: u32,
    #[cfg(all(feature = "headless", not(target_family = "wasm")))]
    format: wgpu::TextureFormat,
    frame_step: Duration,
    frame_origin: Instant,
    time: Duration,
    cursor: crate::cursor::CursorState,
    keyboard: crate::keyboard_state::KeyboardState,
    ime: crate::ImeState,
    diagnostics: LayoutFrameDiagnostics,
    work: crate::work::FrameWork,
    work_totals: crate::work::WorkTotals,
    frame_clock_diagnostics: crate::runtime::FrameClockDiagnostics,
}

impl<'a> UiTest<'a> {
    /// Creates a layout-only test.
    pub fn new() -> Self {
        let scale_factor = crate::dp::scale_factor();
        let session = crate::RuntimeSession::for_root();
        session.activate();
        crate::dp::set_scale_factor(scale_factor);
        reset_runtime_for_layout_test((800, 600));
        Self {
            owner: ModelScope::new(),
            #[cfg(all(feature = "headless", not(target_family = "wasm")))]
            renderer: None,
            content: Box::new(|| {}),
            session,
            viewport: (800, 600),
            samples: 1,
            #[cfg(all(feature = "headless", not(target_family = "wasm")))]
            format: crate::headless::HeadlessConfig::default().format,
            frame_step: Duration::from_nanos(16_666_667),
            frame_origin: Instant::now(),
            time: Duration::ZERO,
            cursor: Default::default(),
            keyboard: Default::default(),
            ime: Default::default(),
            diagnostics: Default::default(),
            work: Default::default(),
            work_totals: Default::default(),
            frame_clock_diagnostics: Default::default(),
        }
    }

    /// Sets the viewport in physical pixels.
    pub fn viewport(mut self, width: u32, height: u32) -> Self {
        self.resize(width, height);
        self
    }

    /// Sets the physical pixels per density-independent pixel.
    pub fn scale_factor(self, factor: f64) -> Self {
        assert!(
            factor.is_finite() && factor > 0.0,
            "test density must be finite and positive"
        );
        self.run(|| crate::dp::set_scale_factor(factor));
        self
    }

    /// Sets the MSAA sample count used by a later [`UiTest::rendered`] call.
    pub fn samples(mut self, samples: u32) -> Self {
        self.samples = samples;
        self
    }

    /// Sets the output texture format used by a later [`UiTest::rendered`] call.
    #[cfg(all(feature = "headless", not(target_family = "wasm")))]
    pub fn format(mut self, format: wgpu::TextureFormat) -> Self {
        self.format = format;
        self
    }

    /// Sets the virtual time advanced by [`UiTest::frames`]; defaults to 60 Hz.
    pub fn frame_step(mut self, step: Duration) -> Self {
        self.frame_step = step;
        self
    }

    /// Renders frames on the GPU with the given pipelines.
    ///
    /// Call after [`UiTest::samples`] and [`UiTest::format`]. Panics when no adapter is available.
    #[cfg(all(feature = "headless", not(target_family = "wasm")))]
    pub fn rendered(mut self, module: impl crate::render::RenderModule + 'static) -> Self {
        let config = crate::headless::HeadlessConfig {
            width: self.viewport.0,
            height: self.viewport.1,
            scale_factor: self.run(crate::dp::scale_factor),
            sample_count: self.samples,
            format: self.format,
            ..Default::default()
        };
        let renderer = crate::headless::HeadlessRenderer::new_in_session(
            self.session.clone(),
            Box::new(|| {}),
            vec![Box::new(module)],
            config,
        )
        .expect("UiTest rendered backend initialization failed");
        self.frame_origin = renderer.frame_origin;
        self.renderer = Some(Box::new(renderer));
        self
    }

    /// Sets the UI built on each frame.
    pub fn content<F, Output>(mut self, content: F) -> Self
    where
        F: Fn() -> Output + 'a,
    {
        self.content = Box::new(move || {
            content();
        });
        self
    }

    /// Creates state whose lifetime is owned by this test.
    pub fn state<T: 'static>(&self, value: T) -> crate::State<T> {
        self.run(|| self.owner.own_state(value))
    }

    /// Runs work in this test's UI session.
    pub fn run<R>(&self, work: impl FnOnce() -> R) -> R {
        self.session.run(work)
    }

    /// Returns this test's runtime identity.
    pub fn session_id(&self) -> crate::RuntimeSessionId {
        self.session.identity()
    }

    /// Runs one frame at the current test time and captures its layout.
    pub fn frame(&mut self) -> LayoutSnapshot {
        self.step();
        self.run(LayoutSnapshot::capture)
    }

    /// Advances the clock by `duration` and runs one frame.
    pub fn advance(&mut self, duration: Duration) -> LayoutSnapshot {
        self.time += duration;
        self.frame()
    }

    /// Runs `count` frames, advancing the clock by the frame step before each.
    pub fn frames(&mut self, count: u32) -> LayoutSnapshot {
        for _ in 0..count {
            self.time += self.frame_step;
            self.step();
        }
        self.run(LayoutSnapshot::capture)
    }

    /// Returns the elapsed virtual time.
    pub fn frame_time(&self) -> Duration {
        self.time
    }

    /// Runs frames until no frame task or invalidated UI work remains.
    ///
    /// Panics if the UI has not settled after 1,000 frames.
    pub fn settle(&mut self) {
        self.step();
        for _ in 0..1_000 {
            let idle = self.run(|| {
                crate::runtime::frame_clock_diagnostics().waiting_tasks == 0
                    && !crate::runtime::has_pending_build_invalidations()
                    && !crate::runtime::has_pending_layout_invalidations()
            });
            if idle {
                return;
            }
            self.time += self.frame_step;
            self.step();
        }
        panic!("UiTest did not settle after 1,000 frames; frame tasks or UI invalidations remain");
    }

    /// Selects a node by test tag, component name, or semantic label.
    ///
    /// The returned chain can also [`click`](NodeAssert::click) the node.
    pub fn node(&mut self, selector: &str) -> NodeAssert<'_> {
        let snapshot = self.run(LayoutSnapshot::capture);
        let _ = snapshot.resolve(selector);
        NodeAssert {
            snapshot: std::borrow::Cow::Owned(snapshot),
            selector: selector.into(),
            click: Some(Box::new(move |position| self.click_at(position))),
        }
    }

    /// Presses and releases the mouse at `position`, running a frame after each.
    pub fn click_at(&mut self, position: PxPosition) {
        let button = crate::PressKeyEventType::Left;
        self.pointer_press(crate::MOUSE_POINTER_ID, position, button);
        self.frames(1);
        self.pointer_release(crate::MOUSE_POINTER_ID, position, button);
        self.frames(1);
    }

    /// Finds the first published accessibility node whose label matches `label`.
    pub fn accessibility_node(&self, label: &str) -> (NodeId, Node) {
        self.accessibility_snapshot()
            .expect("accessibility tree")
            .nodes
            .into_iter()
            .find(|(_, node)| text_matches(node.label(), label))
            .unwrap_or_else(|| panic!("missing accessibility node {label:?}"))
    }

    /// Queues pointer input for the next frame.
    pub fn inject_pointer(&mut self, change: PointerChange) {
        if let Some(position) = change.position_snapshot() {
            self.cursor
                .update_pointer_position(change.pointer_id, position);
        }
        self.cursor.push_event(change);
    }

    fn pointer_change(
        &mut self,
        pointer: crate::PointerId,
        position: PxPosition,
        content: crate::CursorEventContent,
    ) {
        self.cursor.update_pointer_position(pointer, position);
        self.cursor.push_event(PointerChange::new(
            self.frame_origin + self.time,
            pointer,
            content,
            crate::GestureState::TapCandidate,
        ));
    }

    /// Moves a pointer to a window position.
    pub fn pointer_move(&mut self, pointer: crate::PointerId, position: PxPosition) {
        self.pointer_change(
            pointer,
            position,
            crate::CursorEventContent::Moved(position),
        );
    }

    /// Presses a pointer button at a window position.
    pub fn pointer_press(
        &mut self,
        pointer: crate::PointerId,
        position: PxPosition,
        button: crate::PressKeyEventType,
    ) {
        self.pointer_change(
            pointer,
            position,
            crate::CursorEventContent::Pressed(button),
        );
    }

    /// Releases a pointer button at a window position.
    pub fn pointer_release(
        &mut self,
        pointer: crate::PointerId,
        position: PxPosition,
        button: crate::PressKeyEventType,
    ) {
        self.pointer_change(
            pointer,
            position,
            crate::CursorEventContent::Released(button),
        );
    }

    /// Queues a mouse-wheel scroll in physical pixels.
    pub fn pointer_scroll(
        &mut self,
        pointer: crate::PointerId,
        position: PxPosition,
        delta_x: f32,
        delta_y: f32,
    ) {
        self.pointer_change(
            pointer,
            position,
            crate::CursorEventContent::Scroll(crate::ScrollEventContent {
                delta_x,
                delta_y,
                unit: crate::ScrollDeltaUnit::Pixel,
                source: crate::ScrollEventSource::Wheel,
            }),
        );
    }

    /// Starts a touch contact.
    pub fn touch_start(&mut self, touch: u64, position: PxPosition) {
        let _session = self.session.enter();
        self.cursor.handle_touch_start(touch, position);
    }

    /// Moves a touch contact, scrolling after touch slop is crossed.
    pub fn touch_move(&mut self, touch: u64, position: PxPosition) {
        let _session = self.session.enter();
        if let Some(scroll) = self.cursor.handle_touch_move(touch, position) {
            self.cursor.push_event(scroll);
        }
    }

    /// Ends a touch contact.
    pub fn touch_end(&mut self, touch: u64) {
        let _session = self.session.enter();
        self.cursor.handle_touch_end(touch);
    }

    /// Cancels a touch contact without producing a tap or fling.
    pub fn touch_cancel(&mut self, touch: u64) {
        let _session = self.session.enter();
        self.cursor.handle_touch_cancel(touch);
    }

    /// Queues an owned or native keyboard sample.
    pub fn inject_key_event(&mut self, event: impl Into<crate::KeyboardEvent>) {
        let _session = self.session.enter();
        let event = event.into();
        TesseraRuntime::with_mut(|runtime| {
            runtime
                .component_tree
                .focus_owner_mut()
                .record_keyboard_input(event.state, self.frame_origin + self.time);
        });
        self.keyboard.push_event(event);
    }

    /// Queues a named key press.
    pub fn press_key(&mut self, key: winit::keyboard::NamedKey) {
        self.inject_key_event(crate::KeyboardEvent::named(
            key,
            winit::event::ElementState::Pressed,
        ));
    }

    /// Sets modifiers captured by subsequent keyboard samples.
    pub fn set_key_modifiers(&mut self, modifiers: winit::keyboard::ModifiersState) {
        self.keyboard.update_modifiers(modifiers);
    }

    /// Queues an IME event.
    pub fn inject_ime_event(&mut self, event: winit::event::Ime) {
        let _session = self.session.enter();
        self.ime.push_event_for(None, event);
    }

    /// Commits text through the focused editor's IME path.
    pub fn inject_text(&mut self, text: &str) {
        self.inject_ime_event(winit::event::Ime::Commit(text.into()));
    }

    /// Delivers a system back event.
    pub fn dispatch_system_back(&self, event: crate::system_back::SystemBackEvent) {
        self.run(|| crate::system_back::dispatch(event));
    }

    /// Captures component geometry and semantic metadata.
    pub fn inspect_components(&self) -> ComponentSnapshot {
        self.run(ComponentSnapshot::capture)
    }

    /// Captures the complete published accessibility tree.
    pub fn inspect_accessibility(&self) -> Option<AccessibilitySnapshot> {
        self.accessibility_snapshot()
            .map(|tree| AccessibilitySnapshot::new(tree, self.session_id()))
    }

    /// Returns the complete AccessKit publication.
    pub fn accessibility_snapshot(&self) -> Option<crate::accesskit::TreeUpdate> {
        self.run(|| {
            TesseraRuntime::with_mut(|runtime| {
                runtime.component_tree.accessibility_update(None, true)
            })
        })
    }

    /// Dispatches an assistive-technology action to a published node.
    ///
    /// Returns whether a focus target or action handler received it. Run
    /// another frame to publish resulting state and focus changes.
    pub fn accessibility_action(&mut self, target: NodeId, action: Action) -> bool {
        let request = ActionRequest {
            action,
            target_tree: TreeId::ROOT,
            target_node: target,
            data: None,
        };
        let timestamp = self.frame_origin + self.time;
        self.run(|| {
            let handled = TesseraRuntime::with_mut(|runtime| {
                let (tree, metadatas, focus_owner) =
                    runtime.component_tree.accessibility_dispatch_context();
                crate::accessibility::dispatch_action(
                    tree,
                    metadatas,
                    focus_owner,
                    request,
                    timestamp,
                )
            });
            flush_pending_focus_callbacks();
            handled
        })
    }

    /// Returns layout diagnostics from the last frame.
    pub fn diagnostics(&self) -> LayoutFrameDiagnostics {
        self.diagnostics
    }

    /// Returns executed work from the last frame.
    pub fn work(&self) -> &crate::work::FrameWork {
        &self.work
    }

    /// Returns cumulative executed work.
    pub fn work_totals(&self) -> &crate::work::WorkTotals {
        &self.work_totals
    }

    /// Returns frame-clock activity from the last frame.
    pub fn frame_clock_diagnostics(&self) -> &crate::runtime::FrameClockDiagnostics {
        &self.frame_clock_diagnostics
    }

    /// Changes the physical-pixel viewport.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.viewport = (width, height);
        self.run(|| TesseraRuntime::with_mut(|runtime| runtime.window_size = [width, height]));
        #[cfg(all(feature = "headless", not(target_family = "wasm")))]
        if let Some(renderer) = &mut self.renderer {
            renderer.resize(width, height);
        }
    }

    /// Returns the physical-pixel viewport.
    pub fn physical_size(&self) -> winit::dpi::PhysicalSize<u32> {
        winit::dpi::PhysicalSize::new(self.viewport.0, self.viewport.1)
    }

    #[cfg(all(feature = "headless", not(target_family = "wasm")))]
    fn renderer(&self) -> &crate::headless::HeadlessRenderer {
        self.renderer
            .as_deref()
            .expect("this operation requires UiTest::rendered")
    }

    /// Reads tightly packed RGBA8 pixels from the rendered frame.
    #[cfg(all(feature = "headless", not(target_family = "wasm")))]
    pub fn read_pixels(&self) -> Vec<u8> {
        self.renderer()
            .read_pixels()
            .expect("UiTest pixel readback failed")
    }

    /// Reads one RGBA8 pixel from the rendered frame.
    #[cfg(all(feature = "headless", not(target_family = "wasm")))]
    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let offset = ((y * self.viewport.0 + x) * 4) as usize;
        self.read_pixels()[offset..offset + 4]
            .try_into()
            .expect("RGBA8 pixel")
    }

    /// Captures the rendered component tree as JSON.
    #[cfg(all(feature = "headless", not(target_family = "wasm")))]
    pub fn snapshot(&self) -> serde_json::Value {
        self.renderer().snapshot()
    }

    /// Returns graphics operations from the last rendered frame.
    #[cfg(all(feature = "headless", not(target_family = "wasm")))]
    pub fn frame_graphics_work(&self) -> &crate::render::GraphicsWorkDiagnostics {
        self.renderer().frame_graphics_work()
    }

    /// Returns cumulative graphics operations.
    #[cfg(all(feature = "headless", not(target_family = "wasm")))]
    pub fn graphics_work(&self) -> crate::render::GraphicsWorkDiagnostics {
        self.renderer().graphics_work()
    }

    /// Waits for GPU completion and captures resource usage.
    #[cfg(all(feature = "headless", not(target_family = "wasm")))]
    pub fn resource_snapshot(&self) -> Result<serde_json::Value, wgpu::PollError> {
        self.renderer().resource_snapshot()
    }

    /// Returns the GPU instance for lifetime and allocation observations.
    #[cfg(all(feature = "headless", not(target_family = "wasm")))]
    pub fn gpu_instance(&self) -> &wgpu::Instance {
        self.renderer().gpu_instance()
    }

    fn step(&mut self) {
        #[cfg(all(feature = "headless", not(target_family = "wasm")))]
        if let Some(renderer) = &mut self.renderer {
            renderer.render_test_frame(
                self.time,
                &*self.content,
                &mut self.cursor,
                &mut self.keyboard,
                &mut self.ime,
            );
            self.diagnostics = renderer.diagnostics();
            self.work = renderer.work().clone();
            self.work_totals = renderer.work_totals().clone();
            self.frame_clock_diagnostics = renderer.frame_clock_diagnostics().clone();
            return;
        }
        self.layout_frame();
    }

    fn layout_frame(&mut self) {
        let session = self.session.enter();
        crate::work::begin();
        begin_frame_clock(self.frame_origin + self.time);
        crate::runtime::begin_frame_build_invalidations();
        #[cfg(not(target_family = "wasm"))]
        crate::asset::settle();
        // Match renderer frame order so frame callbacks update state before build.
        tick_frame_clock();
        let _ = build_component_tree(&self.content);

        let screen_size = PxSize::new(
            Px::new(self.viewport.0 as i32),
            Px::new(self.viewport.1 as i32),
        );

        TesseraRuntime::with_mut(|runtime| runtime.component_tree.begin_frame_diagnostics());
        let mut frame_diagnostics = LayoutFrameDiagnostics::default();
        let cursor_position = self.cursor.position();
        let mut pointer_changes = Some(self.cursor.take_events());
        let mut keyboard_events = Some(self.keyboard.take_events());
        let mut ime_events = Some(self.ime.take_events());
        let modifiers = self.keyboard.modifiers();
        let mut retry_focus_move = None;
        let mut retry_focus_reveal = false;
        for retry in 0..=crate::renderer::MAX_FRAME_SETTLING_RETRIES {
            let layout_dirty_nodes = take_layout_dirty_nodes();
            let (diagnostics, next_move, next_reveal, next_keyboard) =
                TesseraRuntime::with_mut(|runtime| {
                    let (
                        _graph,
                        _requests,
                        diagnostics,
                        _record_cost,
                        next_move,
                        next_reveal,
                        next_keyboard,
                    ) = runtime.component_tree.compute(
                        ComputeParams {
                            screen_size,
                            cursor_position,
                            pointer_changes: pointer_changes.take().unwrap_or_default(),
                            keyboard_events: keyboard_events.take().unwrap_or_default(),
                            ime_events: ime_events.take().unwrap_or_default(),
                            retry_focus_move,
                            retry_focus_reveal,
                            modifiers,
                            layout_dirty_nodes,
                        },
                        ComputeMode::LayoutOnly,
                    );
                    (diagnostics, next_move, next_reveal, next_keyboard)
                });
            frame_diagnostics.accumulate_pass(diagnostics);
            flush_pending_focus_callbacks();
            crate::component_tree::flush_pending_globally_positioned_callbacks();
            let next_keyboard = next_keyboard
                || TesseraRuntime::with(|runtime| {
                    runtime.component_tree.has_pending_focus_reveal()
                });
            let focus_retry = next_move.is_some() || next_reveal || next_keyboard;
            if !focus_retry && !crate::renderer::frame_settling_can_progress(false) {
                break;
            }
            if retry == crate::renderer::MAX_FRAME_SETTLING_RETRIES
                || (focus_retry && !crate::renderer::frame_settling_can_progress(next_keyboard))
            {
                if next_reveal
                    && (crate::runtime::has_pending_build_invalidations()
                        || crate::runtime::has_pending_layout_invalidations())
                {
                    TesseraRuntime::with_mut(|runtime| {
                        runtime
                            .component_tree
                            .focus_owner_mut()
                            .retain_reveal_retry();
                    });
                }
                break;
            }
            retry_focus_move = next_move;
            retry_focus_reveal = next_reveal;
            if focus_retry || crate::runtime::has_ready_build_invalidations() {
                let _ = build_component_tree(&self.content);
            }
        }

        self.diagnostics = frame_diagnostics;
        self.work = crate::work::finish();
        self.work_totals.record(&self.work);
        self.frame_clock_diagnostics = crate::runtime::frame_clock_diagnostics();
        #[cfg(not(target_family = "wasm"))]
        if crate::asset::in_flight() {
            drop(session);
            return self.layout_frame();
        }
        self.cursor.frame_cleanup();
    }
}

impl Default for UiTest<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Captured layout information for a single frame.
#[derive(Clone)]
pub struct LayoutSnapshot {
    components: ComponentSnapshot,
}

impl LayoutSnapshot {
    fn capture() -> Self {
        Self {
            components: ComponentSnapshot::capture(),
        }
    }

    fn placed_nodes(&self) -> impl Iterator<Item = &ComponentSnapshotNode> {
        self.components
            .nodes()
            .iter()
            .filter(|node| node.layout_coordinates().is_some())
    }

    /// Starts an assertion chain for a node.
    ///
    /// `root` selects the root node. Other selectors match a test tag, then a
    /// component function name, then a semantic label.
    pub fn node(&self, selector: &str) -> NodeAssert<'_> {
        let _ = self.resolve(selector);
        NodeAssert {
            snapshot: std::borrow::Cow::Borrowed(self),
            selector: selector.to_string(),
            click: None,
        }
    }

    fn resolve(&self, selector: &str) -> &ComponentSnapshotNode {
        if selector == "root" {
            let root = &self.components.nodes()[0];
            assert!(
                root.layout_coordinates().is_some(),
                "root node not placed after layout"
            );
            return root;
        }
        let mut tagged = self.placed_nodes().filter(|node| {
            node.semantics().and_then(|value| value.key.as_deref()) == Some(selector)
        });
        if let Some(node) = tagged.next() {
            assert!(
                tagged.next().is_none(),
                "duplicate layout test selector `{selector}`"
            );
            return node;
        }
        let mut named = self.placed_nodes().filter(|node| node.name() == selector);
        if let Some(node) = named.next() {
            let count = 1 + named.count();
            assert_eq!(
                count, 1,
                "layout selector `{selector}` matched {count} nodes by function name; add a test_tag to disambiguate"
            );
            return node;
        }
        let mut labeled = self.placed_nodes().filter(|node| {
            text_matches(
                node.semantics().and_then(|value| value.label.as_deref()),
                selector,
            )
        });
        if let Some(node) = labeled.next() {
            assert!(
                labeled.next().is_none(),
                "ambiguous semantic label `{selector}`"
            );
            return node;
        }
        let selectors: BTreeSet<_> = self
            .placed_nodes()
            .flat_map(|node| {
                [
                    Some(node.name()),
                    node.semantics().and_then(|value| value.key.as_deref()),
                ]
                .into_iter()
                .flatten()
            })
            .collect();
        panic!(
            "layout test selector `{selector}` not found; available selectors: {:?}",
            selectors
        )
    }
}

impl ComponentSnapshotNode {
    fn position(&self) -> PxPosition {
        self.layout_coordinates()
            .expect("placed layout node")
            .local_to_window
            .translation()
    }

    fn size(&self) -> PxSize {
        self.layout_coordinates()
            .expect("placed layout node")
            .local_size
    }

    fn x(&self) -> i32 {
        self.position().x_px().raw()
    }

    fn y(&self) -> i32 {
        self.position().y_px().raw()
    }

    fn width(&self) -> i32 {
        self.size().width.raw()
    }

    fn height(&self) -> i32 {
        self.size().height.raw()
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
    snapshot: std::borrow::Cow<'a, LayoutSnapshot>,
    selector: String,
    click: Option<Box<dyn FnOnce(PxPosition) + 'a>>,
}

impl<'a> NodeAssert<'a> {
    /// Assert that the node exists.
    pub fn exists(self) -> Self {
        self
    }

    /// Clicks the center of a node selected through [`UiTest::node`].
    pub fn click(mut self) {
        let coordinates = self.current().layout_coordinates().expect("placed node");
        let position = coordinates
            .local_to_window
            .map_point(PxPosition::from_f32_xy(
                coordinates.local_size.width.raw() as f32 / 2.0,
                coordinates.local_size.height.raw() as f32 / 2.0,
            ));
        self.click
            .take()
            .expect("select an interactive node with UiTest::node")(position);
    }

    /// Returns the node's bounds in physical pixels for geometry comparisons.
    pub fn bounds(&self) -> PxRect {
        let node = self.current();
        PxRect::from_position_size(node.position(), node.size())
    }

    /// Assert the node position in physical pixels.
    pub fn position(self, x: i32, y: i32) -> Self {
        let node = self.current();
        assert_eq!(
            (node.x(), node.y()),
            (x, y),
            "layout node `{}` ({}) expected position ({x}, {y}), got ({}, {})",
            self.selector,
            node.name(),
            node.x(),
            node.y()
        );
        self
    }

    /// Assert the node position and size in physical pixels.
    pub fn rect(self, x: i32, y: i32, width: i32, height: i32) -> Self {
        self.position(x, y).size(width, height)
    }

    /// Assert the node size in physical pixels.
    pub fn size(self, width: i32, height: i32) -> Self {
        let node = self.current();
        assert_eq!(
            (node.width(), node.height()),
            (width, height),
            "layout node `{}` ({}) expected size ({width}, {height}), got ({}, {})",
            self.selector,
            node.name(),
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
            node.name(),
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
            node.name(),
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
            node.name(),
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
            node.name(),
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
            node.name(),
            other_node.name(),
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
            node.name(),
            other_node.name(),
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
            node.name(),
            other_node.name(),
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
            node.name(),
            other_node.name(),
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
            node.name(),
            other_node.name(),
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
            node.name(),
            other_node.name(),
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
            node.name(),
            other_node.name(),
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
            node.name(),
            other_node.name(),
            node.height(),
            other_node.height()
        );
        self
    }

    fn current(&self) -> &ComponentSnapshotNode {
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
    release_retired_replay_snapshots();
    reset_focus_read_dependencies();
    reset_render_slot_read_dependencies();
    reset_state_read_dependencies();
    reset_context_read_dependencies();
    reset_build_invalidations();
    reset_frame_clock();
    clear_redraw_waker();
    clear_persistent_focus_handles();
    crate::runtime::reset_component_slots();
}

#[cfg(test)]
mod tests {
    use std::{
        num::NonZero,
        sync::atomic::{AtomicBool, Ordering},
        time::Duration,
    };

    use crate::{
        AccessibilityActionHandler, AccessibilityNode, AxisConstraint, ComputedData, Constraint,
        LayoutModifierChild, LayoutModifierInput, LayoutPolicy, LayoutResult, Modifier,
        NoopDrawModifier, PlacementModifierNode, Px, PxPosition, RenderSlot, SemanticsModifierNode,
        State, StatefulLayoutPolicy, component_tree::NodeRole, launch_once, layout::MeasureScope,
        next_frame, remember, runtime::TesseraRuntime, tessera,
    };

    use super::{UiTest, reset_runtime_for_layout_test};

    #[derive(Clone, Default, PartialEq)]
    struct MeasureProbe {
        width: super::TestProbe<Option<State<i32>>>,
        visible: super::TestProbe<Option<State<bool>>>,
    }

    #[derive(Clone, PartialEq)]
    struct MeasureOnlyWidthPolicy {
        width: State<i32>,
    }

    impl LayoutPolicy for MeasureOnlyWidthPolicy {
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

    #[derive(Clone, PartialEq)]
    struct MeasureRevisionPolicy(State<u8>);

    impl LayoutPolicy for MeasureRevisionPolicy {
        fn measure(
            &self,
            _: &mut MeasureScope<'_>,
        ) -> Result<LayoutResult, crate::MeasurementError> {
            let _revision = self.0.get();
            Ok(LayoutResult::new(ComputedData {
                width: Px(20),
                height: Px(10),
            }))
        }
    }

    #[tessera]
    fn transparent_dirty_siblings(revision: State<u8>) {
        let stable = remember(|| 0_u8);
        for (tag, revision) in [("left", revision), ("right", revision), ("stable", stable)] {
            crate::layout::layout(MeasureRevisionPolicy(revision)).modifier(
                Modifier::new().push_semantics(TestTagSemanticsModifier { tag: tag.into() }),
            );
        }
    }

    #[tessera]
    fn dirty_siblings_fixture(probe: super::TestProbe<Option<State<u8>>>) {
        let revision = remember(|| 0_u8);
        probe.set(Some(revision));
        crate::layout::layout(VerticalStackPolicy)
            .child(move || transparent_dirty_siblings(revision));
    }

    #[test]
    fn dirty_siblings_share_transparent_ancestors_without_losing_measurement() {
        let probe = super::TestProbe::default();
        let mut session = UiTest::new()
            .content(|| dirty_siblings_fixture(probe.clone()))
            .viewport(100, 100);
        session.frame();
        let snapshot = session.inspect_components();
        let instances = ["left", "right", "stable"].map(|tag| snapshot.tagged(tag).instance_id());
        probe.get().expect("test fixture entry exists").set(1);
        session.advance(Duration::from_millis(16));
        assert_eq!(
            instances.map(|instance| session
                .work()
                .for_instance(instance, crate::work::WorkPhase::Measure)),
            [1, 1, 0],
        );
    }

    #[tessera]
    fn measure_only_width_fixture(probe: MeasureProbe) {
        let width = remember(|| 40);
        probe.width.set(Some(width));
        crate::layout::layout(MeasureOnlyWidthPolicy { width }).modifier(
            Modifier::new().push_semantics(TestTagSemanticsModifier {
                tag: "measure-only".into(),
            }),
        );
    }

    #[test]
    fn layout_state_updates_apply_when_component_build_is_skipped() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let probe = MeasureProbe::default();
        let mut session = UiTest::new()
            .content(|| measure_only_width_fixture(probe.clone()))
            .viewport(200, 100);
        session.frame().node("measure-only").width(40);
        let width = probe.width.get().expect("width state");
        width.set(80);
        assert!(!crate::runtime::has_pending_build_invalidations());
        session.frames(1).node("measure-only").width(80);
        width.set(120);
        session.frames(1).node("measure-only").width(120);
    }

    #[tessera]
    fn layout_dependent_build_fixture(probe: MeasureProbe) {
        let width = remember(|| 40);
        let observed_width = remember(|| 0);
        probe.width.set(Some(width));
        // Rebuilds this fixture on every frame.
        let frame = remember(|| 0_u64);
        let _ = frame.get();
        launch_once(async move {
            loop {
                frame.set(next_frame().await);
            }
        });
        crate::layout::layout(MeasureOnlyWidthPolicy { width }).modifier(
            Modifier::new().on_layout_size_changed(move |size: crate::PxSize| {
                if observed_width.get() != size.width.0 {
                    observed_width.set(size.width.0);
                }
            }),
        );
        crate::layout::layout(FixedSizePolicy {
            width: observed_width.get(),
            height: 20,
        })
        .modifier(Modifier::new().push_semantics(TestTagSemanticsModifier {
            tag: "layout-dependent-build".into(),
        }));
    }

    #[test]
    fn layout_observer_build_updates_settle_before_the_frame_finishes() {
        let probe = MeasureProbe::default();
        let mut session = UiTest::new()
            .content(|| layout_dependent_build_fixture(probe.clone()))
            .viewport(200, 100);
        session.frame().node("layout-dependent-build").width(40);
        let width = probe.width.get().expect("test fixture entry exists");
        session.run(|| width.set(80));
        session.frames(1).node("layout-dependent-build").width(80);
        assert_eq!(session.frame_clock_diagnostics().resumed_tasks, 1);
        assert!(!session.run(|| crate::renderer::frame_settling_can_progress(false)));
        session.frames(1).node("layout-dependent-build").width(80);
        assert_eq!(session.frame_clock_diagnostics().resumed_tasks, 1);
        assert!(!session.run(|| crate::renderer::frame_settling_can_progress(false)));
    }

    #[tessera]
    fn layout_feedback_fixture(probe: super::TestProbe<Option<State<i32>>>) {
        let width = remember(|| 1);
        probe.set(Some(width));
        crate::layout::layout(FixedSizePolicy {
            width: width.get(),
            height: 20,
        })
        .modifier(
            Modifier::new()
                .on_layout_size_changed(move |size: crate::PxSize| width.set(size.width.0 + 1)),
        );
    }

    #[test]
    fn nonconverging_layout_build_feedback_yields_at_the_frame_retry_limit() {
        let probe = super::TestProbe::default();
        let mut session = UiTest::new()
            .content(|| layout_feedback_fixture(probe.clone()))
            .viewport(200, 100);
        session.frame();
        let width = probe.get().expect("test fixture entry exists");
        assert_eq!(
            session.run(|| width.get()),
            crate::renderer::MAX_FRAME_SETTLING_RETRIES as i32 + 2
        );
        assert!(session.run(crate::runtime::has_pending_build_invalidations));
    }

    #[derive(Clone, PartialEq)]
    struct RetainedChildPolicy {
        visible: Option<State<bool>>,
    }

    impl LayoutPolicy for RetainedChildPolicy {
        fn measure(
            &self,
            scope: &mut MeasureScope<'_>,
        ) -> Result<LayoutResult, crate::MeasurementError> {
            let mut result = LayoutResult::new(ComputedData {
                width: Px(200),
                height: Px(60),
            });
            if self.visible.is_none_or(|visible| visible.get()) {
                let child = scope.children()[0];
                child.measure(scope, &Constraint::NONE)?;
                result.place_child(child, PxPosition::ZERO);
            }
            Ok(result)
        }
    }

    #[tessera]
    fn retained_measure_fixture(probe: MeasureProbe) {
        let visible = remember(|| true);
        let width = remember(|| 40);
        probe.visible.set(Some(visible));
        probe.width.set(Some(width));
        crate::layout::layout(RetainedChildPolicy {
            visible: Some(visible),
        })
        .child(move || {
            crate::layout::layout(RetainedChildPolicy { visible: None }).child(move || {
                crate::layout::layout(MeasureOnlyWidthPolicy { width }).modifier(
                    Modifier::new().push_semantics(TestTagSemanticsModifier {
                        tag: "retained-measure".into(),
                    }),
                );
            });
        });
    }

    #[test]
    fn unplaced_subtree_keeps_measurement_invalidations_until_it_is_revealed() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let probe = MeasureProbe::default();
        for update_before_hiding in [false, true] {
            let mut session = UiTest::new()
                .content(|| retained_measure_fixture(probe.clone()))
                .viewport(240, 100);
            session.frame().node("retained-measure").width(40);
            probe
                .visible
                .get()
                .expect("test fixture entry exists")
                .set(false);
            if update_before_hiding {
                probe
                    .width
                    .get()
                    .expect("test fixture entry exists")
                    .set(80);
            }
            session.frames(1);
            if !update_before_hiding {
                probe
                    .width
                    .get()
                    .expect("test fixture entry exists")
                    .set(80);
            }
            session.frames(1);
            session.frames(1);
            assert_eq!(
                session.diagnostics().measure_node_calls,
                1,
                "an invalid unplaced subtree must not schedule repeated hidden measurements"
            );
            probe
                .visible
                .get()
                .expect("test fixture entry exists")
                .set(true);
            session
                .frames(1)
                .node("retained-measure")
                .width(80)
                .position(0, 0);
            session.frames(1);
            assert_eq!(session.diagnostics().measure_node_calls, 1);
        }
    }
    #[derive(Clone, PartialEq)]
    struct FixedSizePolicy {
        width: i32,
        height: i32,
    }

    #[derive(Clone, Default, PartialEq)]
    struct PositionProbe {
        attached: super::TestProbe<Option<State<bool>>>,
        counts: super::TestProbe<Option<State<(u32, u32)>>>,
        width: super::TestProbe<Option<State<i32>>>,
        geometry_counts: super::TestProbe<Option<State<(u32, u32, u32)>>>,
    }

    #[tessera]
    fn position_handler_fixture(probe: PositionProbe) {
        use crate::modifier::FocusModifierExt as _;

        let attached = remember(|| false);
        let counts = remember(|| (0_u32, 0_u32));
        let width = remember(|| 40);
        probe.attached.set(Some(attached));
        probe.counts.set(Some(counts));
        probe.width.set(Some(width));
        let geometry_counts = remember(|| (0_u32, 0_u32, 0_u32));
        probe.geometry_counts.set(Some(geometry_counts));
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
        crate::layout::layout(FixedSizePolicy {
            width: width.get(),
            height: 20,
        })
        .modifier(modifier);
    }

    #[test]
    fn newly_attached_position_handlers_receive_unchanged_bounds_once() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let probe = PositionProbe::default();
        let mut session = UiTest::new()
            .content(|| position_handler_fixture(probe.clone()))
            .viewport(200, 100);
        session.frame();
        let counts = probe.counts.get().expect("counts");
        let attached = probe.attached.get().expect("attachment");
        let geometry_counts = probe.geometry_counts.get().expect("geometry counts");
        assert_eq!(geometry_counts.get(), (0, 0, 0));
        assert_eq!(counts.get(), (1, 0));
        attached.set(true);
        session.frames(1);
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
        session.frames(1);
        assert_eq!(counts.get(), (1, 1), "idle layouts do not repeat callbacks");
        assert_eq!(geometry_counts.get(), (1, 1, 1));
        attached.set(false);
        session.frames(1);
        attached.set(true);
        session.frames(1);
        assert_eq!(
            counts.get(),
            (1, 2),
            "reattachment receives an initial rectangle"
        );
        probe.width.get().expect("width").set(80);
        session.frames(1);
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
        session.frames(1);
        assert_eq!(counts.get(), (2, 3));
        assert_eq!(geometry_counts.get(), (3, 3, 2));
    }

    struct ResourceLease {
        owner: State<u32>,
        drops: super::TestProbe<usize>,
    }

    impl Drop for ResourceLease {
        fn drop(&mut self) {
            self.drops.with_mut(|drops| *drops += 1);
            if self.owner.is_alive() {
                self.owner.with_mut(|value| *value += 1);
            }
        }
    }

    #[derive(Clone, Default, PartialEq)]
    struct ResourceProbe {
        owner: super::TestProbe<Option<State<u32>>>,
        local: super::TestProbe<Option<State<bool>>>,
        child: super::TestProbe<Option<State<bool>>>,
    }

    #[tessera]
    fn remembered_resource_child(owner: State<u32>, drops: super::TestProbe<usize>) {
        let _lease = remember(|| ResourceLease { owner, drops });
        crate::layout::layout(crate::layout::Stack);
    }

    #[tessera]
    fn remembered_resource_fixture(drops: super::TestProbe<usize>, probe: ResourceProbe) {
        let owner = remember(|| 0_u32);
        let local = remember(|| true);
        let child = remember(|| true);
        probe.owner.set(Some(owner));
        probe.local.set(Some(local));
        probe.child.set(Some(child));
        crate::layout::layout(crate::layout::Stack).child(move || {
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
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let drops = super::TestProbe::new(0usize);
        let initial = drops.clone();
        let probe = ResourceProbe::default();
        let mut session = UiTest::new()
            .content(|| remembered_resource_fixture(initial.clone(), probe.clone()))
            .viewport(20, 20);
        session.frame();
        let owner = probe.owner.get().expect("owner");
        probe.local.get().expect("local resource").set(false);
        session.frames(1);
        assert_eq!(
            owner.get(),
            1,
            "conditional remembered resources can update their owner during recycling"
        );
        probe.child.get().expect("child resource").set(false);
        session.frames(1);
        assert_eq!(
            owner.get(),
            2,
            "removed child components release resources outside the slot-table borrow"
        );
        assert_eq!(drops.get(), 2);
        probe.local.get().expect("local resource").set(true);
        probe.child.get().expect("child resource").set(true);
        session.frames(1);
        reset_runtime_for_layout_test((20, 20));
        assert!(!owner.is_alive());
        assert_eq!(
            drops.get(),
            4,
            "runtime reset invalidates owners before resource destruction"
        );

        let thread_drops = drops.clone();
        std::thread::spawn(move || {
            let mut session = UiTest::new()
                .content(|| {
                    remembered_resource_fixture(thread_drops.clone(), ResourceProbe::default())
                })
                .viewport(20, 20);
            session.frame();
        })
        .join()
        .expect("thread-local resource teardown must permit liveness checks");
        assert_eq!(drops.get(), 6);
    }

    impl crate::modifier::LayoutPolicy for FixedSizePolicy {
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

    impl crate::modifier::LayoutPolicy for VerticalStackPolicy {
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

    impl crate::modifier::LayoutPolicy for OffsetChildPolicy {
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

    impl crate::modifier::LayoutModifier for AnimatedWidthModifierNode {
        fn measure(
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

    impl crate::modifier::LayoutModifier for TestPaddingModifierNode {
        fn measure(
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

    #[tessera]
    fn tagged_box(tag: Option<String>, width: Option<i32>, height: Option<i32>) {
        let tag = tag.unwrap_or_default();
        let width = width.unwrap_or_default();
        let height = height.unwrap_or_default();
        crate::layout::layout(FixedSizePolicy { width, height })
            .draw(NoopDrawModifier)
            .modifier(Modifier::new().push_semantics(TestTagSemanticsModifier { tag }));
    }

    #[tessera]
    fn responsive_box(tag: Option<String>) {
        let tag = tag.unwrap_or_default();
        crate::layout::layout(crate::layout::Stack)
            .draw(NoopDrawModifier)
            .modifier(Modifier::new().push_semantics(TestTagSemanticsModifier { tag }));
    }

    #[tessera]
    fn stack_content() {
        crate::layout::layout(VerticalStackPolicy)
            .draw(NoopDrawModifier)
            .modifier(Modifier::new())
            .child(|| {
                title_box();
                button_box();
            });
    }

    #[tessera]
    fn sample_layout() {
        crate::layout::layout(FixedSizePolicy {
            width: 800,
            height: 600,
        })
        .draw(NoopDrawModifier)
        .modifier(Modifier::new())
        .child(|| {
            stack_content();
        });
    }

    #[tessera]
    fn title_box() {
        tagged_box().tag("title".to_string()).width(200).height(40);
    }

    #[tessera]
    fn button_box() {
        tagged_box().tag("button".to_string()).width(120).height(32);
    }

    #[tessera]
    fn animated_layout_sample() {
        let offset = remember(|| 0_i32);

        // Rebuilds this component on every animated frame.
        let frame_clock = crate::remember(crate::current_frame_nanos);
        let _ = frame_clock.get();
        crate::launch_once(async move {
            let step = move |frame_nanos| {
                let next_offset = if frame_nanos >= 200_000_000 {
                    100
                } else if frame_nanos >= 100_000_000 {
                    50
                } else {
                    0
                };
                offset.set(next_offset);
                if frame_nanos >= 200_000_000 {
                    false
                } else {
                    true
                }
            };
            loop {
                let now = crate::next_frame().await;
                frame_clock.set(now);
                if !step(now) {
                    break;
                }
            }
        });

        crate::layout::layout(OffsetChildPolicy { x: offset.get() })
            .draw(NoopDrawModifier)
            .modifier(Modifier::new())
            .child(|| {
                tagged_box().tag("moving".to_string()).width(20).height(20);
            });
    }

    #[tessera]
    fn animated_modifier_layout_sample() {
        let width = remember(|| 20_i32);

        // Rebuilds this component on every animated frame.
        let frame_clock = crate::remember(crate::current_frame_nanos);
        let _ = frame_clock.get();
        crate::launch_once(async move {
            let step = move |frame_nanos| {
                let next_width = if frame_nanos >= 200_000_000 {
                    100
                } else if frame_nanos >= 100_000_000 {
                    50
                } else {
                    20
                };
                width.set(next_width);
                if frame_nanos >= 200_000_000 {
                    false
                } else {
                    true
                }
            };
            loop {
                let now = crate::next_frame().await;
                frame_clock.set(now);
                if !step(now) {
                    break;
                }
            }
        });

        crate::layout::layout(crate::layout::Stack)
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

    #[tessera]
    fn animated_offset_modifier_sample() {
        let x = remember(|| 0_i32);

        // Rebuilds this component on every animated frame.
        let frame_clock = crate::remember(crate::current_frame_nanos);
        let _ = frame_clock.get();
        crate::launch_once(async move {
            let step = move |frame_nanos| {
                let next_x = if frame_nanos >= 200_000_000 {
                    100
                } else if frame_nanos >= 100_000_000 {
                    50
                } else {
                    0
                };
                x.set(next_x);
                if frame_nanos >= 200_000_000 {
                    false
                } else {
                    true
                }
            };
            loop {
                let now = crate::next_frame().await;
                frame_clock.set(now);
                if !step(now) {
                    break;
                }
            }
        });

        crate::layout::layout(FixedSizePolicy {
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

    #[tessera]
    fn slot_host(slot: Option<RenderSlot>) {
        let slot = slot.unwrap_or_default();
        crate::layout::layout(VerticalStackPolicy)
            .draw(NoopDrawModifier)
            .modifier(Modifier::new())
            .child(move || {
                slot.render();
            });
    }

    #[tessera]
    fn nested_prop_slot(width: i32, remaining: usize) {
        if remaining == 0 {
            crate::layout::layout(FixedSizePolicy { width, height: 20 }).modifier(
                Modifier::new().push_semantics(TestTagSemanticsModifier {
                    tag: "deep-slot-value".to_owned(),
                }),
            );
        } else {
            slot_host().slot(move || {
                nested_prop_slot(width, remaining - 1);
            });
        }
    }

    #[tessera]
    fn nested_prop_slot_fixture(probe: super::TestProbe<Option<State<i32>>>) {
        let width = remember(|| 20);
        probe.set(Some(width));
        nested_prop_slot(width.get(), 12);
    }

    #[test]
    fn nested_slot_props_finish_forward_replay_before_layout_in_the_same_frame() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let probe = super::TestProbe::default();
        let mut session = UiTest::new()
            .content(|| nested_prop_slot_fixture(probe.clone()))
            .viewport(200, 100);
        session.frame().node("deep-slot-value").size(20, 20);
        probe.get().expect("test fixture entry exists").set(60);
        session.frames(1).node("deep-slot-value").size(60, 20);
        session.frames(1).node("deep-slot-value").size(60, 20);
        assert_eq!(
            session.diagnostics().measure_node_calls,
            1,
            "Completed slot updates must leave the following idle layout cached"
        );
    }

    #[tessera]
    fn animated_nested_slot_sample() {
        let width = remember(|| 20_i32);

        // Rebuilds this component on every animated frame.
        let frame_clock = crate::remember(crate::current_frame_nanos);
        let _ = frame_clock.get();
        crate::launch_once(async move {
            let step = move |frame_nanos| {
                let next_width = if frame_nanos >= 200_000_000 {
                    100
                } else if frame_nanos >= 100_000_000 {
                    50
                } else {
                    20
                };
                width.set(next_width);
                if frame_nanos >= 200_000_000 {
                    false
                } else {
                    true
                }
            };
            loop {
                let now = crate::next_frame().await;
                frame_clock.set(now);
                if !step(now) {
                    break;
                }
            }
        });

        crate::layout::layout(FixedSizePolicy {
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

    #[tessera]
    fn ordered_layout_modifier_sample() {
        crate::layout::layout(crate::layout::Stack)
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

    #[tessera]
    fn explicit_layout_boundary_sample(width: super::TestProbe<Option<State<i32>>>) {
        let state = remember(|| 24);
        width.set(Some(state));
        crate::layout::layout(FixedSizePolicy {
            width: state.get(),
            height: 12,
        });
    }

    #[test]
    fn ui_test_layout_assertions_smoke() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let layout = UiTest::new().content(sample_layout).frame();
        layout.node("root").rect(0, 0, 800, 600);
        layout.node("title").x(0).y(0).size(200, 40);
        layout
            .node("button")
            .below("title")
            .align_start_with("title")
            .size(120, 32);
    }

    #[derive(Clone, Copy, PartialEq)]
    struct IntrinsicAwarePolicy;

    impl LayoutPolicy for IntrinsicAwarePolicy {
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

    impl LayoutPolicy for IntrinsicProbePolicy {
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

    #[tessera]
    fn intrinsic_probe_fixture(intrinsic_first: bool) {
        crate::layout::layout(IntrinsicProbePolicy { intrinsic_first }).child(|| {
            crate::layout::layout(IntrinsicAwarePolicy).modifier(
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
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        for intrinsic_first in [false, true] {
            let mut session = UiTest::new()
                .content(move || intrinsic_probe_fixture(intrinsic_first))
                .viewport(200, 200);
            session.frame().node("intrinsic-child").rect(7, 9, 38, 42);
            for _ in 0..2 {
                session.frames(1).node("intrinsic-child").rect(7, 9, 38, 42);
            }
            assert!(session.diagnostics().cache_hits_direct > 0);
            assert_eq!(session.diagnostics().cache_store_count, 0);
        }
    }

    #[test]
    fn ui_test_advances_animation_frames() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let mut session = UiTest::new()
            .content(animated_layout_sample)
            .viewport(200, 100);

        let first = session.frame();
        first.node("moving").rect(0, 0, 20, 20);

        let mid = session.advance(Duration::from_millis(100));
        mid.node("moving").rect(50, 0, 20, 20);

        let end = session.advance(Duration::from_millis(100));
        end.node("moving").rect(100, 0, 20, 20);
        assert_eq!(session.frame_time(), Duration::from_millis(200));
    }

    #[test]
    fn ui_test_advances_modifier_driven_animation_frames() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let mut ui = UiTest::new()
            .viewport(200, 100)
            .content(animated_modifier_layout_sample);
        ui.frame().node("modifier_box").size(20, 20);
        ui.advance(Duration::from_millis(100))
            .node("modifier_box")
            .size(50, 20);
        ui.advance(Duration::from_millis(100))
            .node("modifier_box")
            .size(100, 20);
    }

    #[test]
    fn ui_test_advances_offset_modifier_frames() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let mut ui = UiTest::new()
            .viewport(200, 100)
            .content(animated_offset_modifier_sample);
        ui.frame().node("offset_box").rect(0, 0, 20, 20);
        ui.advance(Duration::from_millis(100))
            .node("offset_box")
            .rect(50, 0, 20, 20);
        ui.advance(Duration::from_millis(100))
            .node("offset_box")
            .rect(100, 0, 20, 20);
    }

    #[test]
    fn ui_test_advances_nested_slot_animation_frames() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let mut ui = UiTest::new()
            .viewport(200, 100)
            .content(animated_nested_slot_sample);
        ui.frame().node("nested_moving").size(20, 20);
        ui.advance(Duration::from_millis(100))
            .node("nested_moving")
            .size(50, 20);
        ui.advance(Duration::from_millis(100))
            .node("nested_moving")
            .size(100, 20);
    }

    #[test]
    fn layout_modifier_order_follows_source_chain() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let layout = UiTest::new()
            .viewport(200, 100)
            .content(ordered_layout_modifier_sample)
            .frame();
        layout.node("ordered_parent").rect(0, 0, 70, 40);
        layout.node("ordered_child").rect(10, 10, 50, 20);
    }

    #[test]
    fn layout_replay_updates_size_without_allocating_a_second_node() {
        let width = super::TestProbe::default();
        let mut session = UiTest::new()
            .content({
                let width = width.clone();
                move || {
                    explicit_layout_boundary_sample(width.clone());
                }
            })
            .viewport(100, 100);
        let layout_id = || {
            TesseraRuntime::with(|runtime| {
                let tree = runtime.component_tree.tree();
                let root = tree
                    .get_node_id_at(NonZero::<usize>::MIN)
                    .expect("build root");
                let mut nodes = root
                    .descendants(tree)
                    .filter(|&id| tree[id].get().fn_name == "explicit_layout_boundary_sample");
                let id = nodes.next().expect("layout node");
                assert_eq!(tree[id].get().role, NodeRole::Layout);
                assert!(nodes.next().is_none());
                id
            })
        };

        session
            .frame()
            .node("explicit_layout_boundary_sample")
            .size(24, 12);
        let initial = session.session.run(layout_id);
        width.get().expect("width state").set(36);
        session
            .frames(1)
            .node("explicit_layout_boundary_sample")
            .size(36, 12);
        assert_eq!(session.session.run(layout_id), initial);
    }

    #[derive(Clone, PartialEq)]
    struct IntrinsicStateLeaf {
        width: State<i32>,
    }

    impl LayoutPolicy for IntrinsicStateLeaf {
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

    impl LayoutPolicy for IntrinsicPositionPolicy {
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

    #[tessera]
    fn intrinsic_state_fixture(probe: super::TestProbe<Option<State<i32>>>) {
        let width = remember(|| 40);
        probe.set(Some(width));
        crate::layout::layout(IntrinsicPositionPolicy).child(move || {
            crate::layout::layout(IntrinsicStateLeaf { width }).modifier(
                Modifier::new().push_semantics(TestTagSemanticsModifier {
                    tag: "intrinsic-state".into(),
                }),
            );
        });
    }

    #[test]
    fn intrinsic_state_dependencies_survive_normal_measurement_and_invalidate_the_probe_owner() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let probe = super::TestProbe::default();
        let mut session = UiTest::new()
            .content(|| intrinsic_state_fixture(probe.clone()))
            .viewport(240, 100);
        session.frame().node("intrinsic-state").rect(40, 0, 20, 20);
        let width = probe.get().expect("test fixture entry exists");
        for value in [70, 30] {
            width.set(value);
            assert!(!crate::runtime::has_pending_build_invalidations());
            session
                .frames(1)
                .node("intrinsic-state")
                .size(20, 20)
                .position(value, 0);
        }
    }

    #[derive(Clone, Default, PartialEq)]
    struct PhaseProbe {
        offset: super::TestProbe<Option<State<i32>>>,
        visible: super::TestProbe<Option<State<bool>>>,
    }

    #[derive(Clone, PartialEq)]
    struct PlacementStatePolicy {
        offset: State<i32>,
        transform: bool,
    }

    impl LayoutPolicy for PlacementStatePolicy {
        fn measure(
            &self,
            scope: &mut MeasureScope<'_>,
        ) -> Result<LayoutResult, crate::MeasurementError> {
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

        fn place(
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
            position.offset(Px(self.offset.get()), Px::ZERO)
        }
    }

    #[tessera]
    fn placement_state_fixture(transform: bool, probe: PhaseProbe) {
        let offset = remember(|| 0);
        probe.offset.set(Some(offset));
        let modifier = if transform {
            Modifier::new().push_placement(StateOffsetModifier { offset })
        } else {
            Modifier::new()
        };
        crate::layout::layout(PlacementStatePolicy { offset, transform })
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
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        for transform in [false, true] {
            let probe = PhaseProbe::default();
            let mut session = UiTest::new()
                .content(|| placement_state_fixture(transform, probe.clone()))
                .viewport(240, 100);
            session.frame().node("phase-child").position(0, 0);
            assert!(
                session
                    .work()
                    .executed::<PlacementStatePolicy>(crate::work::WorkPhase::Measure)
                    > 0
            );
            assert!(
                session
                    .work()
                    .count("placement_state_fixture", crate::work::WorkPhase::Build)
                    > 0
            );
            if transform {
                assert!(
                    session
                        .work()
                        .executed::<StateOffsetModifier>(crate::work::WorkPhase::Transform)
                        > 0
                );
            }
            session.frames(1).node("phase-child").position(0, 0);
            assert_eq!(session.diagnostics().geometry_nodes_updated, 0);
            assert_eq!(
                session
                    .work()
                    .executed::<StateOffsetModifier>(crate::work::WorkPhase::Transform),
                0
            );
            probe
                .offset
                .get()
                .expect("test fixture entry exists")
                .set(35);
            assert!(!crate::runtime::has_pending_build_invalidations());
            session.frames(1).node("phase-child").position(35, 0);
            assert_eq!(
                session
                    .work()
                    .executed::<PlacementStatePolicy>(crate::work::WorkPhase::Measure),
                0
            );
            assert_eq!(
                session
                    .work()
                    .count("placement_state_fixture", crate::work::WorkPhase::Build),
                0
            );
            assert!(session.diagnostics().geometry_nodes_updated > 0);
            session.frames(1).node("phase-child").position(35, 0);
            assert_eq!(session.diagnostics().geometry_nodes_updated, 0);
        }
    }

    #[tessera]
    fn retained_phase_fixture(transform: bool, probe: PhaseProbe) {
        let visible = remember(|| true);
        probe.visible.set(Some(visible));
        crate::layout::layout(RetainedChildPolicy {
            visible: Some(visible),
        })
        .child(move || {
            placement_state_fixture(transform, probe.clone());
        });
    }

    #[test]
    fn unplaced_subtree_keeps_placement_and_transform_invalidations_without_remeasuring() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        for transform in [false, true] {
            for update_before_hiding in [false, true] {
                let probe = PhaseProbe::default();
                let mut session = UiTest::new()
                    .content(|| retained_phase_fixture(transform, probe.clone()))
                    .viewport(240, 100);
                session.frame().node("phase-child").position(0, 0);
                assert!(
                    session
                        .work()
                        .executed::<PlacementStatePolicy>(crate::work::WorkPhase::Measure)
                        > 0
                );
                probe
                    .visible
                    .get()
                    .expect("test fixture entry exists")
                    .set(false);
                if update_before_hiding {
                    probe
                        .offset
                        .get()
                        .expect("test fixture entry exists")
                        .set(35);
                }
                session.frames(1);
                assert_eq!(
                    session
                        .work()
                        .executed::<PlacementStatePolicy>(crate::work::WorkPhase::Measure),
                    0
                );
                if !update_before_hiding {
                    probe
                        .offset
                        .get()
                        .expect("test fixture entry exists")
                        .set(35);
                }
                session.frames(1);
                assert_eq!(
                    session
                        .work()
                        .executed::<PlacementStatePolicy>(crate::work::WorkPhase::Measure),
                    0
                );
                session.frames(1);
                assert_eq!(
                    session
                        .work()
                        .executed::<PlacementStatePolicy>(crate::work::WorkPhase::Measure),
                    0
                );
                assert_eq!(session.diagnostics().measure_node_calls, 1);
                probe
                    .visible
                    .get()
                    .expect("test fixture entry exists")
                    .set(true);
                session.frames(1).node("phase-child").position(35, 0);
                assert_eq!(
                    session
                        .work()
                        .executed::<PlacementStatePolicy>(crate::work::WorkPhase::Measure),
                    0,
                    "revealing a placement-only update must retain measured dimensions"
                );
            }
        }
    }

    #[tessera]
    fn inner_layout_cache_sample() {
        crate::layout::layout(crate::layout::Stack);
    }

    #[tessera]
    fn nested_composition_layout_sample() {
        crate::layout::layout(crate::layout::Stack).child(|| inner_layout_cache_sample());
    }

    #[test]
    fn stable_frame_reuses_layout_cache_across_composition_boundaries() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let mut session = UiTest::new()
            .content(|| nested_composition_layout_sample())
            .viewport(120, 80);
        session.frame();
        session.frames(1);

        let diagnostics = session.diagnostics();
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

    #[derive(Default)]
    struct PolicyWithLocalMeasurement(AtomicBool);

    impl Clone for PolicyWithLocalMeasurement {
        fn clone(&self) -> Self {
            Self(AtomicBool::new(self.0.load(Ordering::Relaxed)))
        }
    }

    impl PartialEq for PolicyWithLocalMeasurement {
        fn eq(&self, other: &Self) -> bool {
            self.0.load(Ordering::Relaxed) == other.0.load(Ordering::Relaxed)
        }
    }

    impl LayoutPolicy for PolicyWithLocalMeasurement {
        fn measure(
            &self,
            _: &mut MeasureScope<'_>,
        ) -> Result<LayoutResult, crate::MeasurementError> {
            self.0.store(true, Ordering::Relaxed);
            Ok(LayoutResult::new(ComputedData {
                width: Px(20),
                height: Px(10),
            }))
        }
    }

    #[tessera]
    fn policy_snapshot_fixture(probe: super::TestProbe<Option<State<bool>>>) {
        let renamed = remember(|| false);
        probe.set(Some(renamed));
        let tag = if renamed.get() { "renamed" } else { "original" };
        crate::layout::layout(StatefulLayoutPolicy(PolicyWithLocalMeasurement::default()))
            .modifier(Modifier::new().push_semantics(TestTagSemanticsModifier { tag: tag.into() }));
    }

    #[test]
    fn measurement_local_policy_mutation_does_not_invalidate_a_semantics_only_replay() {
        let probe = super::TestProbe::default();
        let mut session = UiTest::new()
            .content(|| {
                let probe = probe.clone();
                crate::layout::layout(crate::layout::Stack)
                    .child(move || policy_snapshot_fixture(probe.clone()));
            })
            .viewport(100, 100);
        session.frame().node("original").size(20, 10);
        assert!(
            session
                .work()
                .executed::<StatefulLayoutPolicy<PolicyWithLocalMeasurement>>(
                    crate::work::WorkPhase::Measure,
                )
                > 0
        );

        probe.get().expect("test fixture entry exists").set(true);
        session
            .advance(Duration::from_millis(16))
            .node("renamed")
            .size(20, 10);
        assert_eq!(
            session
                .work()
                .executed::<StatefulLayoutPolicy<PolicyWithLocalMeasurement>>(
                    crate::work::WorkPhase::Measure,
                ),
            0
        );
    }

    #[tessera]
    fn registered_handles_layout() {
        crate::layout::layout(VerticalStackPolicy).child(|| {
            let first = tagged_box().tag("declared-first".to_owned());
            let second = tagged_box().tag("declared-second".to_owned());
            if true {
                second.width(20).height(12);
                first.width(10).height(8);
            }
            drop(second);
            let forgotten = tagged_box().tag("forgotten".to_owned()).width(5).height(6);
            std::mem::forget(forgotten);
        });
    }

    #[test]
    fn saved_component_handles_keep_layout_order_through_cross_group_configuration_and_disposal() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let snapshot = UiTest::new()
            .content(registered_handles_layout)
            .viewport(100, 100)
            .frame();
        snapshot.node("declared-first").rect(0, 0, 10, 8);
        snapshot.node("declared-second").rect(0, 8, 20, 12);
        snapshot.node("forgotten").rect(0, 20, 5, 6);
    }

    #[tessera]
    fn closed_slot_host(slot: Option<RenderSlot>) {
        let _ = slot;
    }

    #[test]
    fn a_parent_that_does_not_expand_its_slot_registers_no_child_content() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        #[tessera]
        fn fixture() {
            closed_slot_host().slot(|| panic!("unexpanded slot was invoked"));
            tagged_box().tag("visible".to_owned()).width(10).height(10);
        }
        UiTest::new()
            .content(fixture)
            .frame()
            .node("visible")
            .size(10, 10);
    }

    #[tessera]
    fn interleaved_slot_fixture() {
        crate::layout::layout(VerticalStackPolicy).child(|| {
            tagged_box()
                .tag("before-slot".to_owned())
                .width(10)
                .height(8);
            RenderSlot::new(|| {
                tagged_box()
                    .tag("inside-slot".to_owned())
                    .width(20)
                    .height(12);
            })
            .render();
            tagged_box().tag("after-slot".to_owned()).width(5).height(6);
        });
    }

    #[test]
    fn controlled_slot_expansion_preserves_preceding_and_following_declaration_order() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let snapshot = UiTest::new().content(interleaved_slot_fixture).frame();
        snapshot.node("before-slot").rect(0, 0, 10, 8);
        snapshot.node("inside-slot").rect(0, 8, 20, 12);
        snapshot.node("after-slot").rect(0, 20, 5, 6);
    }

    #[derive(Clone, Default, PartialEq)]
    struct CommittedProbe {
        last: super::TestProbe<Option<TaggedBoxBuilder>>,
        rebuild: super::TestProbe<Option<State<u32>>>,
        rejected: super::TestProbe<usize>,
    }

    fn expect_committed_call(handle: &TaggedBoxBuilder, probe: &CommittedProbe) {
        let error = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            handle.width(99);
        }))
        .expect_err("committed call must reject configuration");
        assert!(
            error
                .downcast_ref::<String>()
                .expect("test fixture provides downcast_ref")
                .contains("is no longer Pending")
        );
        probe.rejected.set(probe.rejected.get() + 1);
    }

    #[tessera]
    fn committed_call_fixture(probe: CommittedProbe) {
        let revision = remember(|| 0);
        probe.rebuild.set(Some(revision));
        let width = 10 + revision.get() as i32;
        let handle = tagged_box()
            .tag("committed-call".to_owned())
            .width(width)
            .height(8);
        if let Some(previous) = probe.last.take() {
            expect_committed_call(&previous, &probe);
        }
        RenderSlot::new(|| {}).render();
        expect_committed_call(&handle, &probe);
        probe.last.set(Some(handle));
    }

    #[test]
    fn committed_handles_fail_within_a_frame_and_cannot_mutate_the_next_builds_use() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let probe = CommittedProbe::default();
        probe.last.set(None);
        probe.rejected.set(0);
        let mut session = UiTest::new().content(|| committed_call_fixture(probe.clone()));
        session.frame().node("committed-call").size(10, 8);
        assert_eq!(probe.rejected.get(), 1);
        probe
            .rebuild
            .get()
            .expect("test fixture entry exists")
            .set(1);
        session.frames(1).node("committed-call").size(11, 8);
        assert_eq!(probe.rejected.get(), 3);
        probe.last.set(None);
    }

    #[derive(Clone, Copy, PartialEq)]
    struct CallTheme(i32);

    #[derive(Clone, Default, PartialEq)]
    struct ContextProbe {
        reversed: super::TestProbe<Option<State<bool>>>,
        owners: super::TestProbe<Vec<(String, State<u32>, RenderSlot)>>,
    }

    #[tessera]
    fn contextual_call(tag: String, child: Option<RenderSlot>, probe: ContextProbe) {
        assert_eq!(
            crate::use_context::<CallTheme>()
                .expect("test fixture provides use_context")
                .get()
                .0,
            17
        );
        let local = remember(|| 0);
        let child = child.expect("test fixture provides child");
        probe.owners.with_mut(|owners| {
            if let Some(previous) = owners.iter_mut().find(|entry| entry.0 == tag) {
                assert!(
                    previous.1 == local,
                    "configuration groups changed state identity"
                );
                assert!(
                    previous.2 == child,
                    "configuration groups changed slot identity"
                );
            } else {
                owners.push((tag, local, child));
            }
        });
        child.render();
    }

    fn configure_contextual_call(handle: &ContextualCallBuilder, tag: &'static str, delta: i32) {
        handle.child(move || {
            let context = crate::use_context::<CallTheme>()
                .expect("test fixture provides use_context")
                .get();
            tagged_box()
                .tag(tag.to_owned())
                .width(context.0 + delta)
                .height(8);
        });
    }

    #[tessera]
    fn contextual_calls_fixture(probe: ContextProbe) {
        let reversed = remember(|| false);
        probe.reversed.set(Some(reversed));
        let reverse = reversed.get();
        crate::layout::layout(VerticalStackPolicy).child(move || {
            let first = crate::provide_context(CallTheme(17), || {
                contextual_call("first".to_owned(), probe.clone())
            });
            let second = crate::provide_context(CallTheme(17), || {
                contextual_call("second".to_owned(), probe.clone())
            });
            crate::provide_context(CallTheme(91), || {
                if reverse {
                    configure_contextual_call(&second, "context-second", 5);
                    configure_contextual_call(&first, "context-first", 3);
                } else {
                    configure_contextual_call(&first, "context-first", 0);
                    configure_contextual_call(&second, "context-second", 1);
                }
            });
        });
    }

    #[test]
    fn delayed_property_slots_restore_declaration_context_and_keep_identity_across_replay() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let probe = ContextProbe::default();
        let mut session = UiTest::new().content(|| contextual_calls_fixture(probe.clone()));
        let snapshot = session.frame();
        snapshot.node("context-first").rect(0, 0, 17, 8);
        snapshot.node("context-second").rect(0, 8, 18, 8);
        probe
            .reversed
            .get()
            .expect("test fixture entry exists")
            .set(true);
        let snapshot = session.frames(1);
        snapshot.node("context-first").rect(0, 0, 20, 8);
        snapshot.node("context-second").rect(0, 8, 22, 8);
        probe.owners.with(|owners| assert_eq!(owners.len(), 2));
    }

    #[tessera]
    fn root_props_call(
        action: crate::Callback,
        child: RenderSlot,
        probe: super::TestProbe<Option<State<i32>>>,
    ) {
        let tick = remember(|| 0);
        probe.set(Some(tick));
        let _ = tick.get();
        action.call();
        child.render();
    }

    #[test]
    fn entry_owned_prop_functors_survive_independent_replays_of_the_root_component() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let calls = super::TestProbe::new(0usize);
        let probe = super::TestProbe::default();
        let observed = calls.clone();
        let mut session = UiTest::new().content({
            let probe = probe.clone();
            move || {
                let calls = observed.clone();
                root_props_call(
                    move || {
                        calls.with_mut(|calls| *calls += 1);
                    },
                    || {
                        tagged_box()
                            .tag("root-prop-child".to_owned())
                            .width(12)
                            .height(8);
                    },
                    probe.clone(),
                );
            }
        });
        session.frame().node("root-prop-child").size(12, 8);
        for value in 1..3 {
            probe.get().expect("test fixture entry exists").set(value);
            session.frames(1).node("root-prop-child").size(12, 8);
        }
        assert_eq!(calls.get(), 3);
    }

    #[tessera]
    fn static_slot_choice_fixture(probe: super::TestProbe<Option<State<bool>>>) {
        let choice = remember(|| false);
        probe.set(Some(choice));
        let host = crate::layout::layout(crate::layout::Stack);
        if choice.get() {
            host.child(|| {
                tagged_box()
                    .tag("static-slot-b".to_owned())
                    .width(20)
                    .height(8);
            });
        } else {
            host.child(|| {
                tagged_box()
                    .tag("static-slot-a".to_owned())
                    .width(10)
                    .height(8);
            });
        }
    }

    #[test]
    fn changing_a_capture_free_slot_body_invalidates_readers_even_when_its_handle_is_stable() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let probe = super::TestProbe::default();
        let mut session = UiTest::new().content(|| static_slot_choice_fixture(probe.clone()));
        session.frame().node("static-slot-a").size(10, 8);
        probe.get().expect("test fixture entry exists").set(true);
        session.frames(1).node("static-slot-b").size(20, 8);
    }

    #[tessera]
    fn required_unit_slot_probe(child: crate::RenderSlotWith<()>) {
        child.render(());
    }

    #[tessera]
    fn unit_slot_leaf() {
        tagged_box()
            .tag("required-unit-child".to_owned())
            .width(9)
            .height(4);
    }

    #[test]
    fn required_unit_render_slot_accepts_a_component_entry_as_its_build_function() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        UiTest::new()
            .content(|| required_unit_slot_probe(unit_slot_leaf))
            .frame()
            .node("required-unit-child")
            .size(9, 4);
    }
}
