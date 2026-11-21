//! Circular buttons for contextual help.
//!
//! ## Usage
//!
//! Open documentation or guidance related to the current task.

use tessera_foundation::{
    modifier::{ClickableArgs, InteractionState, ModifierExt as _},
    pointer_gestures::TapGestureConfig,
};
use tessera_ui::{
    AxisConstraint, Callback, Color, ComputedData, Constraint, Dp, DrawModifierContent,
    DrawModifierContext, DrawModifierNode, FocusRequester, LayoutPolicy, LayoutResult,
    MeasurementError, Modifier, PxPosition,
    accesskit::Role,
    layout::{MeasureScope, layout},
    modifier::{CursorModifierExt as _, ModifierCapabilityExt as _},
    remember, tessera, use_context,
    winit::window::CursorIcon,
};

use crate::{
    modifier::ModifierExt as _,
    pipelines::perceptual_fill::{PerceptualFillCommand, PerceptualPath},
    shape_def::Shape,
};

use super::{
    choice::ChoiceKeyboard,
    theme::{Appearance, HigTheme},
};

#[cfg(test)]
#[path = "help_link_tests.rs"]
mod tests;

#[derive(Clone, Copy, PartialEq)]
struct HelpLayout;

impl LayoutPolicy for HelpLayout {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = scope.parent_constraint();
        let diameter = Dp(24.0).into();
        let width = parent.width().clamp(diameter);
        let height = parent.height().clamp(diameter);
        let mut result = LayoutResult::new(ComputedData { width, height });
        for child in scope.children() {
            child.measure(
                scope,
                &Constraint::new(
                    AxisConstraint::exact(diameter),
                    AxisConstraint::exact(diameter),
                ),
            )?;
            result.place_child(
                child,
                PxPosition::from_f32_xy(
                    (width - diameter).to_f32() * 0.5,
                    (height - diameter).to_f32() * 0.5,
                ),
            );
        }
        Ok(result)
    }
}

#[derive(Clone, PartialEq)]
struct HelpBackground {
    background: Color,
    foreground: Color,
}

impl DrawModifierNode for HelpBackground {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        context
            .render_input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(PerceptualFillCommand {
                color: self.background,
                radii: [Dp(12.0); 2],
                insets: [Dp::ZERO; 4],
                clip_radius: None,
                edge_smoothness: 1.0,
                stroke: None,
                path: Some(PerceptualPath {
                    color: self.foreground,
                    width: Dp(1.505),
                    points: [
                        [9.262, 9.088],
                        [9.410, 7.901],
                        [10.982, 6.569],
                        [13.373, 7.360],
                        [14.331, 7.734],
                        [15.318, 9.162],
                        [14.671, 9.531],
                        [15.294, 9.755],
                        [14.316, 10.994],
                        [13.766, 11.428],
                        [11.603, 12.928],
                        [11.783, 13.211],
                        [11.735, 14.193],
                    ]
                    .map(|point| point.map(|coordinate| coordinate / 24.0)),
                    dot: [11.763 / 24.0, 17.473 / 24.0],
                    dot_radius: Dp(1.015),
                }),
                line: None,
                outline_width: Dp::ZERO,
                snap_to_pixels: false,
            });
        content.draw(context.render_input);
    }
}

#[tessera]
fn help_face(background: Color, foreground: Color, focused: bool) {
    let mut modifier = Modifier::new().push_draw(HelpBackground {
        background,
        foreground,
    });
    if focused {
        modifier = modifier.border_with_shape(
            Dp(2.0),
            Color::from_rgb(0.0, 0.19461783, 1.0).with_alpha(0.6),
            Shape::Ellipse,
        );
    }
    layout(tessera_ui::layout::Stack).modifier(modifier);
}

/// # help_link
///
/// Opens contextual help from a circular question-mark button.
///
/// ## Usage
///
/// Offer documentation or explanations without leaving the current workflow.
///
/// ## Parameters
///
/// - `on_click` — action requested by pointer release, Space release, or accessibility.
/// - `enabled` — input and focus availability; defaults to true.
/// - `window_active` — active-window feedback; defaults to true.
/// - `modifier`, `focus_requester` — layout, semantics, and optional external focus handle.
/// - `accessibility_label`, `accessibility_description` — spoken name (defaults to "Help") and context.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::help_link::help_link;
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// help_link().on_click(|| println!("Show the export guide"));
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn help_link(
    on_click: Option<Callback>,
    enabled: Option<bool>,
    window_active: Option<bool>,
    modifier: Option<Modifier>,
    focus_requester: Option<FocusRequester>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let enabled = enabled.unwrap_or(true);
    let active = window_active.unwrap_or(true);
    let next = (enabled, on_click.unwrap_or_else(Callback::noop));
    let value = remember(|| next);
    if value.get() != next {
        value.set(next);
    }
    let activate = Callback::new(move || {
        let (enabled, callback) = value.get();
        if enabled {
            callback.call();
        }
    });
    let requester = if let Some(requester) = focus_requester {
        requester
    } else {
        remember(FocusRequester::new).get()
    };
    let interaction = remember(InteractionState::new);
    let held = remember(|| false);
    if !enabled {
        if held.get() {
            held.set(false);
        }
    } else if held.get() && !requester.is_focused() {
        held.set(false);
    }
    let pressed = enabled
        && active
        && (held.get() || interaction.with(|state| state.is_pressed() && state.is_hovered()));
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    let tint = if dark { Color::WHITE } else { Color::BLACK };
    let background = tint.with_alpha(if !enabled {
        0.04
    } else if pressed {
        0.16
    } else {
        0.075
    });
    let foreground = tint.with_alpha(if enabled { 0.85 } else { 0.26 });
    let modifier = modifier
        .unwrap_or_default()
        .clickable(ClickableArgs {
            enabled,
            on_click: activate,
            role: Some(Role::Button),
            label: Some(accessibility_label.unwrap_or_else(|| "Help".into())),
            description: accessibility_description,
            focus_requester: Some(requester),
            interaction_state: Some(interaction),
            keyboard_activation: false,
            tap_config: TapGestureConfig {
                slop_px: f32::INFINITY,
                cancel_on_exit: false,
                ..Default::default()
            },
            ..Default::default()
        })
        .push_keyboard_input(ChoiceKeyboard {
            enabled,
            held,
            activate,
        })
        .hover_cursor_icon(CursorIcon::Default);
    layout(HelpLayout).modifier(modifier).child(move || {
        help_face(
            background,
            foreground,
            enabled && active && requester.is_focus_visible(),
        );
    });
}
