//! An interactive slider component for selecting a value in a range.
//!
//! ## Usage
//!
//! Use to allow users to select a value from a continuous range.
use std::sync::Arc;

use derive_builder::Builder;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tessera_ui::{
    Color, ComputedData, Constraint, CursorEventContent, DimensionValue, Dp, InputHandlerInput,
    MeasureInput, MeasurementError, Px, PxPosition,
    accesskit::{Action, Role},
    focus_state::Focus,
    tessera,
    winit::window::CursorIcon,
};

use crate::{
    material_color,
    shape_def::{RoundedCorner, Shape},
    surface::{SurfaceArgsBuilder, surface},
};

const ACCESSIBILITY_STEP: f32 = 0.05;
const MIN_TOUCH_TARGET: Dp = Dp(40.0);
const HANDLE_GAP: Dp = Dp(6.0);
const HANDLE_HEIGHT_DEFAULT: Dp = Dp(44.0);
const DECORATION_DIAMETER: Dp = Dp(4.0);

/// Stores the interactive state for the [`slider`] component, such as whether the slider is currently being dragged by the user.
/// The [`SliderState`] handle owns the necessary locking internally, so callers can simply clone and pass it between components.
pub(crate) struct SliderStateInner {
    /// True if the user is currently dragging the slider.
    pub is_dragging: bool,
    /// The focus handler for the slider.
    pub focus: Focus,
    /// True when the cursor is hovering inside the slider bounds.
    pub is_hovered: bool,
}

impl Default for SliderStateInner {
    fn default() -> Self {
        Self::new()
    }
}

impl SliderStateInner {
    pub fn new() -> Self {
        Self {
            is_dragging: false,
            focus: Focus::new(),
            is_hovered: false,
        }
    }
}

/// External state for the `slider` component.
///
/// # Example
///
/// ```
/// use tessera_ui_basic_components::slider::SliderState;
///
/// let slider_state = SliderState::new();
/// ```
#[derive(Clone)]
pub struct SliderState {
    inner: Arc<RwLock<SliderStateInner>>,
}

impl SliderState {
    /// Creates a new slider state handle.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(SliderStateInner::new())),
        }
    }

    pub(crate) fn read(&self) -> RwLockReadGuard<'_, SliderStateInner> {
        self.inner.read()
    }

    pub(crate) fn write(&self) -> RwLockWriteGuard<'_, SliderStateInner> {
        self.inner.write()
    }

    /// Returns whether the slider handle is currently being dragged.
    pub fn is_dragging(&self) -> bool {
        self.inner.read().is_dragging
    }

    /// Manually sets the dragging flag. Useful for custom gesture integrations.
    pub fn set_dragging(&self, dragging: bool) {
        self.inner.write().is_dragging = dragging;
    }

    /// Requests focus for the slider.
    pub fn request_focus(&self) {
        self.inner.write().focus.request_focus();
    }

    /// Clears focus from the slider if it is currently focused.
    pub fn clear_focus(&self) {
        self.inner.write().focus.unfocus();
    }

    /// Returns `true` if this slider currently holds focus.
    pub fn is_focused(&self) -> bool {
        self.inner.read().focus.is_focused()
    }

    /// Returns `true` if the cursor is hovering over this slider.
    pub fn is_hovered(&self) -> bool {
        self.inner.read().is_hovered
    }
}

impl Default for SliderState {
    fn default() -> Self {
        Self::new()
    }
}

/// Arguments for the `slider` component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct SliderArgs {
    /// The current value of the slider, ranging from 0.0 to 1.0.
    #[builder(default = "0.0")]
    pub value: f32,

    /// Callback function triggered when the slider's value changes.
    #[builder(default = "Arc::new(|_| {})")]
    pub on_change: Arc<dyn Fn(f32) + Send + Sync>,

    /// Total width of the slider control.
    #[builder(default = "DimensionValue::Fixed(Dp(260.0).to_px())")]
    pub width: DimensionValue,

    /// The height of the slider track.
    #[builder(default = "Dp(16.0)")]
    pub track_height: Dp,

    /// The color of the active part of the track (progress fill).
    #[builder(default = "crate::material_color::global_material_scheme().primary")]
    pub active_track_color: Color,

    /// The color of the inactive part of the track (background).
    #[builder(default = "crate::material_color::global_material_scheme().secondary_container")]
    pub inactive_track_color: Color,

    /// The thickness of the handle indicator.
    #[builder(default = "Dp(4.0)")]
    pub thumb_diameter: Dp,

    /// Color of the handle indicator.
    #[builder(default = "crate::material_color::global_material_scheme().primary")]
    pub thumb_color: Color,

    /// Height of the handle focus layer (hover/drag halo).
    #[builder(default = "Dp(18.0)")]
    pub state_layer_diameter: Dp,

    /// Base color for the state layer; alpha will be adjusted per interaction state.
    #[builder(
        default = "crate::material_color::global_material_scheme().primary.with_alpha(0.18)"
    )]
    pub state_layer_color: Color,

    /// Disable interaction.
    #[builder(default = "false")]
    pub disabled: bool,
    /// Optional accessibility label read by assistive technologies.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_label: Option<String>,
    /// Optional accessibility description.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_description: Option<String>,
}

