//! Modal dialog provider — show modal content above the main app UI.
//!
//! ## Usage
//!
//! Used to show modal dialogs such as alerts, confirmations, wizards and forms;
//! dialogs block interaction with underlying content while active.
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use derive_setters::Setters;
use tessera_ui::{
    Callback, Color, ComputedData, DimensionValue, Dp, MeasurementError, Modifier, Px, PxPosition,
    RenderSlot, State,
    layout::{LayoutInput, LayoutOutput, LayoutSpec, RenderInput},
    provide_context, remember, tessera, use_context, winit, with_frame_nanos,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    animation,
    boxed::{BoxedArgs, boxed},
    column::{ColumnArgs, column},
    fluid_glass::{FluidGlassArgs, fluid_glass},
    modifier::ModifierExt,
    row::{RowArgs, row},
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
    theme::{ContentColor, MaterialTheme},
};

/// The duration of the full dialog animation.
const ANIM_TIME: Duration = Duration::from_millis(300);

/// Compute normalized (0..1) linear progress from an optional animation timer.
/// Placing this here reduces inline complexity inside the component body.
fn compute_dialog_progress(timer_opt: Option<Instant>) -> f32 {
    timer_opt.as_ref().map_or(1.0, |timer| {
        let elapsed = timer.elapsed();
        if elapsed >= ANIM_TIME {
            1.0
        } else {
            elapsed.as_secs_f32() / ANIM_TIME.as_secs_f32()
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

/// Arguments for the [`dialog_provider`] component.
#[derive(Clone, PartialEq, Setters)]
pub struct DialogProviderArgs {
    /// Callback function triggered when a close request is made, for example by
    /// clicking the scrim or pressing the `ESC` key.
    #[setters(skip)]
    pub on_close_request: Callback,
    /// Padding around the dialog content.
    pub padding: Dp,
    /// The visual style of the dialog's scrim.
    pub style: DialogStyle,
    /// Whether the dialog is initially open (for declarative usage).
    pub is_open: bool,
    /// Optional external controller for dialog visibility and animation state.
    #[setters(skip)]
    pub controller: Option<State<DialogController>>,
    /// Optional main content rendered behind the dialog.
    #[setters(skip)]
    pub main_content: Option<RenderSlot>,
    /// Optional dialog content rendered above the scrim.
    #[setters(skip)]
    pub dialog_content: Option<RenderSlot>,
}

impl DialogProviderArgs {
    /// Create args with a required close-request callback.
    pub fn new(on_close_request: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            on_close_request: Callback::new(on_close_request),
            padding: Dp(24.0),
            style: DialogStyle::default(),
            is_open: false,
            controller: None,
            main_content: None,
            dialog_content: None,
        }
    }

    /// Set the close-request callback.
    pub fn on_close_request<F>(mut self, on_close_request: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_close_request = Callback::new(on_close_request);
        self
    }

    /// Set the close-request callback using a shared callback.
    pub fn on_close_request_shared(mut self, on_close_request: impl Into<Callback>) -> Self {
        self.on_close_request = on_close_request.into();
        self
    }

    /// Sets an external dialog controller.
    pub fn controller(mut self, controller: State<DialogController>) -> Self {
        self.controller = Some(controller);
        self
    }

    /// Sets the main content slot.
    pub fn main_content<F>(mut self, main_content: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.main_content = Some(RenderSlot::new(main_content));
        self
    }

    /// Sets the main content slot using a shared render slot.
    pub fn main_content_shared(mut self, main_content: impl Into<RenderSlot>) -> Self {
        self.main_content = Some(main_content.into());
        self
    }

    /// Sets the dialog content slot.
    pub fn dialog_content<F>(mut self, dialog_content: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.dialog_content = Some(RenderSlot::new(dialog_content));
        self
    }

    /// Sets the dialog content slot using a shared render slot.
    pub fn dialog_content_shared(mut self, dialog_content: impl Into<RenderSlot>) -> Self {
        self.dialog_content = Some(dialog_content.into());
        self
    }
}

/// Controller for [`dialog_provider`], controlling visibility and animation.
pub struct DialogController {
    is_open: bool,
    timer: Option<Instant>,
}

impl DialogController {
    /// Creates a new dialog controller.
    pub fn new(initial_open: bool) -> Self {
        Self {
            is_open: initial_open,
            timer: None,
        }
    }

    /// Opens the dialog, starting the animation if necessary.
    pub fn open(&mut self) {
        if !self.is_open {
            self.is_open = true;
            let mut timer = Instant::now();
            if let Some(old_timer) = self.timer {
                let elapsed = old_timer.elapsed();
                if elapsed < ANIM_TIME {
                    timer += ANIM_TIME - elapsed;
                }
            }
            self.timer = Some(timer);
        }
    }

    /// Closes the dialog, starting the closing animation if necessary.
    pub fn close(&mut self) {
        if self.is_open {
            self.is_open = false;
            let mut timer = Instant::now();
            if let Some(old_timer) = self.timer {
                let elapsed = old_timer.elapsed();
                if elapsed < ANIM_TIME {
                    timer += ANIM_TIME - elapsed;
                }
            }
            self.timer = Some(timer);
        }
    }

    /// Returns whether the dialog is currently open.
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    fn snapshot(&self) -> (bool, Option<Instant>) {
        (self.is_open, self.timer)
    }
}

impl Default for DialogController {
    fn default() -> Self {
        Self::new(false)
    }
}

type DialogRenderFn = dyn Fn() + Send + Sync;

#[derive(Clone)]
struct DialogContentWrapperArgs {
    style: DialogStyle,
    alpha: f32,
    padding: Dp,
    content: Arc<DialogRenderFn>,
}

impl PartialEq for DialogContentWrapperArgs {
    fn eq(&self, other: &Self) -> bool {
        self.style == other.style
            && self.alpha == other.alpha
            && self.padding == other.padding
            && Arc::ptr_eq(&self.content, &other.content)
    }
}

#[derive(Clone)]
struct DialogProviderRenderArgs {
    on_close_request: Callback,
    padding: Dp,
    style: DialogStyle,
    controller: State<DialogController>,
    main_content: Arc<DialogRenderFn>,
    dialog_content: Arc<DialogRenderFn>,
}

impl PartialEq for DialogProviderRenderArgs {
    fn eq(&self, other: &Self) -> bool {
        self.on_close_request == other.on_close_request
            && self.padding == other.padding
            && self.style == other.style
            && self.controller == other.controller
            && Arc::ptr_eq(&self.main_content, &other.main_content)
            && Arc::ptr_eq(&self.dialog_content, &other.dialog_content)
    }
}

fn render_scrim(style: DialogStyle, on_close_request: &Callback, is_open: bool, progress: f32) {
    match style {
        DialogStyle::Glass => {
            let blur_radius = blur_radius_for(progress, is_open, 5.0);
            fluid_glass(&crate::fluid_glass::FluidGlassArgs::with_child(
                FluidGlassArgs::default()
                    .on_click_shared(on_close_request.clone())
                    .tint_color(Color::TRANSPARENT)
                    .modifier(Modifier::new().fill_max_size())
                    .dispersion_height(Dp(0.0))
                    .refraction_height(Dp(0.0))
                    .block_input(true)
                    .blur_radius(Dp(blur_radius as f64))
                    .border(None)
                    .shape(Shape::RoundedRectangle {
                        top_left: RoundedCorner::manual(Dp(0.0), 3.0),
                        top_right: RoundedCorner::manual(Dp(0.0), 3.0),
                        bottom_right: RoundedCorner::manual(Dp(0.0), 3.0),
                        bottom_left: RoundedCorner::manual(Dp(0.0), 3.0),
                    })
                    .noise_amount(0.0),
                || {},
            ));
        }
        DialogStyle::Material => {
            let alpha = scrim_alpha_for(progress, is_open);
            let scrim_color = use_context::<MaterialTheme>()
                .expect("MaterialTheme must be provided")
                .get()
                .color_scheme
                .scrim;
            surface(&crate::surface::SurfaceArgs::with_child(
                SurfaceArgs::default()
                    .style(scrim_color.with_alpha(alpha).into())
                    .on_click_shared(on_close_request.clone())
                    .modifier(Modifier::new().fill_max_size())
                    .block_input(true),
                || {},
            ));
        }
    }
}

fn make_keyboard_input_handler(
    on_close: Callback,
) -> Box<dyn for<'a> Fn(tessera_ui::InputHandlerInput<'a>) + Send + Sync + 'static> {
    Box::new(move |input| {
        input.keyboard_events.drain(..).for_each(|event| {
            if event.state == winit::event::ElementState::Pressed
                && let winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) =
                    event.physical_key
            {
                on_close.call();
            }
        });
    })
}

