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
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
};

/// Stores the interactive state for the [`slider`] component, such as whether the slider is currently being dragged by the user.
/// The [`SliderState`] handle owns the necessary locking internally, so callers can simply clone and pass it between components.
pub(crate) struct SliderStateInner {
    /// True if the user is currently dragging the slider.
    pub is_dragging: bool,
    /// The focus handler for the slider.
    pub focus: Focus,
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
        }
    }
}

#[derive(Clone)]
pub struct SliderState {
    inner: Arc<RwLock<SliderStateInner>>,
}

impl SliderState {
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

    /// Returns whether the slider thumb is currently being dragged.
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

    /// The width of the slider track.
    #[builder(default = "Dp(200.0)")]
    pub width: Dp,

    /// The height of the slider track.
    #[builder(default = "Dp(12.0)")]
    pub track_height: Dp,

    /// The color of the active part of the track (progress fill).
    #[builder(default = "Color::new(0.2, 0.5, 0.8, 1.0)")]
    pub active_track_color: Color,

    /// The color of the inactive part of the track (background).
    #[builder(default = "Color::new(0.8, 0.8, 0.8, 1.0)")]
    pub inactive_track_color: Color,

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

/// Helper: compute normalized progress (0.0..1.0) from cursor X and width.
/// Returns None when cursor is not available.
fn cursor_progress(cursor_pos: Option<PxPosition>, width_f: f32) -> Option<f32> {
    cursor_pos.map(|pos| (pos.x.0 as f32 / width_f).clamp(0.0, 1.0))
}

fn handle_slider_state(input: &mut InputHandlerInput, state: &SliderState, args: &SliderArgs) {
    if args.disabled {
        return;
    }

    let is_in_component = cursor_within_component(input.cursor_position_rel, &input.computed_data);

    if is_in_component {
        input.requests.cursor_icon = CursorIcon::Pointer;
    }

    if !is_in_component && !state.read().is_dragging {
        return;
    }

    let width_f = input.computed_data.width.0 as f32;
    let mut new_value: Option<f32> = None;

    handle_cursor_events(input, state, &mut new_value, width_f);
    update_value_on_drag(input, state, &mut new_value, width_f);
    notify_on_change(new_value, args);
}

fn handle_cursor_events(
    input: &mut InputHandlerInput,
    state: &SliderState,
    new_value: &mut Option<f32>,
    width_f: f32,
) {
    for event in input.cursor_events.iter() {
        match &event.content {
            CursorEventContent::Pressed(_) => {
                {
                    let mut inner = state.write();
                    inner.focus.request_focus();
                    inner.is_dragging = true;
                }
                if let Some(v) = cursor_progress(input.cursor_position_rel, width_f) {
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
    width_f: f32,
) {
    if state.read().is_dragging
        && let Some(v) = cursor_progress(input.cursor_position_rel, width_f)
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

fn render_track(args: &SliderArgs) {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::Fixed(args.width.to_px()))
            .height(DimensionValue::Fixed(args.track_height.to_px()))
            .style(args.inactive_track_color.into())
            .shape({
                let radius = Dp(args.track_height.0 / 2.0);
                Shape::RoundedRectangle {
                    top_left: radius,
                    top_right: radius,
                    bottom_right: radius,
                    bottom_left: radius,
                    g2_k_value: 2.0, // Capsule shape
                }
            })
            .build().expect("builder construction failed"),
        None,
        move || {
            render_progress_fill(args);
        },
    );
}

fn render_progress_fill(args: &SliderArgs) {
    let progress_width = args.width.to_px().to_f32() * args.value;
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::Fixed(Px(progress_width as i32)))
            .height(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .style(args.active_track_color.into())
            .shape({
                let radius = Dp(args.track_height.0 / 2.0);
                Shape::RoundedRectangle {
                    top_left: radius,
                    top_right: radius,
                    bottom_right: radius,
                    bottom_left: radius,
                    g2_k_value: 2.0, // Capsule shape
                }
            })
            .build().expect("builder construction failed"),
        None,
        || {},
    );
}

fn measure_slider(
    input: &MeasureInput,
    args: &SliderArgs,
) -> Result<ComputedData, MeasurementError> {
    let self_width = args.width.to_px();
    let self_height = args.track_height.to_px();

    let track_id = input.children_ids[0];

    // Measure track
    let track_constraint = Constraint::new(
        DimensionValue::Fixed(self_width),
        DimensionValue::Fixed(self_height),
    );
    input.measure_child(track_id, &track_constraint)?;
    input.place_child(track_id, PxPosition::new(Px(0), Px(0)));

    Ok(ComputedData {
        width: self_width,
        height: self_height,
    })
}

/// # slider
///
/// Renders an interactive slider for selecting a value between 0.0 and 1.0.
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
/// use tessera_ui::Dp;
/// use tessera_ui_basic_components::slider::{slider, SliderArgsBuilder, SliderState};
///
/// // In a real application, you would manage this state.
/// let slider_state = SliderState::new();
///
/// slider(
///     SliderArgsBuilder::default()
///         .width(Dp(200.0))
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

    render_track(&args);

    let cloned_args = args.clone();
    let state_clone = state.clone();
    input_handler(Box::new(move |mut input| {
        handle_slider_state(&mut input, &state_clone, &cloned_args);
        apply_slider_accessibility(
            &mut input,
            &cloned_args,
            cloned_args.value,
            &cloned_args.on_change,
        );
    }));

    let cloned_args = args.clone();
    measure(Box::new(move |input| measure_slider(input, &cloned_args)));
}
const ACCESSIBILITY_STEP: f32 = 0.05;


