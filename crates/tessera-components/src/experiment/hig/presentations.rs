//! Tooltips, popovers, alerts, confirmation dialogs, sheets, and inspectors.
//!
//! ## Usage
//!
//! Show help, focused tasks, and important decisions above or beside window content.

use std::time::Duration;

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, AxisConstraint, Callback, CallbackWith, Color,
    ComputedData, Constraint, Dp, FloatRect, LayoutPolicy, LayoutResult, MeasurementError,
    Modifier, PointerEventPass, PointerInput, PointerInputModifierNode, Px, PxPosition, RenderSlot,
    SemanticsModifierNode, State,
    accesskit::Role,
    launch,
    layout::{MeasureScope, Stack, layout},
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    remember, tessera, use_context,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    animation::{Delay, animate_to, run_frames},
    boxed::boxed,
    column::column,
    modifier::ModifierExt as _,
    shape_def::Shape,
    spacer::spacer,
};

use super::{
    button::{ButtonRole, ButtonScope, ButtonStyle, button, button_scope},
    motion::Eased,
    presentation::{PopupPlacement, SurfaceStyle, present_surface},
    text::text,
    theme::{Appearance, HigTheme, TextStyle},
    toggle::{ToggleStyle, toggle},
};

#[cfg(test)]
#[path = "presentations_tests.rs"]
mod tests;

/// Pause before a tooltip appears.
const TOOLTIP_DELAY: Duration = Duration::from_millis(1000);
/// Duration of an inspector opening or closing.
const INSPECTOR_MOTION: Duration = Duration::from_millis(250);
/// Width of an inspector.
const INSPECTOR_WIDTH: f64 = 260.0;

struct Hover(State<bool>);

impl PointerInputModifierNode for Hover {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        if input.pass != PointerEventPass::Main {
            return;
        }
        let inside = input.cursor_position_rel.is_some_and(|position| {
            position.x >= 0.0
                && position.y >= 0.0
                && position.x < input.computed_data.width.to_f32()
                && position.y < input.computed_data.height.to_f32()
        });
        if self.0.get() != inside {
            self.0.set(inside);
        }
    }
}

struct Tooltip(String);

impl SemanticsModifierNode for Tooltip {
    fn apply(&self, node: &mut AccessibilityNode, _: &mut Option<AccessibilityActionHandler>) {
        node.tooltip = Some(self.0.clone());
    }
}

/// # help
///
/// Shows a tooltip for its content after the pointer rests on it.
///
/// ## Usage
///
/// Explain icon-only buttons and other controls whose purpose is not obvious.
///
/// ## Parameters
///
/// - `text` — the tooltip, also published as the content's accessibility hint.
/// - `child` — the described content.
///
/// The tooltip appears in the nearest
/// [`presentation_host`](super::presentation::presentation_host).
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{button::button, presentations::help};
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// help("Share this document").child(|| {
///     button("Share");
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn help(#[prop(into)] text: String, child: Option<RenderSlot>) {
    let hovered = remember(|| false);
    let shown = remember(|| false);
    let anchor = remember(|| FloatRect::ZERO);
    let hovering = hovered.get();
    launch(hovering, async move {
        if !hovering {
            shown.set(false);
            return;
        }
        let mut delay = Delay::new(None, TOOLTIP_DELAY);
        run_frames(shown, move |_, now| !delay.elapsed(now)).await;
        shown.set(true);
    });
    let tip = text.clone();
    present_surface(
        shown.get(),
        anchor.get(),
        SurfaceStyle {
            placement: PopupPlacement::Below,
            modal: false,
            radius: Dp(5.0),
            padding: Dp(4.0),
            focus: false,
        },
        RenderSlot::new(move || {
            super::text::text(tip.clone()).style(TextStyle {
                font_size: Dp(11.0),
                line_height: Dp(14.0),
                ..Default::default()
            });
        }),
        Callback::new(move || shown.set(false)),
    );
    layout(Stack)
        .modifier(
            Modifier::new()
                .push_pointer_input(Hover(hovered))
                .push_semantics(Tooltip(text))
                .on_globally_positioned_precise(move |rect| {
                    if anchor.get() != rect {
                        anchor.set(rect);
                    }
                }),
        )
        .child(move || {
            if let Some(child) = child {
                child.render();
            }
        });
}

