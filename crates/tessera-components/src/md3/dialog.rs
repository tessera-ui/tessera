//! Material 3 dialogs for alerts, confirmations, and focused tasks.
//!
//! ## Usage
//!
//! Show blocking alerts, confirmations, wizards and forms.

use tessera_foundation::{
    modifier::{ModifierExt as _, SemanticsArgs},
    pointer_gestures::{TapGesture, TapGestureConfig, detect_tap_gestures},
};
use tessera_ui::{
    AxisConstraint, Callback, Color, ComputedData, Constraint, Dp, FocusTraversalPolicy,
    LayoutResult, MeasurementError, Modifier, Px, PxPosition, RenderSlot, State,
    accesskit::Role,
    focus, launch,
    layout::{MeasureScope, Stack, layout},
    modifier::FocusModifierExt as _,
    popup::{WindowOrigin, popup},
    provide_context, remember, tessera,
    time::Instant,
    use_context,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    animation::{
        Motion, run_frames,
        visibility_motion::{VisibilityMotion, VisibilitySample},
    },
    boxed::boxed,
    column::column,
    flow_row::flow_row,
    md3::{
        app_bar::top_app_bar,
        button::button,
        icon_button::{IconButtonColorStyle, icon_button},
        material_icons::filled,
        modifier::ModifierExt as _,
        scrollable::ScrollableController,
        surface::surface,
        text::text,
        theme::{ContentColor, MaterialTheme},
    },
    res::I18N,
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
};

/// The purpose announced for a modal dialog.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialogRole {
    /// A modal task, form, or informational dialog.
    #[default]
    Dialog,
    /// An urgent message that requires an immediate response.
    AlertDialog,
}

impl DialogRole {
    fn accessibility_role(self) -> Role {
        match self {
            Self::Dialog => Role::Dialog,
            Self::AlertDialog => Role::AlertDialog,
        }
    }
}

/// Controller for [`dialog`], controlling visibility and animation.
pub struct DialogController {
    is_open: bool,
    motion: VisibilityMotion,
    requests: u32,
}

impl DialogController {
    /// Creates a new dialog controller.
    pub fn new(initial_open: bool) -> Self {
        Self {
            is_open: initial_open,
            motion: VisibilityMotion::new(initial_open, false),
            requests: 0,
        }
    }

    /// Opens the dialog, starting the animation if necessary.
    pub fn open(&mut self) {
        if !self.is_open {
            self.motion.request(true);
            self.is_open = true;
            self.requests = self.requests.wrapping_add(1);
        }
    }

    /// Closes the dialog, starting the closing animation if necessary.
    pub fn close(&mut self) {
        if self.is_open {
            self.motion.request(false);
            self.is_open = false;
            self.requests = self.requests.wrapping_add(1);
        }
    }

    /// Returns whether the dialog is currently open.
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    fn advance(&mut self, now: Instant) -> bool {
        self.motion.advance(now)
    }

    fn is_animating(&self) -> bool {
        self.motion.sample().active
    }

    fn snapshot(&self) -> (bool, VisibilitySample) {
        (self.is_open, self.motion.sample())
    }
}

impl Default for DialogController {
    fn default() -> Self {
        Self::new(false)
    }
}

