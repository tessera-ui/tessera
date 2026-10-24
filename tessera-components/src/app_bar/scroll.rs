//! Nested scrolling for top app bars and their page content.
//!
//! ## Usage
//!
//! Coordinate a screen header with scrolling lists and feeds.

use tessera_ui::{
    CallbackWith, Dp, FocusModifierExt, Modifier, RenderSlot, ScrollEventSource, State,
    current_frame_nanos, layout::layout, provide_context, receive_frame_nanos, remember, tessera,
    use_context,
};

use crate::{
    animation::Animatable,
    nested_scroll::{
        NestedScrollConnection, PostScrollInput, PreScrollInput, ScrollDelta, ScrollVelocity,
    },
    scrollable::ScrollableController,
};

/// How a top app bar responds to vertical page scrolling.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AppBarScrollBehavior {
    /// Keep the bar expanded and change its color when content scrolls under
    /// it.
    Pinned,
    /// Collapse on upward movement and expand immediately on downward movement.
    EnterAlways,
    /// Collapse on upward movement and expand after the content reaches its
    /// start.
    #[default]
    ExitUntilCollapsed,
}

/// Retained height, overlap, and settling motion for a scrolling top app bar.
pub struct AppBarScrollState {
    behavior: AppBarScrollBehavior,
    enabled: bool,
    snap: bool,
    range: f64,
    collapse: Animatable,
    color: Animatable,
    content_offset: f64,
    last_frame: Option<u64>,
    last_input: Option<u64>,
    touch_active: bool,
    pending_snap: bool,
    #[cfg(test)]
    pub(super) frame_ticks: u64,
}

impl Default for AppBarScrollState {
    fn default() -> Self {
        Self {
            behavior: AppBarScrollBehavior::default(),
            enabled: true,
            snap: true,
            range: 0.0,
            collapse: Animatable::spring(0.0, 380.0_f32.sqrt(), 0.8),
            color: Animatable::spring(0.0, 40.0, 1.0),
            content_offset: 0.0,
            last_frame: None,
            last_input: None,
            touch_active: false,
            pending_snap: false,
            #[cfg(test)]
            frame_ticks: 0,
        }
    }
}

impl AppBarScrollState {
    /// Returns zero when expanded and one when fully collapsed.
    pub fn collapsed_fraction(&self) -> f32 {
        self.collapse.value().clamp(0.0, 1.0)
    }

    /// Returns the negative logical height offset applied to the bar.
    pub fn height_offset(&self) -> Dp {
        Dp(-self.range * self.collapsed_fraction() as f64)
    }

    /// Returns the accumulated logical content displacement, at most zero.
    pub fn content_offset(&self) -> Dp {
        Dp(self.content_offset)
    }