#[tessera]
fn dialog_content_wrapper_node(args: &DialogContentWrapperArgs) {
    let style = args.style;
    let alpha = args.alpha;
    let padding = args.padding;
    let content = args.content.clone();
    layout(DialogContentLayout { alpha });

    boxed(
        BoxedArgs::default()
            .modifier(Modifier::new().fill_max_size())
            .alignment(Alignment::Center),
        |scope| {
            scope.child(move || {
                let content = content.clone();
                surface(&crate::surface::SurfaceArgs::with_child(
                    SurfaceArgs::default()
                        .style(Color::TRANSPARENT.into())
                        .modifier(
                            Modifier::new()
                                .constrain(Some(DimensionValue::WRAP), Some(DimensionValue::WRAP))
                                .padding_all(Dp(24.0)),
                        ),
                    move || match style {
                        DialogStyle::Glass => {
                            let content_for_glass = content.clone();
                            fluid_glass(&crate::fluid_glass::FluidGlassArgs::with_child(
                                FluidGlassArgs::default()
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
                                    .padding(padding),
                                move || {
                                    content_for_glass();
                                },
                            ));
                        }
                        DialogStyle::Material => {
                            let content_for_material = content.clone();
                            surface(&crate::surface::SurfaceArgs::with_child(
                                SurfaceArgs::default()
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
                                    .block_input(true),
                                move || {
                                    let content_for_material = content_for_material.clone();
                                    Modifier::new().padding_all(padding).run(move || {
                                        content_for_material();
                                    });
                                },
                            ));
                        }
                    },
                ));
            });
        },
    );
}

