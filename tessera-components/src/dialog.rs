//! Modal dialog provider — show modal content above the main app UI.
//!
//! ## Usage
//!
//! Used to show modal dialogs such as alerts, confirmations, wizards and forms;
//! dialogs block interaction with underlying content while active.
use std::time::Duration;

use tessera_ui::{
    AxisConstraint, Callback, Color, ComputedData, Dp, FocusScopeNode, FocusTraversalPolicy,
    MeasurementError, Modifier, Px, PxPosition, RenderSlot, State, current_frame_nanos,
    layout::{
        LayoutInput, LayoutOutput, LayoutPolicy, RenderInput, RenderPolicy, layout_primitive,
    },
    modifier::FocusModifierExt as _,
    provide_context, receive_frame_nanos, remember, tessera, use_context, winit,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    animation,
    boxed::boxed,
    column::column,
    fluid_glass::{GlassBorder, fluid_glass},
    modifier::{ModifierExt, with_keyboard_input},
    row::row,
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{ContentColor, MaterialTheme},
};

/// The duration of the full dialog animation.
const ANIM_TIME: Duration = Duration::from_millis(300);

/// Compute normalized (0..1) linear progress from an optional animation timer.
/// Placing this here reduces inline complexity inside the component body.
fn compute_dialog_progress(animation_start_frame_nanos: Option<u64>) -> f32 {
    animation_start_frame_nanos.map_or(1.0, |start_frame_nanos| {
        let elapsed_nanos = current_frame_nanos().saturating_sub(start_frame_nanos);
        let animation_nanos = ANIM_TIME.as_nanos().min(u64::MAX as u128) as u64;
        if elapsed_nanos >= animation_nanos {
            1.0
        } else {
            elapsed_nanos as f32 / animation_nanos as f32
        }
    })
}

/// Compute blur radius for glass style scrim.
fn blur_radius_for(progress: f32, is_open: bool, max_blur_radius: f32) -> f32 {
    if is_open {
        progress * max_blur_radius
    } else {
        max_blur_radius * (1.0 - progress)
    }
}

/// Compute scrim alpha for material style.
fn scrim_alpha_for(progress: f32, is_open: bool) -> f32 {
    if is_open {
        progress * 0.5
    } else {
        0.5 * (1.0 - progress)
    }
}

/// Defines the visual style of the dialog's scrim.
#[derive(Default, Clone, PartialEq, Copy)]
pub enum DialogStyle {
    /// A translucent glass effect that blurs the content behind it.
    Glass,
    /// A simple, semi-transparent dark overlay.
    #[default]
    Material,
}

impl DialogProviderRenderBuilder {
    fn on_close_request_handle(mut self, on_close_request: Callback) -> Self {
        self.props.on_close_request = Some(on_close_request);
        self
    }

    fn main_content_slot(mut self, main_content: RenderSlot) -> Self {
        self.props.main_content = Some(main_content);
        self
    }

    fn dialog_content_slot(mut self, dialog_content: RenderSlot) -> Self {
        self.props.dialog_content = Some(dialog_content);
        self
    }
}

impl DialogContentWrapperBuilder {
    fn on_close_request_handle(mut self, on_close_request: Callback) -> Self {
        self.props.on_close_request = on_close_request;
        self
    }

    fn content_slot(mut self, content: RenderSlot) -> Self {
        self.props.content = Some(content);
        self
    }
}

/// Controller for [`dialog_provider`], controlling visibility and animation.
pub struct DialogController {
    is_open: bool,
    animation_start_frame_nanos: Option<u64>,
}

impl DialogController {
    /// Creates a new dialog controller.
    pub fn new(initial_open: bool) -> Self {
        Self {
            is_open: initial_open,
            animation_start_frame_nanos: None,
        }
    }