#[tessera]
fn dialog_content_wrapper(
    accessibility_label: Option<String>,
    accessibility_description: Option<String>,
    role: Option<DialogRole>,
    alpha: Option<f32>,
    padding: Option<Dp>,
    content: Option<RenderSlot>,
    full_screen: Option<bool>,
) {
    let full_screen = full_screen.unwrap_or(false);
    let alpha = alpha.unwrap_or(1.0);
    let padding = padding.unwrap_or(Dp(24.0));
    let modifier = Modifier::new()
        .focus_group(None)
        .focus_traversal_policy(
            FocusTraversalPolicy::linear()
                .wrap(true)
                .tab_navigation(true),
        )
        .semantics(SemanticsArgs {
            role: Some(role.unwrap_or_default().accessibility_role()),
            label: accessibility_label,
            description: accessibility_description,
            modal: true,
            focusable: true,
            merge_descendants: true,
            ..Default::default()
        });
    let policy = DialogContentLayout { alpha };
    tessera_ui::layout::layout(policy.clone())
        .draw(policy)
        .child(move || {
            let content = content;
            let modifier = modifier.clone();
            boxed()
                .modifier(Modifier::new().fill_max_size())
                .alignment(Alignment::Center)
                .children(move || {
                    let content = content;
                    let modifier = modifier.clone();
                    surface()
                        .style(Color::TRANSPARENT.into())
                        .modifier(
                            Modifier::new()
                                .constrain(Some(AxisConstraint::NONE), Some(AxisConstraint::NONE))
                                .padding_all(if full_screen { Dp::ZERO } else { Dp(24.0) }),
                        )
                        .child(move || {
                            let content = content;
                            surface()
                                .modifier(if full_screen {
                                    modifier.clone().width(Dp(560.0)).fill_max_height()
                                } else {
                                    modifier.clone()
                                })
                                .style(
                                    use_context::<MaterialTheme>()
                                        .expect("MaterialTheme must be provided")
                                        .get()
                                        .color_scheme
                                        .surface_container_high
                                        .into(),
                                )
                                .elevation(if full_screen { Dp::ZERO } else { Dp(6.0) })
                                .shape(if full_screen {
                                    Shape::RECTANGLE
                                } else {
                                    Shape::RoundedRectangle {
                                        top_left: RoundedCorner::manual(Dp(28.0), 3.0),
                                        top_right: RoundedCorner::manual(Dp(28.0), 3.0),
                                        bottom_right: RoundedCorner::manual(Dp(28.0), 3.0),
                                        bottom_left: RoundedCorner::manual(Dp(28.0), 3.0),
                                    }
                                })
                                .block_input(true)
                                .child(move || {
                                    let content = content;
                                    layout(Stack)
                                        .modifier(Modifier::new().padding_all(padding))
                                        .child(move || {
                                            if let Some(content) = content {
                                                content.render();
                                            }
                                        });
                                });
                        });
                });
        });
}

#[derive(Clone, PartialEq)]
struct DialogContentLayout {
    alpha: f32,
}

impl tessera_ui::LayoutPolicy for DialogContentLayout {
    fn measure_eq(&self, _other: &Self) -> bool {
        true
    }

    fn placement_eq(&self, _other: &Self) -> bool {
        true
    }

    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let Some(child) = input.children().first().copied() else {
            return Ok(result.with_size(ComputedData {
                width: Px(0),
                height: Px(0),
            }));
        };
        let child_constraint = input.parent_constraint().without_min();
        let computed = child.measure(input, &child_constraint)?;
        result.place_child(child, PxPosition::ZERO);
        Ok(result.with_size(computed.size()))
    }
}

impl tessera_ui::DrawModifierNode for DialogContentLayout {
    fn draw(
        &self,
        ctx: &mut tessera_ui::DrawModifierContext<'_, '_>,
        content: &mut dyn tessera_ui::DrawModifierContent,
    ) {
        let input = &mut *ctx.render_input;
        input.metadata_mut().multiply_opacity(self.alpha);
        content.draw(input);
    }
}