/// Helper: check if a cursor position is within the bounds of a component.
fn cursor_within_component(cursor_pos: Option<PxPosition>, computed: &ComputedData) -> bool {
    if let Some(pos) = cursor_pos {
        let within_x = pos.x.0 >= 0 && pos.x.0 < computed.width.0;
        let within_y = pos.y.0 >= 0 && pos.y.0 < computed.height.0;
        within_x && within_y
    } else {
        false
    }
}

/// Helper: compute normalized progress (0.0..1.0) from cursor X and overall width.
/// Returns None when cursor is not available.
fn cursor_progress(cursor_pos: Option<PxPosition>, layout: &SliderLayout) -> Option<f32> {
    if layout.component_width.0 <= 0 {
        return None;
    }
    cursor_pos.map(|pos| {
        (pos.x.0 as f32 / layout.component_width.to_f32())
            .clamp(0.0, 1.0)
    })
}

fn handle_slider_state(
    input: &mut InputHandlerInput,
    state: &SliderState,
    args: &SliderArgs,
    layout: &SliderLayout,
) {
    if args.disabled {
        let mut inner = state.write();
        inner.is_hovered = false;
        inner.is_dragging = false;
        return;
    }

    let is_in_component = cursor_within_component(input.cursor_position_rel, &input.computed_data);

    {
        let mut inner = state.write();
        inner.is_hovered = is_in_component;
    }

    if is_in_component {
        input.requests.cursor_icon = CursorIcon::Pointer;
    }

    if !is_in_component && !state.read().is_dragging {
        return;
    }

    let mut new_value: Option<f32> = None;

    handle_cursor_events(input, state, &mut new_value, layout);
    update_value_on_drag(input, state, &mut new_value, layout);
    notify_on_change(new_value, args);
}

fn handle_cursor_events(
    input: &mut InputHandlerInput,
    state: &SliderState,
    new_value: &mut Option<f32>,
    layout: &SliderLayout,
) {
    for event in input.cursor_events.iter() {
        match &event.content {
            CursorEventContent::Pressed(_) => {
                {
                    let mut inner = state.write();
                    inner.focus.request_focus();
                    inner.is_dragging = true;
                }
                if let Some(v) = cursor_progress(input.cursor_position_rel, layout) {
                    *new_value = Some(v);
                }
            }
            CursorEventContent::Released(_) => {
                state.write().is_dragging = false;
            }
            _ => {}
        }
    }
}

fn update_value_on_drag(
    input: &InputHandlerInput,
    state: &SliderState,
    new_value: &mut Option<f32>,
    layout: &SliderLayout,
) {
    if state.read().is_dragging
        && let Some(v) = cursor_progress(input.cursor_position_rel, layout)
    {
        *new_value = Some(v);
    }
}

fn notify_on_change(new_value: Option<f32>, args: &SliderArgs) {
    if let Some(v) = new_value
        && (v - args.value).abs() > f32::EPSILON
    {
        (args.on_change)(v);
    }
}

fn apply_slider_accessibility(
    input: &mut InputHandlerInput<'_>,
    args: &SliderArgs,
    current_value: f32,
    on_change: &Arc<dyn Fn(f32) + Send + Sync>,
) {
    let mut builder = input.accessibility().role(Role::Slider);

    if let Some(label) = args.accessibility_label.as_ref() {
        builder = builder.label(label.clone());
    }
    if let Some(description) = args.accessibility_description.as_ref() {
        builder = builder.description(description.clone());
    }

    builder = builder
        .numeric_value(current_value as f64)
        .numeric_range(0.0, 1.0);

    if args.disabled {
        builder = builder.disabled();
    } else {
        builder = builder
            .focusable()
            .action(Action::Increment)
            .action(Action::Decrement);
    }

    builder.commit();

    if args.disabled {
        return;
    }

    let value_for_handler = current_value;
    let on_change = on_change.clone();
    input.set_accessibility_action_handler(move |action| {
        let new_value = match action {
            Action::Increment => Some((value_for_handler + ACCESSIBILITY_STEP).clamp(0.0, 1.0)),
            Action::Decrement => Some((value_for_handler - ACCESSIBILITY_STEP).clamp(0.0, 1.0)),
            _ => None,
        };

        if let Some(new_value) = new_value
            && (new_value - value_for_handler).abs() > f32::EPSILON
        {
            on_change(new_value);
        }
    });
}