#[derive(Clone, PartialEq)]
struct DialogContentLayout {
    alpha: f32,
}

impl LayoutSpec for DialogContentLayout {
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

    fn record(&self, input: &RenderInput<'_>) {
        input.metadata_mut().opacity *= self.alpha;
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
/// - `args` — configuration for dialog appearance and the `on_close_request`
///   callback; see [`DialogProviderArgs`].
/// - `main_content` — closure that renders the always-visible base UI.
/// - `dialog_content` — closure that renders dialog content.
///
/// # Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::dialog::{
///     BasicDialogArgs, DialogProviderArgs, basic_dialog, dialog_provider,
/// };
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # let args = tessera_components::theme::MaterialThemeProviderArgs::new(
/// #     || MaterialTheme::default(),
/// #     || {
/// dialog_provider(
///     &DialogProviderArgs::new(|| {})
///         .is_open(true)
///         .main_content(|| { /* main content */ })
///         .dialog_content(|| {
///             basic_dialog(
///                 &BasicDialogArgs::new("This is the dialog body text.").headline("Dialog Title"),
///             );
///         }),
/// );
/// #     },
/// # );
/// # material_theme(&args);
/// # }
/// # component();
/// ```
#[tessera]
pub fn dialog_provider(args: &DialogProviderArgs) {
    let args = args.clone();
    let main_content_slot = args
        .main_content
        .clone()
        .unwrap_or_else(|| RenderSlot::new(|| {}));
    let dialog_content_slot = args
        .dialog_content
        .clone()
        .unwrap_or_else(|| RenderSlot::new(|| {}));
    let main_content: Arc<DialogRenderFn> = Arc::new(move || {
        main_content_slot.render();
    });
    let dialog_content: Arc<DialogRenderFn> = Arc::new(move || {
        dialog_content_slot.render();
    });
    let controller = args
        .controller
        .unwrap_or_else(|| remember(|| DialogController::new(args.is_open)));

    // In controlled mode (external controller provided), do not override
    // controller state from `args.is_open`.
    if args.controller.is_none() {
        let current_open = controller.with(|c| c.is_open());
        if args.is_open != current_open {
            if args.is_open {
                controller.with_mut(|c| c.open());
            } else {
                controller.with_mut(|c| c.close());
            }
        }
    }

    let provider_render_args = DialogProviderRenderArgs {
        on_close_request: args.on_close_request.clone(),
        padding: args.padding,
        style: args.style,
        controller,
        main_content,
        dialog_content,
    };
    dialog_provider_node(&provider_render_args);
}

#[tessera]
fn dialog_provider_node(args: &DialogProviderRenderArgs) {
    let args = args.clone();
    let controller = args.controller;
    let main_content = args.main_content;
    let dialog_content = args.dialog_content;

    // Render the main application content unconditionally.
    main_content();

    // If the dialog is open, render the modal overlay.
    // Sample state once to avoid repeated locks and improve readability.
    let (is_open, timer_opt) = controller.with(|c| c.snapshot());

    let is_animating = timer_opt.is_some_and(|t| t.elapsed() < ANIM_TIME);
    if is_animating {
        let controller_for_frame = controller;
        with_frame_nanos(move |_| {
            controller_for_frame.with_mut(|_| {});
        });
    }

    if is_open || is_animating {
        let progress = animation::easing(compute_dialog_progress(timer_opt));

        let content_alpha = if is_open {
            progress * 1.0 // Transition from 0 to 1 alpha
        } else {
            1.0 * (1.0 - progress) // Transition from 1 to 0 alpha
        };

        render_scrim(args.style, &args.on_close_request, is_open, progress);
        let handler = make_keyboard_input_handler(args.on_close_request.clone());
        input_handler(handler);

        let content_wrapper_args = DialogContentWrapperArgs {
            style: args.style,
            alpha: content_alpha,
            padding: args.padding,
            content: dialog_content.clone(),
        };
        dialog_content_wrapper_node(&content_wrapper_args);
    }
}

/// Arguments for the [`basic_dialog`] component.
#[derive(Clone, PartialEq, Setters)]
pub struct BasicDialogArgs {
    /// Optional icon to display at the top of the dialog.
    #[setters(skip)]
    pub icon: Option<RenderSlot>,
    /// Optional headline text.
    #[setters(strip_option, into)]
    pub headline: Option<String>,
    /// The supporting text of the dialog.
    #[setters(into)]
    pub supporting_text: String,
    /// The button used to confirm a proposed action, thus resolving what
    /// triggered the dialog.
    #[setters(skip)]
    pub confirm_button: Option<RenderSlot>,
    /// The button used to dismiss a proposed action, thus resolving what
    /// triggered the dialog.
    #[setters(skip)]
    pub dismiss_button: Option<RenderSlot>,
}

impl BasicDialogArgs {
    /// Create args with required supporting text.
    pub fn new(supporting_text: impl Into<String>) -> Self {
        Self {
            icon: None,
            headline: None,
            supporting_text: supporting_text.into(),
            confirm_button: None,
            dismiss_button: None,
        }
    }

    /// Sets the optional icon drawing callback.
    pub fn icon<F>(mut self, icon: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.icon = Some(RenderSlot::new(icon));
        self
    }

    /// Sets the optional icon drawing callback using a shared callback.
    pub fn icon_shared(mut self, icon: impl Into<RenderSlot>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Sets the confirm button content.
    pub fn confirm_button<F>(mut self, f: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.confirm_button = Some(RenderSlot::new(f));
        self
    }

    /// Sets the confirm button content using a shared callback.
    pub fn confirm_button_shared(mut self, f: impl Into<RenderSlot>) -> Self {
        self.confirm_button = Some(f.into());
        self
    }

    /// Sets the dismiss button content.
    pub fn dismiss_button<F>(mut self, f: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.dismiss_button = Some(RenderSlot::new(f));
        self
    }

    /// Sets the dismiss button content using a shared callback.
    pub fn dismiss_button_shared(mut self, f: impl Into<RenderSlot>) -> Self {
        self.dismiss_button = Some(f.into());
        self
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
/// - `args` — configuration for the dialog content; see [`BasicDialogArgs`].
///
/// # Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::{
///     button::{ButtonArgs, button},
///     dialog::{BasicDialogArgs, basic_dialog},
///     text::text,
/// };
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # let args = tessera_components::theme::MaterialThemeProviderArgs::new(
/// #     || MaterialTheme::default(),
/// #     || {
/// basic_dialog(
///     &BasicDialogArgs::new("This is the dialog body text.")
///         .headline("Dialog Title")
///         .confirm_button(|| {
///             button(&ButtonArgs::filled(|| {}).child(|| {
///                 text(&tessera_components::text::TextArgs::default().text("Confirm"));
///             }));
///         }),
/// );
/// #     },
/// # );
/// # material_theme(&args);
/// # }
/// # component();
/// ```
#[tessera]
pub fn basic_dialog(args: &BasicDialogArgs) {
    let args = args.clone();
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let alignment = if args.icon.is_some() {
        CrossAxisAlignment::Center
    } else {
        CrossAxisAlignment::Start
    };
    let icon = args.icon.clone();
    let headline = args.headline.clone();
    let supporting_text = args.supporting_text.clone();
    let confirm_button = args.confirm_button.clone();
    let dismiss_button = args.dismiss_button.clone();

    column(
        ColumnArgs::default()
            .modifier(Modifier::new().constrain(
                Some(DimensionValue::Wrap {
                    min: Some(Dp(280.0).into()),
                    max: Some(Dp(560.0).into()),
                }),
                Some(DimensionValue::WRAP),
            ))
            .cross_axis_alignment(alignment),
        move |scope| {
            // Icon
            if let Some(icon) = icon.as_ref() {
                let icon = icon.clone();
                let icon_color = scheme.secondary;
                scope.child(move || {
                    provide_context(
                        || ContentColor {
                            current: icon_color,
                        },
                        || {
                            icon.render();
                        },
                    );
                });
                scope.child(|| {
                    spacer(&crate::spacer::SpacerArgs::new(
                        Modifier::new().height(Dp(16.0)),
                    ));
                });
            }

            // Headline
            if let Some(headline) = headline.as_ref() {
                let headline = headline.clone();
                scope.child(move || {
                    text(&crate::text::TextArgs::from(
                        &TextArgs::default()
                            .text(headline.clone())
                            .size(Dp(24.0))
                            .color(scheme.on_surface),
                    ));
                });
                scope.child(|| {
                    spacer(&crate::spacer::SpacerArgs::new(
                        Modifier::new().height(Dp(16.0)),
                    ));
                });
            }

            // Supporting Text
            scope.child(move || {
                text(&crate::text::TextArgs::from(
                    &TextArgs::default()
                        .text(supporting_text.clone())
                        .size(Dp(14.0))
                        .color(scheme.on_surface_variant),
                ));
            });

            if confirm_button.is_some() || dismiss_button.is_some() {
                scope.child(|| {
                    spacer(&crate::spacer::SpacerArgs::new(
                        Modifier::new().height(Dp(24.0)),
                    ));
                });
                let action_color = scheme.primary;
                scope.child(move || {
                    provide_context(
                        || ContentColor {
                            current: action_color,
                        },
                        || {
                            let dismiss_button = dismiss_button.clone();
                            let confirm_button = confirm_button.clone();
                            row(
                                RowArgs::default()
                                    .modifier(Modifier::new().fill_max_width())
                                    .main_axis_alignment(MainAxisAlignment::End),
                                |s| {
                                    let has_dismiss = dismiss_button.is_some();
                                    let has_confirm = confirm_button.is_some();

                                    if let Some(dismiss) = dismiss_button.as_ref() {
                                        let dismiss = dismiss.clone();
                                        s.child(move || dismiss.render());
                                    }

                                    if has_dismiss && has_confirm {
                                        s.child(|| {
                                            spacer(&crate::spacer::SpacerArgs::new(
                                                Modifier::new().width(Dp(8.0)),
                                            ));
                                        });
                                    }

                                    if let Some(confirm) = confirm_button.as_ref() {
                                        let confirm = confirm.clone();
                                        s.child(move || confirm.render());
                                    }
                                },
                            );
                        },
                    );
                });
            }
        },
    );
}