/// # dialog
///
/// Show modal content above the current screen with Material 3 dialog styling.
/// Use for alerts, confirmations, forms, and full-screen tasks.
///
/// ## Usage
///
/// Present a decision or focused task without navigating away from the current screen.
///
/// ## Parameters
///
/// - `on_dismiss_request` — callback for Escape, back, outside presses, and the scrim.
/// - `accessibility_label` — optional accessible name; typically the title shown in the body.
/// - `accessibility_description` — optional concise description for custom content.
/// - `role` — dialog purpose; defaults to `DialogRole::Dialog`.
/// - `padding` — padding inside the dialog surface.
/// - `full_screen` — fill the window height, with a maximum width of 560dp.
/// - `controller` — optional visibility and motion controller. Without one, the dialog is
///   visible whenever it is declared; stop declaring it to dismiss it.
/// - `content` — dialog body.
///
/// ## Examples
///
/// ```
/// use tessera_components::md3::{
///     dialog::{DialogController, basic_dialog, dialog},
///     theme::material_theme,
/// };
/// use tessera_ui::{remember, tessera};
/// #[tessera]
/// fn example() {
///     material_theme().child(|| {
///         let open = remember(|| DialogController::new(true));
///         dialog(move || open.with_mut(DialogController::close))
///             .controller(open)
///             .accessibility_label("Reset settings?")
///             .content(|| {
///                 basic_dialog().headline("Reset settings?");
///             });
///     });
/// }
/// ```
#[tessera]
pub fn dialog(
    on_dismiss_request: Callback,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    role: Option<DialogRole>,
    padding: Option<Dp>,
    full_screen: Option<bool>,
    controller: Option<State<DialogController>>,
    content: Option<RenderSlot>,
) {
    let controller = if let Some(controller) = controller {
        controller
    } else {
        remember(|| DialogController::new(true))
    };
    let full_screen = full_screen.unwrap_or(false);
    let padding = padding.unwrap_or(if full_screen { Dp::ZERO } else { Dp(24.0) });
    let (is_open, visibility) = controller.with(DialogController::snapshot);
    let animating = controller.with(DialogController::is_animating);
    if animating {
        launch(
            controller.with(|controller| controller.requests),
            run_frames(controller, DialogController::advance),
        );
    }
    if !is_open && !animating {
        return;
    }
    let alpha = visibility.opacity;
    let scrim_alpha = alpha * 0.32;
    let dismiss = Callback::new(move || {
        if controller.with(DialogController::is_open) {
            on_dismiss_request.call();
        }
    });
    popup(WindowOrigin)
        .modal(true)
        .on_dismiss_request(dismiss)
        .child(move || {
            let accessibility_label = accessibility_label.clone();
            let accessibility_description = accessibility_description.clone();
            let scrim = use_context::<MaterialTheme>()
                .expect("MaterialTheme must be provided")
                .get()
                .color_scheme
                .scrim;
            surface()
                .style(scrim.with_alpha(scrim_alpha).into())
                .modifier(Modifier::new().fill_max_size().pointer_input(
                    dismiss,
                    move |mut scope| async move {
                        detect_tap_gestures(&mut scope, TapGestureConfig::default(), |gesture| {
                            if matches!(gesture, TapGesture::Tapped(_, _)) {
                                dismiss.call();
                                focus::defer_keyboard_until_layout();
                            }
                        })
                        .await;
                    },
                ))
                .child(|| {});
            dialog_content_wrapper()
                .accessibility_label_optional(accessibility_label)
                .accessibility_description_optional(accessibility_description)
                .role(role.unwrap_or_default())
                .alpha(alpha)
                .padding(padding)
                .full_screen(full_screen)
                .content_optional(content);
        });
}
/// # full_screen_dialog
///
/// Present a scrollable form with a close action and optional save action.
///
/// ## Usage
///
/// Edit information or complete a task that needs the available screen height.
///
/// ## Parameters
///
/// - `on_close` — callback invoked by the close button.
/// - `modifier` — modifier chain applied to the content surface.
/// - `title` — optional headline in the top bar.
/// - `confirm_label` — text for the optional confirm action, defaulting to
///   Save.
/// - `on_confirm` — optional callback; omitting it hides the confirm action.
/// - `content` — scrollable form or task content.
///
/// ## Examples
///
/// ```
/// use tessera_components::md3::{
///     dialog::{DialogController, dialog, full_screen_dialog},
///     text::text,
///     theme::material_theme,
/// };
/// use tessera_ui::{Callback, remember, tessera};
/// #[tessera]
/// fn example() {
///     material_theme().child(|| {
///         let open = remember(|| DialogController::new(true));
///         let close = Callback::new(move || open.with_mut(DialogController::close));
///         dialog(close)
///             .controller(open)
///             .full_screen(true)
///             .accessibility_label("New event")
///             .content(move || {
///                 full_screen_dialog(close).title("New event").content(|| {
///                     text("Event details");
///                 });
///             });
///     });
/// }
/// ```
#[tessera]
pub fn full_screen_dialog(
    on_close: Callback,
    modifier: Option<Modifier>,
    #[prop(into)] title: Option<String>,
    #[prop(into)] confirm_label: Option<String>,
    on_confirm: Option<Callback>,
    content: Option<RenderSlot>,
) {
    let title = title.unwrap_or_default();
    let confirm_label = confirm_label.unwrap_or_else(|| I18N.text("md3-save"));
    let content = content.unwrap_or_else(RenderSlot::empty);
    let scroll = remember(ScrollableController::new);
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    surface()
        .modifier(modifier.unwrap_or_default().fill_max_size())
        .style(scheme.surface_container_high.into())
        .shape(Shape::RECTANGLE)
        .content_color(scheme.on_surface)
        .child(move || {
            let title = title.clone();
            let confirm_label = confirm_label.clone();
            column()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    let bar = top_app_bar()
                        .title(title.clone())
                        .container_color(scheme.surface_container_high)
                        .navigation_icon(move || {
                            icon_button()
                                .standard()
                                .color_style(IconButtonColorStyle::Inherited)
                                .icon(filled::CLOSE_SVG)
                                .accessibility_label(I18N.text("md3-close-dialog"))
                                .on_click_shared(on_close);
                        });
                    if let Some(on_confirm) = on_confirm {
                        let confirm_label = confirm_label.clone();
                        bar.action(move || {
                            let confirm_label = confirm_label.clone();
                            button()
                                .text()
                                .content_color(scheme.primary)
                                .ripple_color(scheme.primary)
                                .accessibility_label(confirm_label.clone())
                                .on_click_shared(on_confirm)
                                .child(move || {
                                    text(confirm_label.clone());
                                });
                        });
                    }
                    column()
                        .modifier(
                            Modifier::new()
                                .fill_max_width()
                                .weight(1.0)
                                .vertical_scroll(scroll)
                                .padding_all(Dp(24.0)),
                        )
                        .children_shared(content);
                });
        });
}