fn render_active_segment(layout: SliderLayout, colors: &SliderColors) {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::Fill { min: None, max: None })
            .height(DimensionValue::Fixed(layout.track_height))
            .style(colors.active_track.into())
            .shape(Shape::RoundedRectangle {
                top_left: RoundedCorner::Capsule,
                top_right: RoundedCorner::manual(layout.track_corner_radius, 3.0),
                bottom_right: RoundedCorner::manual(layout.track_corner_radius, 3.0),
                bottom_left: RoundedCorner::Capsule,
            })
            .build()
            .expect("builder construction failed"),
        None,
        || {},
    );
}

fn render_inactive_segment(layout: SliderLayout, colors: &SliderColors) {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::Fill { min: None, max: None })
            .height(DimensionValue::Fixed(layout.track_height))
            .style(colors.inactive_track.into())
            .shape(Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(layout.track_corner_radius, 3.0),
                top_right: RoundedCorner::Capsule,
                bottom_right: RoundedCorner::Capsule,
                bottom_left: RoundedCorner::manual(layout.track_corner_radius, 3.0),
            })
            .build()
            .expect("builder construction failed"),
        None,
        || {},
    );
}

fn render_focus(layout: SliderLayout, colors: &SliderColors) {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::Fixed(layout.focus_width))
            .height(DimensionValue::Fixed(layout.focus_height))
            .style(colors.handle_focus.into())
            .shape(Shape::capsule())
            .build()
            .expect("builder construction failed"),
        None,
        || {},
    );
}

fn render_handle(layout: SliderLayout, colors: &SliderColors) {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::Fixed(layout.handle_width))
            .height(DimensionValue::Fixed(layout.handle_height))
            .style(colors.handle.into())
            .shape(Shape::capsule())
            .build()
            .expect("builder construction failed"),
        None,
        || {},
    );
}

fn render_decoration_dot(layout: SliderLayout, colors: &SliderColors) {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::Fixed(layout.decoration_diameter))
            .height(DimensionValue::Fixed(layout.decoration_diameter))
            .style(colors.handle.into())
            .shape(Shape::Ellipse)
            .build()
            .expect("builder construction failed"),
        None,
        || {},
    );
}

fn measure_slider(
    input: &MeasureInput,
    layout: SliderLayout,
    clamped_value: f32,
) -> Result<ComputedData, MeasurementError> {
    let self_width = layout.component_width;
    let self_height = layout.component_height;

    let active_id = input.children_ids[0];
    let inactive_id = input.children_ids[1];
    let focus_id = input.children_ids[2];
    let handle_id = input.children_ids[3];
    let dot_id = input.children_ids[4];

    let active_width = layout.active_width(clamped_value);
    let inactive_width = layout.inactive_width(clamped_value);

    let active_constraint = Constraint::new(
        DimensionValue::Fixed(active_width),
        DimensionValue::Fixed(layout.track_height),
    );
    input.measure_child(active_id, &active_constraint)?;
    input.place_child(
        active_id,
        PxPosition::new(Px(0), layout.track_y),
    );

    let inactive_constraint = Constraint::new(
        DimensionValue::Fixed(inactive_width),
        DimensionValue::Fixed(layout.track_height),
    );
    input.measure_child(inactive_id, &inactive_constraint)?;
    input.place_child(
        inactive_id,
        PxPosition::new(
            Px(active_width.0 + layout.handle_gap.0 * 2 + layout.handle_width.0),
            layout.track_y,
        ),
    );

    let focus_constraint = Constraint::new(
        DimensionValue::Fixed(layout.focus_width),
        DimensionValue::Fixed(layout.focus_height),
    );
    input.measure_child(focus_id, &focus_constraint)?;

    let handle_constraint = Constraint::new(
        DimensionValue::Fixed(layout.handle_width),
        DimensionValue::Fixed(layout.handle_height),
    );
    input.measure_child(handle_id, &handle_constraint)?;

    let handle_center = layout.handle_center(clamped_value);
    let focus_offset = layout.center_child_offset(layout.focus_width);
    input.place_child(
        focus_id,
        PxPosition::new(
            Px(handle_center.x.0 - focus_offset.0),
            layout.focus_y,
        ),
    );

    let handle_offset = layout.center_child_offset(layout.handle_width);
    input.place_child(
        handle_id,
        PxPosition::new(
            Px(handle_center.x.0 - handle_offset.0),
            layout.handle_y,
        ),
    );

    let dot_size = layout.decoration_diameter;
    let dot_constraint = Constraint::new(
        DimensionValue::Fixed(dot_size),
        DimensionValue::Fixed(dot_size),
    );
    input.measure_child(dot_id, &dot_constraint)?;
    let dot_offset = layout.center_child_offset(layout.decoration_diameter);
    let inactive_start =
        active_width.0 + layout.handle_gap.0 * 2 + layout.handle_width.0;
    let padding = Dp(8.0).to_px() - dot_size / Px(2);
    let dot_center_x = Px(inactive_start + inactive_width.0 - padding.0);
    input.place_child(
        dot_id,
        PxPosition::new(
            Px(dot_center_x.0 - dot_offset.0),
            layout.decoration_y,
        ),
    );

    Ok(ComputedData {
        width: self_width,
        height: self_height,
    })
}