    /// Opens the dialog, starting the animation if necessary.
    pub fn open(&mut self) {
        if !self.is_open {
            self.is_open = true;
            let now_nanos = current_frame_nanos();
            if let Some(old_start_frame_nanos) = self.animation_start_frame_nanos {
                let elapsed_nanos = now_nanos.saturating_sub(old_start_frame_nanos);
                let animation_nanos = ANIM_TIME.as_nanos().min(u64::MAX as u128) as u64;
                if elapsed_nanos < animation_nanos {
                    self.animation_start_frame_nanos =
                        Some(now_nanos.saturating_add(animation_nanos - elapsed_nanos));
                    return;
                }
            }
            self.animation_start_frame_nanos = Some(now_nanos);
        }
    }

    /// Closes the dialog, starting the closing animation if necessary.
    pub fn close(&mut self) {
        if self.is_open {
            self.is_open = false;
            let now_nanos = current_frame_nanos();
            if let Some(old_start_frame_nanos) = self.animation_start_frame_nanos {
                let elapsed_nanos = now_nanos.saturating_sub(old_start_frame_nanos);
                let animation_nanos = ANIM_TIME.as_nanos().min(u64::MAX as u128) as u64;
                if elapsed_nanos < animation_nanos {
                    self.animation_start_frame_nanos =
                        Some(now_nanos.saturating_add(animation_nanos - elapsed_nanos));
                    return;
                }
            }
            self.animation_start_frame_nanos = Some(now_nanos);
        }
    }

    /// Returns whether the dialog is currently open.
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    fn is_animating(&self) -> bool {
        self.animation_start_frame_nanos
            .map(|start| {
                let elapsed_nanos = current_frame_nanos().saturating_sub(start);
                let animation_nanos = ANIM_TIME.as_nanos().min(u64::MAX as u128) as u64;
                elapsed_nanos < animation_nanos
            })
            .unwrap_or(false)
    }

    fn snapshot(&self) -> (bool, Option<u64>) {
        (self.is_open, self.animation_start_frame_nanos)
    }
}

impl Default for DialogController {
    fn default() -> Self {
        Self::new(false)
    }
}

