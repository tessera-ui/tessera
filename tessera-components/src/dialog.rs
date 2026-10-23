//! Modal dialog provider — show modal content above the main app UI.
//!
//! ## Usage
//!
//! Show blocking alerts, confirmations, wizards and forms.

use tessera_ui::{
    AxisConstraint, Callback, Color, ComputedData, Dp, FocusProperties, FocusScopeNode,
    FocusTraversalPolicy, LayoutResult, MeasurementError, Modifier, Px, PxPosition, RenderSlot,
    State, current_frame_nanos,
    layout::{MeasureScope, layout},
    modifier::FocusModifierExt as _,
    provide_context, receive_frame_nanos, remember, tessera, use_context, winit,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    animation::visibility_motion::{VisibilityMotion, VisibilitySample},
    app_bar::top_app_bar,
    boxed::boxed,
    button::button,
    column::column,
    icon_button::{IconButtonColorStyle, icon_button},
    material_icons::filled,
    modal_focus::{ModalFocusState, modal_focus_modifier},
    modifier::{ModifierExt, SemanticsArgs, with_keyboard_input},
    overlay_semantics::{dialog_semantics, overlay_background},
    row::row,
    scrollable::ScrollableController,
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{ContentColor, MaterialTheme},
};

/// Controller for [`dialog_provider`], controlling visibility and animation.
pub struct DialogController {
    is_open: bool,
    motion: VisibilityMotion,
}

impl DialogController {
    /// Creates a new dialog controller.
    pub fn new(initial_open: bool) -> Self {
        Self {
            is_open: initial_open,
            motion: VisibilityMotion::new(initial_open, false),
        }
    }

    /// Opens the dialog, starting the animation if necessary.
    pub fn open(&mut self) {
        if !self.is_open {
            self.motion.retarget(true, current_frame_nanos());
            self.is_open = true;
        }
    }

    /// Closes the dialog, starting the closing animation if necessary.
    pub fn close(&mut self) {
        if self.is_open {
            self.motion.retarget(false, current_frame_nanos());
            self.is_open = false;
        }
    }

    /// Returns whether the dialog is currently open.
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    fn is_animating(&self) -> bool {
        self.motion.sample(current_frame_nanos()).active
    }

    fn snapshot(&self) -> (bool, VisibilitySample) {
        (self.is_open, self.motion.sample(current_frame_nanos()))
    }
}

impl Default for DialogController {
    fn default() -> Self {
        Self::new(false)
    }
}

fn make_keyboard_handler(
    on_close: Callback,
) -> Box<dyn for<'a> Fn(tessera_ui::KeyboardInput<'a>) + Send + Sync + 'static> {
    Box::new(move |mut input| {
        let mut handled = false;
        input.keyboard_events.retain(|event| {
            if event.state != winit::event::ElementState::Pressed {
                return true;
            }

            if let winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) =
                event.physical_key
            {
                on_close.call();
                handled = true;
                return false;
            }

            true
        });
        if handled {
            input.block_keyboard();
        }
    })
}