#[derive(Clone, Copy)]
struct SliderLayout {
    component_width: Px,
    component_height: Px,
    track_total_width: Px,
    track_height: Px,
    track_corner_radius: Dp,
    track_y: Px,
    handle_width: Px,
    handle_height: Px,
    handle_y: Px,
    handle_gap: Px,
    focus_width: Px,
    focus_height: Px,
    focus_y: Px,
    decoration_diameter: Px,
    decoration_y: Px,
}

impl SliderLayout {
    fn active_width(&self, value: f32) -> Px {
        let clamped = value.clamp(0.0, 1.0);
        Px::saturating_from_f32(self.track_total_width.to_f32() * clamped)
    }

    fn inactive_width(&self, value: f32) -> Px {
        let active = self.active_width(value);
        Px((self.track_total_width.0 - active.0).max(0))
    }

    fn center_child_offset(&self, width: Px) -> Px {
        Px(width.0 / 2)
    }

    fn handle_center(&self, value: f32) -> PxPosition {
        let active_width = self.active_width(value);
        let center_x = active_width.to_f32()
            + self.handle_gap.to_f32()
            + self.handle_width.to_f32() / 2.0;
        let max_x =
            (self.component_width.to_f32() - self.handle_width.to_f32() / 2.0).max(0.0);
        let clamped_x = center_x.clamp(self.handle_width.to_f32() / 2.0, max_x);
        PxPosition::new(Px(clamped_x.round() as i32), Px(self.component_height.0 / 2))
    }
}

fn resolve_component_width(args: &SliderArgs, parent_constraint: &Constraint) -> Px {
    let fallback = Dp(260.0).to_px();
    let merged = Constraint::new(args.width, DimensionValue::Fixed(args.track_height.to_px()))
        .merge(parent_constraint);

    match merged.width {
        DimensionValue::Fixed(px) => px,
        DimensionValue::Fill { max, .. } | DimensionValue::Wrap { max, .. } => {
            max.unwrap_or(fallback)
        }
    }
}

fn fallback_component_width(args: &SliderArgs) -> Px {
    match args.width {
        DimensionValue::Fixed(px) => px,
        DimensionValue::Fill { max, .. } | DimensionValue::Wrap { max, .. } => {
            max.unwrap_or(Dp(260.0).to_px())
        }
    }
}

fn slider_layout(args: &SliderArgs, component_width: Px) -> SliderLayout {
    let handle_width = args.thumb_diameter.to_px();
    let track_height = args.track_height.to_px();
    let touch_target_height = MIN_TOUCH_TARGET.to_px();
    let handle_gap = HANDLE_GAP.to_px();
    let handle_height = HANDLE_HEIGHT_DEFAULT.to_px();
    let focus_width = Px((handle_width.to_f32() * 1.6).round() as i32);
    let focus_height = Px((handle_height.to_f32() * 1.2).round() as i32);
    let decoration_diameter = DECORATION_DIAMETER.to_px();
    let track_corner_radius = Dp(args.track_height.0 / 2.0);

    let track_total_width =
        Px((component_width.0 - handle_width.0 - handle_gap.0 * 2).max(0));

    let component_height = Px(
        *[
            track_height.0,
            handle_height.0,
            focus_height.0,
            touch_target_height.0,
        ]
        .iter()
        .max()
        .expect("non-empty"),
    );
    let track_y = Px((component_height.0 - track_height.0) / 2);

    SliderLayout {
        component_width,
        component_height,
        track_total_width,
        track_height,
        track_corner_radius,
        track_y,
        handle_width,
        handle_height,
        handle_gap,
        handle_y: Px((component_height.0 - handle_height.0) / 2),
        focus_width,
        focus_height,
        focus_y: Px((component_height.0 - focus_height.0) / 2),
        decoration_diameter,
        decoration_y: Px((component_height.0 - decoration_diameter.0) / 2),
    }
}