fn render_scrim(style: DialogStyle, on_close_request: Callback, is_open: bool, progress: f32) {
    match style {
        DialogStyle::Glass => {
            let blur_radius = blur_radius_for(progress, is_open, 5.0);
            fluid_glass()
                .on_click_shared(on_close_request)
                .tint_color(Color::TRANSPARENT)
                .modifier(Modifier::new().fill_max_size())
                .dispersion_height(Dp(0.0))
                .refraction_height(Dp(0.0))
                .block_input(true)
                .blur_radius(Dp(blur_radius as f64))
                .border(GlassBorder::new(Px(0)))
                .shape(Shape::RoundedRectangle {
                    top_left: RoundedCorner::manual(Dp(0.0), 3.0),
                    top_right: RoundedCorner::manual(Dp(0.0), 3.0),
                    bottom_right: RoundedCorner::manual(Dp(0.0), 3.0),
                    bottom_left: RoundedCorner::manual(Dp(0.0), 3.0),
                })
                .noise_amount(0.0)
                .with_child(|| {});
        }
        DialogStyle::Material => {
            let alpha = scrim_alpha_for(progress, is_open);
            let scrim_color = use_context::<MaterialTheme>()
                .expect("MaterialTheme must be provided")
                .get()
                .color_scheme
                .scrim;
            surface()
                .style(scrim_color.with_alpha(alpha).into())
                .on_click_shared(on_close_request)
                .modifier(Modifier::new().fill_max_size())
                .block_input(true)
                .with_child(|| {});
        }
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
    style: DialogStyle,
    alpha: f32,
    padding: Dp,
    just_opened: bool,
    on_close_request: Callback,
    content: Option<RenderSlot>,
) {
    let content = content.expect("dialog_content_wrapper requires content");
    let focus_scope = remember(FocusScopeNode::new).get();
    let modifier = with_keyboard_input(
        Modifier::new()
            .focus_scope_with(focus_scope)
            .focus_traversal_policy(
                FocusTraversalPolicy::linear()
                    .wrap(true)
                    .tab_navigation(true),
            ),
        make_keyboard_handler(on_close_request),
    );
    if just_opened {
        focus_scope.restore_focus();
    }
    let policy = DialogContentLayout { alpha };
    layout_primitive()
        .modifier(modifier)
        .layout_policy(policy.clone())
        .render_policy(policy)
        .child(move || {
            let content = content;
            boxed()
                .modifier(Modifier::new().fill_max_size())
                .alignment(Alignment::Center)
                .children(move || {
                    let content = content;
                    surface()
                        .style(Color::TRANSPARENT.into())
                        .modifier(
                            Modifier::new()
                                .constrain(Some(AxisConstraint::NONE), Some(AxisConstraint::NONE))
                                .padding_all(Dp(24.0)),
                        )
                        .with_child(move || match style {
                            DialogStyle::Glass => {
                                let content = content;
                                fluid_glass()
                                    .tint_color(Color::WHITE.with_alpha(alpha / 2.5))
                                    .blur_radius(Dp(5.0 * alpha as f64))
                                    .shape(Shape::RoundedRectangle {
                                        top_left: RoundedCorner::manual(Dp(28.0), 3.0),
                                        top_right: RoundedCorner::manual(Dp(28.0), 3.0),
                                        bottom_right: RoundedCorner::manual(Dp(28.0), 3.0),
                                        bottom_left: RoundedCorner::manual(Dp(28.0), 3.0),
                                    })
                                    .refraction_amount(32.0 * alpha)
                                    .block_input(true)
                                    .padding(padding)
                                    .with_child(move || {
                                        content.render();
                                    });
                            }
                            DialogStyle::Material => {
                                let content = content;
                                surface()
                                    .style(
                                        use_context::<MaterialTheme>()
                                            .expect("MaterialTheme must be provided")
                                            .get()
                                            .color_scheme
                                            .surface_container_high
                                            .into(),
                                    )
                                    .elevation(Dp(6.0))
                                    .shape(Shape::RoundedRectangle {
                                        top_left: RoundedCorner::manual(Dp(28.0), 3.0),
                                        top_right: RoundedCorner::manual(Dp(28.0), 3.0),
                                        bottom_right: RoundedCorner::manual(Dp(28.0), 3.0),
                                        bottom_left: RoundedCorner::manual(Dp(28.0), 3.0),
                                    })
                                    .block_input(true)
                                    .with_child(move || {
                                        let content = content;
                                        layout_primitive()
                                            .modifier(Modifier::new().padding_all(padding))
                                            .child(move || {
                                                content.render();
                                            });
                                    });
                            }
                        });
                });
        });
}

#[derive(Clone, PartialEq)]
struct DialogContentLayout {
    alpha: f32,
}

impl LayoutPolicy for DialogContentLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let Some(child_id) = input.children_ids().first().copied() else {
            return Ok(ComputedData {
                width: Px(0),
                height: Px(0),
            });
        };
        let computed = input.measure_child_in_parent_constraint(child_id)?;
        output.place_child(child_id, PxPosition::ZERO);
        Ok(computed)
    }
}

impl RenderPolicy for DialogContentLayout {
    fn record(&self, input: &RenderInput<'_>) {
        input.metadata_mut().multiply_opacity(self.alpha);
    }
}

/// # dialog_provider
///
/// Provide a modal dialog at the top level of an application.
///
/// # Usage
///
/// Show modal content for alerts, confirmation dialogs, multi-step forms, or
/// onboarding steps that require blocking user interaction with the main UI.
///
/// # Parameters
///
/// - `on_close_request` — optional close-request callback.
/// - `padding` — padding applied inside the dialog surface.
/// - `style` — dialog scrim style.
/// - `is_open` — declarative open state.
/// - `controller` — optional external controller.
/// - `main_content` — optional main content rendered behind the dialog.
/// - `dialog_content` — optional modal content rendered above the scrim.
///
/// # Examples
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
    on_close_request: Option<Callback>,
    padding: Dp,
    style: DialogStyle,
    is_open: bool,
    #[prop(skip_setter)] controller: Option<State<DialogController>>,
    main_content: Option<RenderSlot>,
    dialog_content: Option<RenderSlot>,
) {
    let on_close_request = on_close_request.unwrap_or_default();
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
        .on_close_request_handle(on_close_request)
        .padding(padding)
        .style(style)
        .controller(controller)
        .main_content_slot(main_content)
        .dialog_content_slot(dialog_content);
}