/// # popover
///
/// Shows content in a popover below its anchor while presented.
///
/// ## Usage
///
/// Offer a focused task or details, such as editing a field's options, without leaving the view.
///
/// ## Parameters
///
/// - `is_presented`, `on_dismiss` — whether the popover shows, and the request to close it after
///   an outside click or Escape.
/// - `content` — the popover's content.
/// - `child` — the anchor the popover points to.
///
/// The popover appears in the nearest
/// [`presentation_host`](super::presentation::presentation_host).
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{
///     button::button,
///     presentation::presentation_host,
///     presentations::popover,
///     text::text,
/// };
/// use tessera_ui::{remember, tessera};
/// # #[tessera]
/// # fn component() {
///
/// let shown = remember(|| false);
/// presentation_host().child(move || {
///     popover(shown.get())
///         .on_dismiss(move || shown.set(false))
///         .content(|| {
///             text("Details");
///         })
///         .child(move || {
///             button("Info").on_click(move || shown.set(true));
///         });
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn popover(
    is_presented: bool,
    on_dismiss: Option<Callback>,
    content: Option<RenderSlot>,
    child: Option<RenderSlot>,
) {
    let anchor = remember(|| FloatRect::ZERO);
    present_surface(
        is_presented,
        anchor.get(),
        SurfaceStyle {
            placement: PopupPlacement::Below,
            modal: false,
            radius: Dp(10.0),
            padding: Dp(12.0),
            focus: true,
        },
        content.unwrap_or_else(RenderSlot::empty),
        on_dismiss.unwrap_or_else(Callback::noop),
    );
    layout(Stack)
        .modifier(Modifier::new().on_globally_positioned_precise(move |rect| {
            if anchor.get() != rect {
                anchor.set(rect);
            }
        }))
        .child(move || {
            if let Some(child) = child {
                child.render();
            }
        });
}

/// One button of an [`alert`] or [`confirmation_dialog`].
#[derive(Clone, PartialEq)]
pub struct AlertAction {
    /// Button title.
    pub title: String,
    /// Purpose; destructive actions are red and cancel actions respond to Escape.
    pub role: Option<ButtonRole>,
    /// Action run before the alert closes.
    pub action: Callback,
}

impl AlertAction {
    /// Creates an action with a title and a callback.
    pub fn new(title: impl Into<String>, action: impl Fn() + 'static) -> Self {
        Self {
            title: title.into(),
            role: None,
            action: Callback::new(action),
        }
    }

    /// Gives the action a role.
    pub fn role(mut self, role: ButtonRole) -> Self {
        self.role = Some(role);
        self
    }
}

/// How serious the decision an [`alert`] asks for is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialogSeverity {
    /// The platform default, standard severity.
    #[default]
    Automatic,
    /// An ordinary alert.
    Standard,
    /// An alert about data loss or another serious consequence; shows a caution badge.
    Critical,
}

/// The suppression checkbox of an alert.
#[derive(Clone, PartialEq)]
struct Suppression {
    title: String,
    suppressed: bool,
    on_change: Option<CallbackWith<bool>>,
}

/// The parts of an alert other than its text and buttons.
#[derive(Clone, PartialEq)]
struct AlertExtras {
    icon: Option<RenderSlot>,
    critical: bool,
    suppression: Option<Suppression>,
}