#[derive(Clone, Copy)]
struct SliderColors {
    active_track: Color,
    inactive_track: Color,
    handle: Color,
    handle_focus: Color,
}

fn slider_colors(args: &SliderArgs, is_hovered: bool, is_dragging: bool) -> SliderColors {
    if args.disabled {
        let scheme = material_color::global_material_scheme();
        return SliderColors {
            active_track: scheme.on_surface.with_alpha(0.38),
            inactive_track: scheme.on_surface.with_alpha(0.12),
            handle: scheme.on_surface.with_alpha(0.38),
            handle_focus: Color::new(0.0, 0.0, 0.0, 0.0),
        };
    }

    let mut state_layer_alpha_scale = 0.0;
    if is_dragging {
        state_layer_alpha_scale = 1.0;
    } else if is_hovered {
        state_layer_alpha_scale = 0.7;
    }
    let base_state = args.state_layer_color;
    let state_layer_alpha = (base_state.a * state_layer_alpha_scale).clamp(0.0, 1.0);
    let handle_focus = Color::new(base_state.r, base_state.g, base_state.b, state_layer_alpha);

    SliderColors {
        active_track: args.active_track_color,
        inactive_track: args.inactive_track_color,
        handle: args.thumb_color,
        handle_focus,
    }
}

/// # slider
///
/// Renders an interactive slider with a bar-style handle for selecting a value between 0.0 and 1.0.
///
/// ## Usage
///
/// Use for settings like volume or brightness, or for any user-adjustable value.
///
/// ## Parameters
///
/// - `args` — configures the slider's value, appearance, and callbacks; see [`SliderArgs`].
/// - `state` — a clonable [`SliderState`] to manage interaction state like dragging and focus.
///
/// ## Examples
///
/// ```
/// use std::sync::Arc;
/// use tessera_ui::{DimensionValue, Dp};
/// use tessera_ui_basic_components::slider::{slider, SliderArgsBuilder, SliderState};
///
/// // In a real application, you would manage this state.
/// let slider_state = SliderState::new();
///
/// slider(
///     SliderArgsBuilder::default()
///         .width(DimensionValue::Fixed(Dp(200.0).to_px()))
///         .value(0.5)
///         .on_change(Arc::new(|new_value| {
///             // In a real app, you would update your state here.
///             println!("Slider value changed to: {}", new_value);
///         }))
///         .build()
///         .unwrap(),
///     slider_state,
/// );
/// ```
#[tessera]
pub fn slider(args: impl Into<SliderArgs>, state: SliderState) {
    let args: SliderArgs = args.into();
    let initial_width = fallback_component_width(&args);
    let layout = slider_layout(&args, initial_width);
    let clamped_value = args.value.clamp(0.0, 1.0);
    let state_snapshot = state.read();
    let colors = slider_colors(&args, state_snapshot.is_hovered, state_snapshot.is_dragging);
    drop(state_snapshot);

    render_active_segment(layout, &colors);
    render_inactive_segment(layout, &colors);
    render_focus(layout, &colors);
    render_handle(layout, &colors);
    render_decoration_dot(layout, &colors);

    let cloned_args = args.clone();
    let state_clone = state.clone();
    let clamped_value_for_accessibility = clamped_value;
    input_handler(Box::new(move |mut input| {
        let resolved_layout =
            slider_layout(&cloned_args, input.computed_data.width);
        handle_slider_state(&mut input, &state_clone, &cloned_args, &resolved_layout);
        apply_slider_accessibility(
            &mut input,
            &cloned_args,
            clamped_value_for_accessibility,
            &cloned_args.on_change,
        );
    }));

    measure(Box::new(move |input| {
        let component_width = resolve_component_width(&args, input.parent_constraint);
        let resolved_layout = slider_layout(&args, component_width);
        measure_slider(input, resolved_layout, clamped_value)
    }));
}