#[tessera]
fn dialog_provider_render(
    on_close_request: Option<Callback>,
    padding: Dp,
    style: DialogStyle,
    controller: Option<State<DialogController>>,
    main_content: Option<RenderSlot>,
    dialog_content: Option<RenderSlot>,
) {
    let on_close_request = on_close_request.unwrap_or_default();
    let controller = controller.expect("dialog_provider_render requires controller");
    let main_content = main_content.unwrap_or_else(RenderSlot::empty);
    let dialog_content = dialog_content.unwrap_or_else(RenderSlot::empty);
    let dialog_open_state = remember(|| false);

    // Render the main application content unconditionally.
    main_content.render();

    // If the dialog is open, render the modal overlay.
    // Sample state once to avoid repeated locks and improve readability.
    let (is_open, timer_opt) = controller.with(|c| c.snapshot());
    let mut just_opened = false;
    dialog_open_state.with_mut(|was_open| {
        just_opened = !*was_open && is_open;
        *was_open = is_open;
    });

    let is_animating = controller.with(|c| c.is_animating());
    if is_animating {
        receive_frame_nanos(move |frame_nanos| {
            let is_animating = controller.with_mut(|controller| {
                let (_, timer_opt) = controller.snapshot();
                if let Some(start_frame_nanos) = timer_opt {
                    let elapsed_nanos = frame_nanos.saturating_sub(start_frame_nanos);
                    let animation_nanos = ANIM_TIME.as_nanos().min(u64::MAX as u128) as u64;
                    elapsed_nanos < animation_nanos
                } else {
                    false
                }
            });
            if is_animating {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }

    if is_open || is_animating {
        let progress = animation::easing(compute_dialog_progress(timer_opt));

        let content_alpha = if is_open {
            progress * 1.0 // Transition from 0 to 1 alpha
        } else {
            1.0 * (1.0 - progress) // Transition from 1 to 0 alpha
        };

        render_scrim(style, on_close_request, is_open, progress);

        dialog_content_wrapper()
            .style(style)
            .alpha(content_alpha)
            .padding(padding)
            .just_opened(just_opened)
            .on_close_request_handle(on_close_request)
            .content_slot(dialog_content);
    }
}

/// # basic_dialog
///
/// A Material Design 3 basic dialog component.
///
/// # Usage
///
/// Use inside the `dialog_content` closure of [`dialog_provider`].
///
/// # Parameters
///
/// - `icon` — optional icon slot shown above the headline.
/// - `headline` — optional headline text.
/// - `supporting_text` — body text shown in the dialog.
/// - `confirm_button` — optional confirm button slot.
/// - `dismiss_button` — optional dismiss button slot.
///
/// # Examples
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
    #[prop(into)] supporting_text: String,
    confirm_button: Option<RenderSlot>,
    dismiss_button: Option<RenderSlot>,
) {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let alignment = if icon.is_some() {
        CrossAxisAlignment::Center
    } else {
        CrossAxisAlignment::Start
    };

    column()
        .modifier(Modifier::new().constrain(
            Some(AxisConstraint::new(
                Dp(280.0).into(),
                Some(Dp(560.0).into()),
            )),
            Some(AxisConstraint::NONE),
        ))
        .cross_axis_alignment(alignment)
        .children(move || {
            // Icon
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

            // Headline
            if let Some(headline) = headline.as_ref() {
                let headline = headline.clone();
                {
                    text()
                        .content(headline.clone())
                        .size(Dp(24.0))
                        .color(scheme.on_surface);
                };
                {
                    spacer().modifier(Modifier::new().height(Dp(16.0)));
                };
            }

            // Supporting Text
            {
                text()
                    .content(supporting_text.clone())
                    .size(Dp(14.0))
                    .color(scheme.on_surface_variant);
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