/// # alert
///
/// Shows an important message with buttons in a modal panel.
///
/// ## Usage
///
/// Report an error or ask people to confirm an action that has consequences.
///
/// ## Parameters
///
/// - `title`, `is_presented` — the alert's title and whether it shows.
/// - `message` — optional explanation below the title.
/// - `actions` — the buttons; defaults to a single OK button. The last non-cancel action is
///   the default action for Return.
/// - `on_dismiss` — called after any action closes the alert, and on Escape.
/// - `icon` — optional symbol above the title.
/// - `severity` — critical alerts show a caution badge; defaults to [`DialogSeverity::Automatic`].
/// - `suppressed`, `on_suppressed_change` — shows a checkbox that lets people stop seeing the
///   alert, and its requested changes.
/// - `suppression_title` — the checkbox title; defaults to “Don’t ask again”.
///
/// The alert appears in the nearest
/// [`presentation_host`](super::presentation::presentation_host).
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{
///     button::ButtonRole,
///     presentation::presentation_host,
///     presentations::{AlertAction, alert},
/// };
/// use tessera_ui::{remember, tessera};
/// # #[tessera]
/// # fn component() {
///
/// let shown = remember(|| true);
/// presentation_host().child(move || {
///     alert("Delete “Report.pdf”?", shown.get())
///         .message("You can’t undo this action.")
///         .actions(vec![
///             AlertAction::new("Cancel", || {}).role(ButtonRole::Cancel),
///             AlertAction::new("Delete", || println!("deleted")).role(ButtonRole::Destructive),
///         ])
///         .on_dismiss(move || shown.set(false));
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn alert(
    #[prop(into)] title: String,
    is_presented: bool,
    #[prop(into)] message: Option<String>,
    actions: Option<Vec<AlertAction>>,
    on_dismiss: Option<Callback>,
    icon: Option<RenderSlot>,
    severity: Option<DialogSeverity>,
    suppressed: Option<bool>,
    on_suppressed_change: Option<CallbackWith<bool>>,
    #[prop(into)] suppression_title: Option<String>,
) {
    let extras = AlertExtras {
        icon,
        critical: severity == Some(DialogSeverity::Critical),
        suppression: suppressed.map(|suppressed| Suppression {
            title: suppression_title.unwrap_or_else(|| "Don’t ask again".to_owned()),
            suppressed,
            on_change: on_suppressed_change,
        }),
    };
    let dismiss = on_dismiss.unwrap_or_else(Callback::noop);
    let actions = actions.unwrap_or_else(|| vec![AlertAction::new("OK", || {})]);
    let cancel = actions
        .iter()
        .find(|action| action.role == Some(ButtonRole::Cancel))
        .map(|action| action.action);
    let escape = Callback::new(move || {
        if let Some(cancel) = cancel {
            cancel.call();
        }
        dismiss.call();
    });
    present_surface(
        is_presented,
        FloatRect::ZERO,
        SurfaceStyle {
            placement: PopupPlacement::Center,
            modal: true,
            radius: Dp(16.0),
            padding: Dp(16.0),
            focus: true,
        },
        RenderSlot::new(move || {
            alert_body(title.clone(), actions.clone(), dismiss, extras.clone())
                .message_optional(message.clone());
        }),
        escape,
    );
}

#[tessera]
fn alert_body(
    title: String,
    actions: Vec<AlertAction>,
    dismiss: Callback,
    extras: AlertExtras,
    message: Option<String>,
) {
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    let tint = if dark { Color::WHITE } else { Color::BLACK };
    let default = actions
        .iter()
        .rposition(|action| action.role != Some(ButtonRole::Cancel));
    column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().width(Dp(228.0)).semantics(SemanticsArgs {
            role: Some(Role::AlertDialog),
            label: Some(title.clone()),
            description: message.clone(),
            modal: true,
            ..Default::default()
        }))
        .children(move || {
            if extras.icon.is_some() || extras.critical {
                alert_icon(extras.critical).icon_optional(extras.icon);
                spacer().modifier(Modifier::new().height(Dp(10.0)));
            }
            text(title.clone())
                .style(TextStyle {
                    font_weight: 700,
                    ..Default::default()
                })
                .color(tint.with_alpha(0.85));
            if let Some(message) = message.clone() {
                spacer().modifier(Modifier::new().height(Dp(6.0)));
                text(message).color(tint.with_alpha(0.85));
            }
            if let Some(suppression) = extras.suppression.clone() {
                spacer().modifier(Modifier::new().height(Dp(12.0)));
                let checkbox = toggle(suppression.suppressed)
                    .style(ToggleStyle::Checkbox)
                    .label(suppression.title);
                if let Some(on_change) = suppression.on_change {
                    checkbox.on_value_change_shared(on_change);
                }
            }
            spacer().modifier(Modifier::new().height(Dp(16.0)));
            let actions = actions.clone();
            button_scope().child(move |scope| {
                alert_buttons(actions.clone(), dismiss, scope).default_optional(default);
            });
        });
}

/// The alert's symbol; a critical alert adds a caution badge at its bottom-trailing corner, or
/// shows the badge alone.
#[tessera]
fn alert_icon(critical: bool, icon: Option<RenderSlot>) {
    let badge = if icon.is_some() { 20.0 } else { 36.0 };
    boxed().alignment(Alignment::BottomEnd).children(move || {
        if let Some(icon) = icon {
            icon.render();
        }
        if critical {
            caution_badge(badge);
        }
    });
}