/// # basic_dialog
///
/// Present a brief confirmation or alert with a scrollable message body.
///
/// ## Usage
///
/// Use as the content of [`dialog`].
///
/// ## Parameters
///
/// - `icon` — optional icon slot shown above the headline.
/// - `headline` — optional headline text.
/// - `supporting_text` — body text shown in the dialog.
/// - `content` — optional custom body replacing the supporting text.
///   Overflowing content scrolls while the headline and actions remain visible.
/// - `confirm_button` — optional confirm button slot.
/// - `dismiss_button` — optional dismiss button slot.
///
/// Use inside [`dialog`].
#[tessera]
pub fn basic_dialog(
    icon: Option<RenderSlot>,
    #[prop(into)] headline: Option<String>,
    #[prop(into)] supporting_text: Option<String>,
    content: Option<RenderSlot>,
    confirm_button: Option<RenderSlot>,
    dismiss_button: Option<RenderSlot>,
) {
    let supporting_text = supporting_text.unwrap_or_default();
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let scheme = theme.color_scheme;
    let alignment = if icon.is_some() {
        CrossAxisAlignment::Center
    } else {
        CrossAxisAlignment::Start
    };
    let scroll = remember(ScrollableController::new);

    tessera_ui::layout::layout(BasicDialogLayout {
        center: icon.is_some(),
    })
    .modifier(Modifier::new().constrain(
        Some(AxisConstraint::new(
            Dp(232.0).into(),
            Some(Dp(512.0).into()),
        )),
        Some(AxisConstraint::NONE),
    ))
    .child(move || {
        let headline = headline.clone();
        let supporting_text = supporting_text.clone();
        column().cross_axis_alignment(alignment).children(move || {
            if let Some(icon) = icon.as_ref() {
                let icon = *icon;
                let icon_color = scheme.secondary;
                {
                    provide_context(
                        ContentColor {
                            current: icon_color,
                        },
                        || {
                            icon.render();
                        },
                    );
                };
                {
                    spacer().modifier(Modifier::new().height(Dp(16.0)));
                };
            }

            if let Some(headline) = headline.as_ref() {
                let headline = headline.clone();
                {
                    text(headline.clone())
                        .style(theme.typography.headline_small)
                        .color(scheme.on_surface);
                };
                {
                    spacer().modifier(Modifier::new().height(Dp(16.0)));
                };
            }
        });

        column()
            .modifier(Modifier::new().vertical_scroll(scroll))
            .cross_axis_alignment(alignment)
            .children(move || {
                if let Some(content) = content {
                    content.render();
                } else {
                    text(supporting_text.clone())
                        .style(theme.typography.body_medium)
                        .color(scheme.on_surface_variant);
                }
            });

        column().children(move || {
            if confirm_button.is_some() || dismiss_button.is_some() {
                {
                    spacer().modifier(Modifier::new().height(Dp(24.0)));
                };
                let action_color = scheme.primary;
                {
                    provide_context(
                        ContentColor {
                            current: action_color,
                        },
                        || {
                            flow_row()
                                .modifier(Modifier::new().fill_max_width())
                                .main_axis_alignment(MainAxisAlignment::End)
                                .item_spacing(Dp(8.0))
                                .line_spacing(Dp(8.0))
                                .children(move || {
                                    if let Some(dismiss) = dismiss_button.as_ref() {
                                        let dismiss = *dismiss;
                                        dismiss.render();
                                    }

                                    if let Some(confirm) = confirm_button.as_ref() {
                                        let confirm = *confirm;
                                        confirm.render();
                                    }
                                });
                        },
                    );
                };
            }
        });
    });
}