    /// Sets a normalized collapsed amount immediately, cancelling a pending
    /// snap.
    pub fn set_collapsed_fraction(&mut self, fraction: f32) {
        let fraction = if fraction.is_finite() {
            fraction.clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.collapse = Animatable::spring(fraction, 380.0_f32.sqrt(), 0.8);
        self.pending_snap = false;
    }

    /// Restores an expanded, unscrolled bar and stops its motion.
    pub fn reset(&mut self) {
        self.set_collapsed_fraction(0.0);
        self.content_offset = 0.0;
        self.color = Animatable::spring(0.0, 40.0, 1.0);
        self.touch_active = false;
        self.last_input = None;
        self.last_frame = None;
    }

    /// Returns whether motion or a wheel-settling frame is still pending.
    pub fn is_animating(&self) -> bool {
        !self.collapse.is_idle()
            || !self.color.is_idle()
            || (self.pending_snap && !self.touch_active)
    }

    pub(super) fn configure_height(&mut self, expanded: Dp, compact: Dp) {
        self.range = if self.behavior == AppBarScrollBehavior::Pinned {
            0.0
        } else {
            (expanded.0 - compact.0).max(0.0)
        };
        if self.range > 0.0 {
            self.color = Animatable::spring(0.0, 40.0, 1.0);
        } else {
            self.color
                .set_target(if self.content_offset < 0.0 { 1.0 } else { 0.0 });
        }
    }

    pub(super) fn behavior(&self) -> AppBarScrollBehavior {
        self.behavior
    }
    pub(super) fn height_range(&self) -> Dp {
        Dp(self.range)
    }
    pub(super) fn drag_enabled(&self) -> bool {
        self.enabled && self.behavior != AppBarScrollBehavior::Pinned
    }
    pub(super) fn drag_by(&mut self, pixels: f32) {
        if self.drag_enabled() {
            self.input(ScrollEventSource::Touch);
            self.consume(pixels);
        }
    }
    pub(super) fn end_drag(&mut self) {
        self.finish(current_frame_nanos());
    }

    pub(super) fn color_fraction(&self) -> f32 {
        if self.range > 0.0 && self.behavior != AppBarScrollBehavior::Pinned {
            crate::progress::cubic_bezier_easing(self.collapsed_fraction(), 0.4, 0.0, 1.0, 1.0)
        } else {
            self.color.value().clamp(0.0, 1.0)
        }
    }

    fn input(&mut self, source: ScrollEventSource) {
        let frame = current_frame_nanos();
        if !self.is_animating() {
            self.last_frame = Some(frame);
        }
        self.last_input = Some(frame);
        self.touch_active = source == ScrollEventSource::Touch;
        self.pending_snap = self.snap
            && self.range > 0.0
            && self.collapsed_fraction() > 0.0
            && self.collapsed_fraction() < 1.0;
    }

    fn consume(&mut self, pixels: f32) -> f32 {
        if self.range <= 0.0 || !pixels.is_finite() {
            return 0.0;
        }
        let density = Dp(1.0).to_pixels_f32().max(f32::EPSILON) as f64;
        let old = self.height_offset().0;
        let next = (old + pixels as f64 / density).clamp(-self.range, 0.0);
        self.set_collapsed_fraction((-next / self.range) as f32);
        self.pending_snap =
            self.snap && self.collapsed_fraction() > 0.0 && self.collapsed_fraction() < 1.0;
        ((next - old) * density) as f32
    }

    fn pre_scroll(&mut self, input: PreScrollInput) -> ScrollDelta {
        if !self.enabled || input.available.y == 0.0 {
            return ScrollDelta::ZERO;
        }
        self.input(input.source);
        if self.behavior == AppBarScrollBehavior::Pinned
            || (self.behavior == AppBarScrollBehavior::ExitUntilCollapsed
                && input.available.y > 0.0)
        {
            return ScrollDelta::ZERO;
        }
        ScrollDelta::new(0.0, self.consume(input.available.y))
    }

    fn post_scroll(&mut self, input: PostScrollInput) -> ScrollDelta {
        if !self.enabled || (input.consumed_by_child.y == 0.0 && input.available.y == 0.0) {
            return ScrollDelta::ZERO;
        }
        self.input(input.source);
        self.content_offset = (self.content_offset
            + input.consumed_by_child.y as f64 / Dp(1.0).to_pixels_f32() as f64)
            .min(0.0);
        if input.available.y > 0.0 {
            self.content_offset = 0.0;
        }
        if self.range <= 0.0 {
            self.color
                .set_target(if self.content_offset < 0.0 { 1.0 } else { 0.0 });
        }
        let consumed = if self.behavior == AppBarScrollBehavior::ExitUntilCollapsed
            && input.available.y > 0.0
        {
            self.consume(input.available.y)
        } else {
            0.0
        };
        ScrollDelta::new(0.0, consumed)
    }

    fn finish(&mut self, frame: u64) {
        self.touch_active = false;
        self.last_frame = Some(frame);
        if self.pending_snap && self.snap {
            self.collapse
                .set_target(if self.collapsed_fraction() < 0.5 {
                    0.0
                } else {
                    1.0
                });
        }
        self.pending_snap = false;
    }

    fn tick(&mut self, frame: u64) {
        #[cfg(test)]
        {
            self.frame_ticks += 1;
        }
        let previous = self.last_frame.replace(frame).unwrap_or(frame);
        if self.pending_snap
            && !self.touch_active
            && self
                .last_input
                .is_some_and(|last| frame.saturating_sub(last) >= 150_000_000)
        {
            self.finish(frame);
        }
        let dt = frame.saturating_sub(previous) as f32 / 1_000_000_000.0;
        self.collapse.tick(dt);
        self.color.tick(dt);
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct AppBarScrollContext(pub State<AppBarScrollState>);

/// # app_bar_scroll_provider
///
/// Coordinate a top app bar and scrollable page content for navigation and
/// feeds.
///
/// ## Usage
///
/// Wrap both the app bar and its scrolling content, such as a complete
/// scaffold.
///
/// ## Parameters
///
/// - `state` — optional external height and overlap state.
/// - `behavior` — pinned, immediately expanding, or expand-at-start behavior.
/// - `enabled` — whether scroll input updates the bar.
/// - `snap` — settle an intermediate height after scrolling ends; defaults to
///   true.
/// - `scroll_controller` — optional content controller to synchronize overlap
///   after programmatic scrolling.
/// - `child` — app bar and scrolling page content in the shared scroll scope.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     app_bar::{AppBarScrollState, TopAppBarVariant, app_bar_scroll_provider, top_app_bar},
///     column::column,
///     modifier::ModifierExt as _,
///     scaffold::scaffold,
///     scrollable::ScrollableController,
///     text::text,
/// };
/// use tessera_ui::{Modifier, remember, tessera};
/// #[tessera]
/// fn screen() {
///     let state = remember(AppBarScrollState::default);
///     let scroll = remember(ScrollableController::new);
///     app_bar_scroll_provider()
///         .state(state)
///         .scroll_controller(scroll)
///         .child(move || {
///             scaffold()
///                 .top_bar(|| {
///                     top_app_bar()
///                         .variant(TopAppBarVariant::MediumFlexible)
///                         .title("Inbox");
///                 })
///                 .content(move || {
///                     column()
///                         .modifier(Modifier::new().vertical_scroll(scroll))
///                         .children(|| {
///                             text().content("Messages");
///                         });
///                 });
///         });
///     assert_eq!(state.with(AppBarScrollState::collapsed_fraction), 0.0);
/// }
/// # #[tessera]
/// # fn app() {
/// #     tessera_components::theme::material_theme().child(|| screen());
/// # }
/// # app();
/// ```
#[tessera]
pub fn app_bar_scroll_provider(
    state: Option<State<AppBarScrollState>>,
    behavior: Option<AppBarScrollBehavior>,
    enabled: Option<bool>,
    snap: Option<bool>,
    scroll_controller: Option<State<ScrollableController>>,
    child: Option<RenderSlot>,
) {
    let state = if let Some(state) = state {
        state
    } else {
        remember(AppBarScrollState::default)
    };
    let active = remember(|| state);
    if active.get() != state {
        active.set(state);
    }
    let behavior = behavior.unwrap_or_default();
    let enabled = enabled.unwrap_or(true);
    let snap = snap.unwrap_or(true);
    if state.with(|value| (value.behavior, value.enabled, value.snap)) != (behavior, enabled, snap)
    {
        state.with_mut(|value| {
            value.behavior = behavior;
            value.enabled = enabled;
            value.snap = snap;
            value.pending_snap = false;
            if !enabled || !snap {
                let fraction = value.collapsed_fraction();
                value.set_collapsed_fraction(fraction);
                value.touch_active = false;
            }
            if behavior == AppBarScrollBehavior::Pinned {
                value.set_collapsed_fraction(0.0);
            }
        });
    }
    if enabled && let Some(controller) = scroll_controller {
        let offset = controller.with(|controller| controller.target_position().y as f64)
            / Dp(1.0).to_pixels_f32().max(f32::EPSILON) as f64;
        if state.with(|value| value.content_offset) != offset {
            let frame = current_frame_nanos();
            state.with_mut(|value| {
                value.content_offset = offset.min(0.0);
                if value.range <= 0.0 {
                    if value.color.is_idle() {
                        value.last_frame = Some(frame);
                    }
                    value
                        .color
                        .set_target(if offset >= 0.0 { 0.0 } else { 1.0 });
                }
            });
        }
    }
    if state.with(AppBarScrollState::is_animating) {
        receive_frame_nanos(move |frame| {
            let state = active.get();
            state.with_mut(|value| value.tick(frame));
            if state.with(AppBarScrollState::is_animating) {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }
    let parent = use_context::<NestedScrollConnection>().map(|context| context.get());
    let coordinate_scale = remember(|| 1.0_f32);
    let connection = NestedScrollConnection::new()
        .with_coordinate_scale(coordinate_scale)
        .with_parent(parent)
        .with_pre_scroll_handler(CallbackWith::new(move |input| {
            active.get().with_mut(|value| value.pre_scroll(input))
        }))
        .with_post_scroll_handler(CallbackWith::new(move |input| {
            active.get().with_mut(|value| value.post_scroll(input))
        }))
        .with_post_fling_handler(CallbackWith::new(move |_| {
            let frame = current_frame_nanos();
            active.get().with_mut(|value| value.finish(frame));
            ScrollVelocity::ZERO
        }));
    let child = child.unwrap_or_else(RenderSlot::empty);
    layout()
        .modifier(Modifier::new().on_local_to_window(
            move |transform: tessera_ui::LocalToWindow| {
                if coordinate_scale.get() != transform.scale() {
                    coordinate_scale.set(transform.scale());
                }
            },
        ))
        .child(move || {
            let connection = connection.clone();
            provide_context(
                || AppBarScrollContext(state),
                move || {
                    let context =
                        use_context::<AppBarScrollContext>().expect("app bar scroll context");
                    if context.get().0 != state {
                        context.set(AppBarScrollContext(state));
                    }
                    let connection = connection.clone();
                    provide_context(
                        {
                            let connection = connection.clone();
                            move || connection
                        },
                        move || {
                            let context = use_context::<NestedScrollConnection>()
                                .expect("nested scroll context");
                            if context.get() != connection {
                                context.set(connection.clone());
                            }
                            child.render();
                        },
                    );
                },
            );
        });
}