#[tessera]
fn caution_badge(size: f64) {
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .size(Dp(size), Dp(size))
                .background_with_shape(caution_yellow(), Shape::Ellipse)
                .clear_and_set_semantics(SemanticsArgs {
                    hidden: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            text("!")
                .style(TextStyle {
                    font_size: Dp(size * 0.6),
                    line_height: Dp(size * 0.75),
                    font_weight: 800,
                    ..Default::default()
                })
                .color(Color::BLACK.with_alpha(0.85));
        });
}

fn caution_yellow() -> Color {
    // sRGB 255, 204, 0.
    Color::from_rgb(1.0, 0.6038, 0.0)
}

#[tessera]
fn alert_buttons(
    actions: Vec<AlertAction>,
    dismiss: Callback,
    scope: ButtonScope,
    default: Option<usize>,
) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            for (index, action) in actions.iter().cloned().enumerate() {
                if index > 0 {
                    spacer().modifier(Modifier::new().height(Dp(8.0)));
                }
                let callback = action.action;
                button(action.title.clone())
                    .role_optional(action.role)
                    .style(if Some(index) == default {
                        ButtonStyle::BorderedProminent
                    } else {
                        ButtonStyle::Bordered
                    })
                    .control_size(super::theme::ControlSize::Large)
                    .default_action(Some(index) == default)
                    .scope(scope)
                    .modifier(Modifier::new().fill_max_width())
                    .on_click(move || {
                        callback.call();
                        dismiss.call();
                    });
            }
        });
}