#[tessera]
fn dialog_content_wrapper(
    accessibility_label: Option<String>,
    alpha: Option<f32>,
    padding: Option<Dp>,
    just_opened: Option<bool>,
    on_close_request: Option<Callback>,
    content: Option<RenderSlot>,
    focus_scope: Option<FocusScopeNode>,
    full_screen: Option<bool>,
) {
    let full_screen = full_screen.unwrap_or(false);
    let alpha = alpha.unwrap_or(1.0);
    let padding = padding.unwrap_or(Dp(24.0));
    let just_opened = just_opened.unwrap_or(false);
    let on_close_request = on_close_request.unwrap_or_default();
    let content = content.expect("dialog_content_wrapper requires content");
    let focus_scope = focus_scope.expect("modal content requires its focus scope");
    let modifier = dialog_semantics(
        with_keyboard_input(
            Modifier::new()
                .focus_scope_with(focus_scope)
                .focus_properties(FocusProperties::new().skip_traversal(true))
                .focus_traversal_policy(
                    FocusTraversalPolicy::linear()
                        .wrap(true)
                        .tab_navigation(true),
                ),
            make_keyboard_handler(on_close_request),
        ),
        accessibility_label,
        true,
    );
    if just_opened {
        focus_scope.request_focus();
    }
    let policy = DialogContentLayout { alpha };
    layout().policy(policy.clone()).draw(policy).child(move || {
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
                                layout()
                                    .modifier(Modifier::new().padding_all(padding))
                                    .child(move || {
                                        content.render();
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

impl tessera_ui::LayoutModifierNode for DialogContentLayout {
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

/// # dialog_provider
///
/// Provide a modal dialog at the top level of an application.
///
/// ## Usage
///
/// Show modal content for alerts, confirmation dialogs, multi-step forms, or
/// onboarding steps that require blocking user interaction with the main UI.
///
/// ## Parameters
///
/// - `accessibility_label` — accessible name describing the dialog or sheet.
/// - `on_close_request` — optional close-request callback.
/// - `padding` — padding applied inside the dialog surface.
/// - `full_screen` — fill the available height without outer margins or rounded
///   corners; width remains limited to 560dp.
/// - `is_open` — declarative open state.
/// - `controller` — optional external controller.
/// - `main_content` — optional main content rendered behind the dialog.
/// - `dialog_content` — optional modal content rendered above the scrim.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::dialog::{basic_dialog, dialog_provider};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// dialog_provider()
///     .accessibility_label("Dialog Title")
///     .on_close_request(|| {})
///     .is_open(true)
///     .main_content(|| { /* main content */ })
///     .dialog_content(|| {
///         basic_dialog()
///             .supporting_text("This is the dialog body text.")
///             .headline("Dialog Title");
///     });
/// #     });
/// # }
/// # component();
/// ```
#[tessera]
pub fn dialog_provider(
    #[prop(into)] accessibility_label: Option<String>,
    on_close_request: Option<Callback>,
    padding: Option<Dp>,
    full_screen: Option<bool>,
    is_open: Option<bool>,
    #[prop(skip_setter)] controller: Option<State<DialogController>>,
    main_content: Option<RenderSlot>,
    dialog_content: Option<RenderSlot>,
) {
    let on_close_request = on_close_request.unwrap_or_default();
    let full_screen = full_screen.unwrap_or(false);
    let padding = padding.unwrap_or(if full_screen { Dp::ZERO } else { Dp(24.0) });
    let is_open = is_open.unwrap_or(false);
    let main_content = main_content.unwrap_or_else(RenderSlot::empty);
    let dialog_content = dialog_content.unwrap_or_else(RenderSlot::empty);
    let external_controller = controller;
    let controller =
        external_controller.unwrap_or_else(|| remember(|| DialogController::new(is_open)));

    // In controlled mode (external controller provided), do not override
    // controller state from `is_open`.
    if external_controller.is_none() {
        let current_open = controller.with(|c| c.is_open());
        if is_open != current_open {
            if is_open {
                controller.with_mut(|c| c.open());
            } else {
                controller.with_mut(|c| c.close());
            }
        }
    }
    dialog_provider_render()
        .accessibility_label_optional(accessibility_label)
        .on_close_request_shared(on_close_request)
        .padding(padding)
        .full_screen(full_screen)
        .controller(controller)
        .main_content_shared(main_content)
        .dialog_content_shared(dialog_content);
}

#[tessera]
fn dialog_provider_render(
    accessibility_label: Option<String>,
    on_close_request: Option<Callback>,
    padding: Option<Dp>,
    full_screen: Option<bool>,
    controller: Option<State<DialogController>>,
    main_content: Option<RenderSlot>,
    dialog_content: Option<RenderSlot>,
) {
    let on_close_request = on_close_request.unwrap_or_default();
    let padding = padding.unwrap_or(Dp(24.0));
    let controller = controller.expect("dialog_provider_render requires controller");
    let main_content = main_content.unwrap_or_else(RenderSlot::empty);
    let dialog_content = dialog_content.unwrap_or_else(RenderSlot::empty);
    let dialog_open_state = remember(|| false);

    // If the dialog is open, render the modal overlay.
    // Sample state once to avoid repeated locks and improve readability.
    let (is_open, visibility) = controller.with(|c| c.snapshot());
    let mut just_opened = false;
    dialog_open_state.with_mut(|was_open| {
        just_opened = !*was_open && is_open;
        *was_open = is_open;
    });

    let is_animating = controller.with(|c| c.is_animating());
    if is_animating {
        receive_frame_nanos(move |frame_nanos| {
            let is_animating =
                controller.with(|controller| controller.motion.sample(frame_nanos).active);
            if is_animating {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }

    let focus_scope = remember(FocusScopeNode::new).get();
    let focus_state = remember(ModalFocusState::default);
    let visible = is_open || is_animating;
    let modifier = modal_focus_modifier(Modifier::new(), focus_state, visible, focus_scope);
    layout().modifier(modifier).child(move || {
        overlay_background(main_content, visible);
        if is_open || is_animating {
            let content_alpha = visibility.opacity;
            let alpha = visibility.opacity * 0.32;
            let scrim_color = use_context::<MaterialTheme>()
                .expect("MaterialTheme must be provided")
                .get()
                .color_scheme
                .scrim;
            surface()
                .style(scrim_color.with_alpha(alpha).into())
                .on_click_shared(on_close_request)
                .show_ripple(false)
                .show_state_layer(false)
                .modifier(
                    Modifier::new()
                        .fill_max_size()
                        .semantics(SemanticsArgs {
                            hidden: true,
                            ..Default::default()
                        })
                        .focusable()
                        .focus_properties(
                            FocusProperties::new()
                                .can_focus(false)
                                .can_request_focus(false)
                                .skip_traversal(true),
                        ),
                )
                .block_input(true)
                .child(|| {});

            dialog_content_wrapper()
                .accessibility_label_optional(accessibility_label.clone())
                .focus_scope(focus_scope)
                .alpha(content_alpha)
                .padding(padding)
                .full_screen(full_screen.unwrap_or(false))
                .just_opened(just_opened)
                .on_close_request_shared(on_close_request)
                .content_shared(dialog_content);
        }
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
/// use tessera_components::{
///     dialog::{dialog_provider, full_screen_dialog},
///     text::text,
///     theme::material_theme,
/// };
/// use tessera_ui::{Callback, remember, tessera};
/// #[tessera]
/// fn example() {
///     material_theme().child(|| {
///         let open = remember(|| true);
///         let close = Callback::new(move || open.set(false));
///         dialog_provider()
///             .is_open(open.get())
///             .full_screen(true)
///             .on_close_request_shared(close)
///             .main_content(|| {
///                 text().content("Calendar");
///             })
///             .dialog_content(move || {
///                 full_screen_dialog(close).title("New event").content(|| {
///                     text().content("Event details");
///                 });
///             });
///         close.call();
///         assert!(!open.get());
///     });
/// }
/// example();
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
    let confirm_label = confirm_label.unwrap_or_else(|| "Save".to_owned());
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
                    let mut bar = top_app_bar()
                        .title(title.clone())
                        .container_color(scheme.surface_container_high)
                        .navigation_icon(move || {
                            icon_button()
                                .standard()
                                .color_style(IconButtonColorStyle::Inherited)
                                .icon(filled::CLOSE_SVG)
                                .accessibility_label("Close dialog")
                                .on_click_shared(on_close);
                        });
                    if let Some(on_confirm) = on_confirm {
                        let confirm_label = confirm_label.clone();
                        bar = bar.action(move || {
                            let confirm_label = confirm_label.clone();
                            button()
                                .text()
                                .content_color(scheme.primary)
                                .ripple_color(scheme.primary)
                                .accessibility_label(confirm_label.clone())
                                .on_click_shared(on_confirm)
                                .child(move || {
                                    text().content(confirm_label.clone());
                                });
                        });
                    }
                    drop(bar);
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
/// A Material Design 3 basic dialog component.
///
/// ## Usage
///
/// Use inside the `dialog_content` closure of [`dialog_provider`].
///
/// ## Parameters
///
/// - `icon` — optional icon slot shown above the headline.
/// - `headline` — optional headline text.
/// - `supporting_text` — body text shown in the dialog.
/// - `content` — optional custom body replacing the supporting text.
/// - `confirm_button` — optional confirm button slot.
/// - `dismiss_button` — optional dismiss button slot.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::{button::button, dialog::basic_dialog, text::text};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// basic_dialog()
///     .supporting_text("This is the dialog body text.")
///     .headline("Dialog Title")
///     .confirm_button(|| {
///         button().filled().on_click(|| {}).child(|| {
///             text().content("Confirm");
///         });
///     });
/// #     });
/// # }
/// # component();
/// ```
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

    column()
        .modifier(Modifier::new().constrain(
            Some(AxisConstraint::new(
                Dp(232.0).into(),
                Some(Dp(512.0).into()),
            )),
            Some(AxisConstraint::NONE),
        ))
        .cross_axis_alignment(alignment)
        .children(move || {
            if let Some(icon) = icon.as_ref() {
                let icon = *icon;
                let icon_color = scheme.secondary;
                {
                    provide_context(
                        || ContentColor {
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
                    text()
                        .content(headline.clone())
                        .style(theme.typography.headline_small)
                        .color(scheme.on_surface);
                };
                {
                    spacer().modifier(Modifier::new().height(Dp(16.0)));
                };
            }

            {
                if let Some(content) = content {
                    content.render();
                } else {
                    text()
                        .content(supporting_text.clone())
                        .style(theme.typography.body_medium)
                        .color(scheme.on_surface_variant);
                }
            };

            if confirm_button.is_some() || dismiss_button.is_some() {
                {
                    spacer().modifier(Modifier::new().height(Dp(24.0)));
                };
                let action_color = scheme.primary;
                {
                    provide_context(
                        || ContentColor {
                            current: action_color,
                        },
                        || {
                            row()
                                .modifier(Modifier::new().fill_max_width())
                                .main_axis_alignment(MainAxisAlignment::End)
                                .children(move || {
                                    let has_dismiss = dismiss_button.is_some();
                                    let has_confirm = confirm_button.is_some();

                                    if let Some(dismiss) = dismiss_button.as_ref() {
                                        let dismiss = *dismiss;
                                        dismiss.render();
                                    }

                                    if has_dismiss && has_confirm {
                                        {
                                            spacer().modifier(Modifier::new().width(Dp(8.0)));
                                        };
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
}