#[derive(Clone, PartialEq)]
struct BasicDialogLayout {
    center: bool,
}

impl tessera_ui::LayoutPolicy for BasicDialogLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let children = input.children();
        let (Some(header), Some(body), Some(actions)) = (
            children.first().copied(),
            children.get(1).copied(),
            children.get(2).copied(),
        ) else {
            return Ok(LayoutResult::default());
        };
        let parent = input.parent_constraint().without_min();
        let actions_size = actions.measure(input, &parent)?;
        let header_constraint = Constraint::new(
            parent.width,
            AxisConstraint::new(
                Px::ZERO,
                parent
                    .height
                    .resolve_max()
                    .map(|height| (height - actions_size.height).max(Px::ZERO)),
            ),
        );
        let header_size = header.measure(input, &header_constraint)?;
        let body_constraint = Constraint::new(
            parent.width,
            AxisConstraint::new(
                Px::ZERO,
                parent.height.resolve_max().map(|height| {
                    (height - actions_size.height - header_size.height).max(Px::ZERO)
                }),
            ),
        );
        let mut body_size = body.measure(input, &body_constraint)?;
        let width = input.parent_constraint().width().clamp(
            header_size
                .width
                .max(body_size.width)
                .max(actions_size.width),
        );
        if body_size.width != width {
            body_size = body.measure(
                input,
                &Constraint::new(AxisConstraint::exact(width), body_constraint.height),
            )?;
        }
        let height = input
            .parent_constraint()
            .height()
            .clamp(header_size.height + body_size.height + actions_size.height);
        let x = |child_width: Px| {
            if self.center {
                (width.0 - child_width.0) as f32 * 0.5
            } else {
                0.0
            }
        };
        let mut result = LayoutResult::default();
        result.place_child(header, PxPosition::from_f32_xy(x(header_size.width), 0.0));
        result.place_child(
            body,
            PxPosition::from_f32_xy(x(body_size.width), header_size.height.to_f32()),
        );
        result.place_child(
            actions,
            PxPosition::from_f32_xy(
                x(actions_size.width),
                (header_size.height + body_size.height).to_f32(),
            ),
        );
        Ok(result.with_size(ComputedData { width, height }))
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod action_tests;