/// # confirmation_dialog
///
/// Asks people to confirm an action, with a Cancel button added after the given actions.
///
/// ## Usage
///
/// Confirm deleting, discarding, or replacing content before it happens.
///
/// ## Parameters
///
/// - `title`, `is_presented` — the dialog's title and whether it shows.
/// - `message` — optional explanation below the title.
/// - `actions` — the confirming buttons.
/// - `on_dismiss` — called after any action closes the dialog, including Cancel and Escape.
/// - `icon`, `severity`, `suppressed`, `on_suppressed_change`, `suppression_title` — as for
///   [`alert`].
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{
///     button::ButtonRole,
///     presentation::presentation_host,
///     presentations::{AlertAction, confirmation_dialog},
/// };
/// use tessera_ui::{remember, tessera};
/// # #[tessera]
/// # fn component() {
///
/// let shown = remember(|| true);
/// presentation_host().child(move || {
///     confirmation_dialog("Discard changes?", shown.get())
///         .actions(vec![
///             AlertAction::new("Discard", || println!("discarded")).role(ButtonRole::Destructive),
///         ])
///         .on_dismiss(move || shown.set(false));
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn confirmation_dialog(
    #[prop(into)] title: String,
    is_presented: bool,
    #[prop(into)] message: Option<String>,
    actions: Option<Vec<AlertAction>>,
    on_dismiss: Option<Callback>,
    icon: Option<RenderSlot>,
    severity: Option<DialogSeverity>,
    suppressed: Option<bool>,
    on_suppressed_change: Option<CallbackWith<bool>>,
    #[prop(into)] suppression_title: Option<String>,
) {
    let mut actions = actions.unwrap_or_default();
    if !actions
        .iter()
        .any(|action| action.role == Some(ButtonRole::Cancel))
    {
        actions.push(AlertAction::new("Cancel", || {}).role(ButtonRole::Cancel));
    }
    let alert = alert(title, is_presented)
        .message_optional(message)
        .actions(actions)
        .on_dismiss_optional(on_dismiss)
        .severity_optional(severity)
        .suppressed_optional(suppressed)
        .suppression_title_optional(suppression_title);
    if let Some(icon) = icon {
        alert.icon_shared(icon);
    }
    if let Some(on_change) = on_suppressed_change {
        alert.on_suppressed_change_shared(on_change);
    }
}

/// # sheet
///
/// Shows a modal task in a panel at the top of the window.
///
/// ## Usage
///
/// Collect information for a focused task, such as saving or exporting, before returning.
///
/// ## Parameters
///
/// - `is_presented`, `on_dismiss` — whether the sheet shows, and the request to close it on
///   Escape; outside clicks leave the sheet open.
/// - `content` — the sheet's content, usually ending with its buttons.
///
/// The sheet appears in the nearest
/// [`presentation_host`](super::presentation::presentation_host).
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{
///     button::button,
///     presentation::presentation_host,
///     presentations::sheet,
///     text::text,
/// };
/// use tessera_ui::{remember, tessera};
/// # #[tessera]
/// # fn component() {
///
/// let shown = remember(|| true);
/// presentation_host().child(move || {
///     sheet(shown.get())
///         .on_dismiss(move || shown.set(false))
///         .content(move || {
///             text("Export as PDF");
///             button("Done").on_click(move || shown.set(false));
///         });
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn sheet(is_presented: bool, on_dismiss: Option<Callback>, content: Option<RenderSlot>) {
    let content = content.unwrap_or_else(RenderSlot::empty);
    present_surface(
        is_presented,
        FloatRect::ZERO,
        SurfaceStyle {
            placement: PopupPlacement::TopCenter,
            modal: true,
            radius: Dp(16.0),
            padding: Dp(20.0),
            focus: true,
        },
        RenderSlot::new(move || sheet_body(content)),
        on_dismiss.unwrap_or_else(Callback::noop),
    );
}

#[tessera]
fn sheet_body(content: RenderSlot) {
    column()
        .modifier(Modifier::new().width(Dp(400.0)).semantics(SemanticsArgs {
            role: Some(Role::Dialog),
            modal: true,
            ..Default::default()
        }))
        .children_shared(content);
}

/// Places the content and, when visible, the inspector at `fraction` of its width.
#[derive(Clone, Copy, PartialEq)]
struct InspectorLayout(f32);

impl LayoutPolicy for InspectorLayout {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = *scope.parent_constraint().as_ref();
        let width = parent.width.resolve_max().unwrap_or(Px::from(Dp(600.0)));
        let height = parent.height.resolve_max().unwrap_or(Px::from(Dp(400.0)));
        let full = Px::from(Dp(INSPECTOR_WIDTH));
        let shown = Px((full.to_f32() * self.0).round() as i32).min(width);
        let children = scope.children();
        let mut result = LayoutResult::new(ComputedData { width, height });
        if let Some(content) = children.first() {
            content.measure(
                scope,
                &Constraint::new(
                    AxisConstraint::exact(width - shown),
                    AxisConstraint::exact(height),
                ),
            )?;
            result.place_child(*content, PxPosition::ZERO);
        }
        if let Some(inspector) = children.get(1) {
            inspector.measure(
                scope,
                &Constraint::new(AxisConstraint::exact(full), AxisConstraint::exact(height)),
            )?;
            result.place_child(*inspector, PxPosition::new(width - shown, Px::ZERO));
        }
        Ok(result)
    }
}

/// # inspector
///
/// Shows an inspector column at the trailing edge of its content while presented.
///
/// ## Usage
///
/// Show and edit the properties of the current selection beside the main content.
///
/// ## Parameters
///
/// - `is_presented` — whether the inspector shows; it slides in and out.
/// - `content` — the inspector's content.
/// - `child` — the main content, which narrows to make room.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{presentations::inspector, text::text};
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// inspector(true)
///     .content(|| {
///         text("Font: SF Pro");
///     })
///     .child(|| {
///         text("Document");
///     });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn inspector(is_presented: bool, content: Option<RenderSlot>, child: Option<RenderSlot>) {
    let target = if is_presented { 1.0 } else { 0.0 };
    let shown = remember(|| Eased::new(target, INSPECTOR_MOTION));
    launch(is_presented, animate_to(shown, target));
    let fraction = shown.with(Eased::value);
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    layout(InspectorLayout(fraction))
        .modifier(Modifier::new().clip_to_bounds())
        .child(move || {
            inspector_pane(child.unwrap_or_else(RenderSlot::empty));
            if fraction > 0.0 {
                inspector_panel(content.unwrap_or_else(RenderSlot::empty), dark);
            }
        });
}

#[tessera]
fn inspector_pane(content: RenderSlot) {
    layout(Stack).child(content);
}

#[tessera]
fn inspector_panel(content: RenderSlot, dark: bool) {
    column()
        .main_axis_alignment(MainAxisAlignment::Start)
        .modifier(
            Modifier::new()
                .fill_max_size()
                .background(if dark {
                    Color::from_rgb(0.02, 0.02, 0.02)
                } else {
                    Color::from_rgb(0.93, 0.93, 0.93)
                })
                .padding_all(Dp(12.0))
                .semantics(SemanticsArgs {
                    role: Some(Role::Complementary),
                    ..Default::default()
                }),
        )
        .children_shared(content);
}
