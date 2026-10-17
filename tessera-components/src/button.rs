//! Material-styled button components.
//!
//! ## Usage
//!
//! Trigger actions, submit forms, or navigate.
use tessera_ui::{
    AxisConstraint, Callback, Color, ComputedData, Constraint, Dp, FrameNanosControl, LayoutResult,
    MeasurementError, Modifier, Px, PxPosition, RenderSlot, State,
    accesskit::{Role, Toggled},
    context::provide_context,
    current_frame_nanos,
    layout::{MeasureScope, layout},
    receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    alignment::Alignment,
    animation::Animatable,
    boxed::boxed,
    icon::IconSize,
    modifier::{InteractionState, ModifierExt, Padding},
    shape_def::{RoundedCorner, Shape},
    surface::{SurfaceStyle, surface},
    theme::{
        ContentColor, MaterialAlpha, MaterialColorScheme, MaterialTheme, TextStyle,
        content_color_for,
    },
};

/// Visual variants of the Material Design 3 [`button`].
///
/// Each variant resolves its container, content, and border colors from the
/// active [`MaterialColorScheme`] unless explicitly overridden.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    /// Filled button using the primary container color. High emphasis.
    #[default]
    Filled,
    /// Filled button on a low surface container with a shadow.
    Elevated,
    /// Filled button using the secondary container color. Medium emphasis.
    Tonal,
    /// Transparent button with an outline border. Medium emphasis.
    Outlined,
    /// Transparent button without a border. Low emphasis.
    Text,
}
/// Resting shape of an Expressive button.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonShape {
    /// Fully rounded ends, changing to square corners when selected.
    #[default]
    Round,
    /// Size-specific corners, changing to rounded ends when selected.
    Square,
}

/// Expressive button dimensions and content metrics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    /// A 32dp container with 20dp icons.
    ExtraSmall,
    /// A 40dp container with 20dp icons.
    #[default]
    Small,
    /// A 56dp container with 24dp icons.
    Medium,
    /// A 96dp container with 32dp icons.
    Large,
    /// A 136dp container with 40dp icons.
    ExtraLarge,
}

impl ButtonSize {
    /// Returns the minimum visual container height.
    pub const fn height(self) -> Dp {
        Dp(match self {
            Self::ExtraSmall => 32.0,
            Self::Small => 40.0,
            Self::Medium => 56.0,
            Self::Large => 96.0,
            Self::ExtraLarge => 136.0,
        })
    }

    /// Returns the recommended leading and trailing icon size.
    pub const fn icon_size(self) -> Dp {
        Dp(match self {
            Self::ExtraSmall | Self::Small => 20.0,
            Self::Medium => 24.0,
            Self::Large => 32.0,
            Self::ExtraLarge => 40.0,
        })
    }

    /// Returns the spacing between icons and the label.
    pub const fn icon_spacing(self) -> Dp {
        Dp(match self {
            Self::ExtraSmall | Self::Small | Self::Medium => 8.0,
            Self::Large => 12.0,
            Self::ExtraLarge => 16.0,
        })
    }

    /// Returns the default content insets.
    pub const fn content_padding(self) -> Padding {
        let (horizontal, vertical) = match self {
            Self::ExtraSmall => (12.0, 6.0),
            Self::Small => (16.0, 10.0),
            Self::Medium => (24.0, 16.0),
            Self::Large => (48.0, 32.0),
            Self::ExtraLarge => (64.0, 48.0),
        };
        Padding::symmetric(Dp(horizontal), Dp(vertical))
    }

    /// Returns the default outline thickness.
    pub const fn outline_width(self) -> Dp {
        Dp(match self {
            Self::ExtraSmall | Self::Small | Self::Medium => 1.0,
            Self::Large => 2.0,
            Self::ExtraLarge => 3.0,
        })
    }

    /// Returns the resting square corner radius.
    pub const fn square_radius(self) -> Dp {
        Dp(match self {
            Self::ExtraSmall | Self::Small => 12.0,
            Self::Medium => 16.0,
            Self::Large | Self::ExtraLarge => 28.0,
        })
    }

    /// Returns the pressed corner radius for both shape presets.
    pub const fn pressed_radius(self) -> Dp {
        Dp(match self {
            Self::ExtraSmall | Self::Small => 8.0,
            Self::Medium => 12.0,
            Self::Large | Self::ExtraLarge => 16.0,
        })
    }

    fn text_style(self, theme: &MaterialTheme) -> TextStyle {
        match self {
            Self::ExtraSmall | Self::Small => theme.typography.label_large,
            Self::Medium => theme.typography.title_medium,
            Self::Large => theme.typography.headline_small,
            Self::ExtraLarge => theme.typography.headline_large,
        }
    }
}

/// Material Design 3 defaults for [`button`].
pub struct ButtonDefaults;

impl ButtonDefaults {
    /// Default pressed alpha used for ripple feedback.
    pub const PRESSED_ALPHA: f32 = MaterialAlpha::PRESSED;
    /// Default disabled container alpha.
    pub const DISABLED_CONTAINER_ALPHA: f32 = MaterialAlpha::DISABLED_CONTAINER;
    /// Default disabled content alpha.
    pub const DISABLED_CONTENT_ALPHA: f32 = MaterialAlpha::DISABLED_CONTENT;
    /// Disabled container opacity used by filled/elevated buttons.
    pub const FILLED_DISABLED_CONTAINER_ALPHA: f32 = 0.1;
    /// Disabled content opacity used by most buttons.
    pub const DISABLED_LABEL_ALPHA: f32 = 0.38;
    /// Minimum width for buttons (Material default = 58dp).
    pub const MIN_WIDTH: Dp = Dp(58.0);
    /// Minimum height for buttons (Material default = 40dp).
    pub const MIN_HEIGHT: Dp = Dp(40.0);
    /// Horizontal padding used inside buttons.
    pub const CONTENT_HORIZONTAL_PADDING: Dp = Dp(16.0);
    /// Vertical padding used inside buttons.
    pub const CONTENT_VERTICAL_PADDING: Dp = Dp(10.0);

    /// Default disabled container color for filled buttons.
    pub fn disabled_container_color(scheme: &MaterialColorScheme) -> Color {
        scheme
            .on_surface
            .with_alpha(Self::FILLED_DISABLED_CONTAINER_ALPHA)
    }

    /// Default disabled content color for filled buttons.
    pub fn disabled_content_color(scheme: &MaterialColorScheme) -> Color {
        scheme
            .on_surface_variant
            .with_alpha(Self::DISABLED_LABEL_ALPHA)
    }

    /// Default disabled border color for outlined buttons.
    pub fn disabled_border_color(scheme: &MaterialColorScheme) -> Color {
        scheme
            .outline_variant
            .with_alpha(Self::FILLED_DISABLED_CONTAINER_ALPHA)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ButtonMotionTarget {
    fraction: f32,
    radius: f32,
    elevation: f32,
    outline: f32,
    toggle: bool,
}

impl ButtonMotionTarget {
    fn corner(size: ButtonSize, shape: ButtonShape, selected: bool, pressed: bool) -> (f32, f32) {
        if pressed {
            (0.0, size.pressed_radius().0 as f32)
        } else if (shape == ButtonShape::Round) != selected {
            (0.5, 0.0)
        } else {
            (0.0, size.square_radius().0 as f32)
        }
    }
}

struct ButtonMotion {
    fraction: Animatable,
    radius: Animatable,
    elevation: Animatable,
    outline: Animatable,
    target: ButtonMotionTarget,
    frame_nanos: u64,
}

impl ButtonMotion {
    fn spatial(value: f32, toggle: bool) -> Animatable {
        if toggle {
            Animatable::spring(value, 800.0_f32.sqrt(), 0.6)
        } else {
            Animatable::spring(value, 40.0, 1.0)
        }
    }

    fn new(target: ButtonMotionTarget, frame_nanos: u64) -> Self {
        Self {
            fraction: Self::spatial(target.fraction, target.toggle),
            radius: Self::spatial(target.radius, target.toggle),
            elevation: Animatable::spring(target.elevation, 40.0, 1.0),
            outline: Self::spatial(target.outline, true),
            target,
            frame_nanos,
        }
    }

    fn retarget(&mut self, target: ButtonMotionTarget, frame_nanos: u64) {
        self.advance(frame_nanos);
        if target.toggle != self.target.toggle {
            self.fraction = Self::spatial(self.fraction.value(), target.toggle)
                .with_velocity(self.fraction.velocity());
            self.radius = Self::spatial(self.radius.value(), target.toggle)
                .with_velocity(self.radius.velocity());
        }
        self.fraction.set_target(target.fraction);
        self.radius.set_target(target.radius);
        self.elevation.set_target(target.elevation);
        if target.toggle {
            self.outline.set_target(target.outline);
        } else {
            self.outline = Self::spatial(target.outline, true);
        }
        self.target = target;
    }

    fn advance(&mut self, frame_nanos: u64) {
        if frame_nanos > self.frame_nanos {
            let dt = (frame_nanos - self.frame_nanos) as f32 / 1_000_000_000.0;
            self.fraction.tick(dt);
            self.radius.tick(dt);
            self.elevation.tick(dt);
            self.outline.tick(dt);
            self.frame_nanos = frame_nanos;
        }
    }

    fn is_idle(&self) -> bool {
        self.fraction.is_idle()
            && self.radius.is_idle()
            && self.elevation.is_idle()
            && self.outline.is_idle()
    }

    fn shape(&self) -> Shape {
        let corner =
            RoundedCorner::relative(self.fraction.value(), Dp(self.radius.value() as f64), 2.0);
        Shape::RoundedRectangle {
            top_left: corner,
            top_right: corner,
            bottom_right: corner,
            bottom_left: corner,
        }
    }
}

#[derive(Clone)]
struct ButtonResolvedArgs {
    enabled: bool,
    modifier: Modifier,
    color: Color,
    content_color: Option<Color>,
    shape: Shape,
    padding: Padding,
    on_click: Option<Callback>,
    ripple_color: Color,
    border_width: Dp,
    border_color: Option<Color>,
    elevation: Option<Dp>,
    tonal_elevation: Dp,
    disabled_container_color: Color,
    disabled_content_color: Color,
    disabled_border_color: Color,
    accessibility_label: Option<String>,
    accessibility_description: Option<String>,
    child: Option<RenderSlot>,
}

/// # button
///
/// Provides a clickable button with customizable style and ripple feedback.
///
/// ## Usage
///
/// Use to trigger an action when the user clicks or taps.
///
/// ## Parameters
///
/// - `variant` — optional visual variant
///   (`filled`/`elevated`/`tonal`/`outlined`/`text`).
/// - `size` — Expressive size preset; defaults to small.
/// - `shape_style` — round or square preset; defaults to round.
/// - `selected` — controlled toggle state; omit for an ordinary action. Text
///   buttons do not support toggling. Update this value in `on_click`.
/// - `interaction_state` — optional shared state for interaction previews.
/// - `enabled` — optional enabled flag.
/// - `modifier` — modifier chain applied to the button subtree.
/// - `color` — optional container color override.
/// - `content_color` — optional content color override.
/// - `shape` — fixed shape override, replacing automatic corner morphing.
/// - `padding` — optional uniform internal padding override.
/// - `content_padding` — optional per-edge insets; takes precedence over
///   `padding`.
/// - `leading_icon`, `trailing_icon` — optional icons, sized and spaced by the
///   preset.
/// - `on_click` — optional click callback.
/// - `ripple_color` — optional ripple tint override.
/// - `border_width` — optional outline width.
/// - `border_color` — optional outline color.
/// - `elevation` — optional surface elevation.
/// - `tonal_elevation` — optional tonal elevation.
/// - `disabled_container_color` — optional disabled container color.
/// - `disabled_content_color` — optional disabled content color.
/// - `disabled_border_color` — optional disabled border color.
/// - `accessibility_label` — optional accessibility label.
/// - `accessibility_description` — optional accessibility description.
/// - `child` — optional child render slot.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// use tessera_components::button::ButtonSize;
/// assert_eq!(ButtonSize::Medium.height(), tessera_ui::Dp(56.0));
/// # #[tessera]
/// # fn component() {
/// use tessera_components::{button::button, text::text};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// let selected = tessera_ui::remember(|| false);
/// let toggle = tessera_ui::Callback::new(move || selected.set(!selected.get()));
/// button()
///     .tonal()
///     .square()
///     .selected(selected.get())
///     .on_click_shared(toggle)
///     .child(|| {
///         text().content("Favorite");
///     });
/// assert!(!selected.get());
/// toggle.call();
/// assert!(selected.get());
/// #     });
/// # }
/// # component();
/// ```
/// Renders a Material button.
#[tessera]
pub fn button(
    variant: Option<ButtonVariant>,
    size: Option<ButtonSize>,
    shape_style: Option<ButtonShape>,
    selected: Option<bool>,
    interaction_state: Option<State<InteractionState>>,
    enabled: Option<bool>,
    modifier: Option<Modifier>,
    color: Option<Color>,
    content_color: Option<Color>,
    shape: Option<Shape>,
    padding: Option<Dp>,
    content_padding: Option<Padding>,
    leading_icon: Option<RenderSlot>,
    trailing_icon: Option<RenderSlot>,
    on_click: Option<Callback>,
    ripple_color: Option<Color>,
    border_width: Option<Dp>,
    border_color: Option<Color>,
    elevation: Option<Dp>,
    tonal_elevation: Option<Dp>,
    disabled_container_color: Option<Color>,
    disabled_content_color: Option<Color>,
    disabled_border_color: Option<Color>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    child: Option<RenderSlot>,
) {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let variant = variant.unwrap_or_default();
    let size = size.unwrap_or_default();
    assert!(
        variant != ButtonVariant::Text || selected.is_none(),
        "text buttons do not support toggle selection"
    );
    let enabled = enabled.unwrap_or(true);
    let interaction_state = if let Some(state) = interaction_state {
        state
    } else {
        remember(InteractionState::new)
    };
    if !enabled
        && interaction_state.with(|state| {
            state.is_pressed() || state.is_hovered() || state.is_focused() || state.is_dragged()
        })
    {
        interaction_state.with_mut(|state| *state = InteractionState::new());
    }
    let pressed = enabled && interaction_state.with(InteractionState::is_pressed);
    let hovered = enabled && interaction_state.with(InteractionState::is_hovered);
    let focused = enabled && interaction_state.with(InteractionState::is_focused);
    let (
        default_container_color,
        default_content_color,
        default_ripple_color,
        default_border_width,
        default_border_color,
        default_elevation,
        default_disabled_container_color,
        default_disabled_border_color,
    ) = match variant {
        ButtonVariant::Filled => (
            scheme.primary,
            scheme.on_primary,
            scheme.on_primary,
            Dp(0.0),
            None,
            None,
            ButtonDefaults::disabled_container_color(&scheme),
            ButtonDefaults::disabled_border_color(&scheme),
        ),
        ButtonVariant::Elevated => (
            scheme.surface_container_low,
            scheme.primary,
            scheme.primary,
            Dp(0.0),
            None,
            Some(Dp(1.0)),
            ButtonDefaults::disabled_container_color(&scheme),
            ButtonDefaults::disabled_border_color(&scheme),
        ),
        ButtonVariant::Tonal => (
            scheme.secondary_container,
            scheme.on_secondary_container,
            scheme.on_secondary_container,
            Dp(0.0),
            None,
            None,
            ButtonDefaults::disabled_container_color(&scheme),
            ButtonDefaults::disabled_border_color(&scheme),
        ),
        ButtonVariant::Outlined => (
            Color::TRANSPARENT,
            scheme.on_surface_variant,
            scheme.on_surface_variant,
            size.outline_width(),
            Some(scheme.outline_variant),
            None,
            Color::TRANSPARENT,
            ButtonDefaults::disabled_border_color(&scheme),
        ),
        ButtonVariant::Text => (
            Color::TRANSPARENT,
            scheme.primary,
            scheme.primary,
            Dp(0.0),
            None,
            None,
            Color::TRANSPARENT,
            ButtonDefaults::disabled_border_color(&scheme),
        ),
    };
    let (default_container_color, default_content_color) = match (variant, selected) {
        (ButtonVariant::Filled, Some(false)) => {
            (scheme.surface_container, scheme.on_surface_variant)
        }
        (ButtonVariant::Filled | ButtonVariant::Elevated, Some(true)) => {
            (scheme.primary, scheme.on_primary)
        }
        (ButtonVariant::Tonal, Some(true)) => (scheme.secondary, scheme.on_secondary),
        (ButtonVariant::Outlined, Some(true)) => {
            (scheme.inverse_surface, scheme.inverse_on_surface)
        }
        _ => (default_container_color, default_content_color),
    };
    let default_border_width = if variant == ButtonVariant::Outlined && selected == Some(true) {
        Dp(0.0)
    } else {
        default_border_width
    };
    let default_disabled_container_color =
        if variant == ButtonVariant::Outlined && selected == Some(true) {
            ButtonDefaults::disabled_container_color(&scheme)
        } else {
            default_disabled_container_color
        };
    let default_elevation = if !enabled {
        Dp(0.0)
    } else if variant == ButtonVariant::Elevated {
        Dp(if hovered && !pressed && !focused {
            3.0
        } else {
            1.0
        })
    } else if matches!(variant, ButtonVariant::Filled | ButtonVariant::Tonal)
        && hovered
        && !pressed
        && !focused
    {
        Dp(1.0)
    } else {
        default_elevation.unwrap_or(Dp(0.0))
    };
    let (fraction, radius) = if shape.is_some() {
        (0.0, 0.0)
    } else {
        ButtonMotionTarget::corner(
            size,
            shape_style.unwrap_or_default(),
            selected.unwrap_or(false),
            pressed,
        )
    };
    let target = ButtonMotionTarget {
        fraction,
        radius,
        elevation: elevation.unwrap_or(default_elevation).0 as f32,
        outline: border_width.unwrap_or(default_border_width).0 as f32,
        toggle: selected.is_some(),
    };
    let frame = current_frame_nanos();
    let motion = remember(|| ButtonMotion::new(target, frame));
    if motion.with(|motion| motion.target) != target {
        motion.with_mut(|motion| motion.retarget(target, frame));
    }
    if !motion.with(ButtonMotion::is_idle) {
        receive_frame_nanos(move |frame| {
            motion.with_mut(|motion| motion.advance(frame));
            if motion.with(ButtonMotion::is_idle) {
                FrameNanosControl::Stop
            } else {
                FrameNanosControl::Continue
            }
        });
    }
    let container_color = color.unwrap_or(default_container_color);
    let content_color = content_color
        .or_else(|| color.and_then(|color| content_color_for(color, &scheme)))
        .unwrap_or(default_content_color);
    let button_args = ButtonResolvedArgs {
        enabled,
        modifier: modifier.unwrap_or_default(),
        color: container_color,
        content_color: Some(content_color),
        shape: shape.unwrap_or_else(|| motion.with(ButtonMotion::shape)),
        padding: content_padding
            .or_else(|| padding.map(Padding::all))
            .unwrap_or_else(|| size.content_padding()),
        on_click,
        ripple_color: ripple_color.unwrap_or(if selected.is_some() {
            content_color
        } else {
            default_ripple_color
        }),
        border_width: Dp(motion.with(|motion| motion.outline.value()).max(0.0) as f64),
        border_color: border_color.or(default_border_color),
        elevation: Some(Dp(
            motion.with(|motion| motion.elevation.value()).max(0.0) as f64
        )),
        tonal_elevation: tonal_elevation.unwrap_or(Dp(0.0)),
        disabled_container_color: disabled_container_color
            .unwrap_or(default_disabled_container_color),
        disabled_content_color: disabled_content_color
            .unwrap_or_else(|| ButtonDefaults::disabled_content_color(&scheme)),
        disabled_border_color: disabled_border_color.unwrap_or(default_disabled_border_color),
        accessibility_label,
        accessibility_description,
        child,
    };
    let child = button_args.child;
    let padding = button_args.padding;
    let text_style = size.text_style(
        &use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get(),
    );
    let child = RenderSlot::new(move || {
        layout()
            .modifier(Modifier::new().padding(padding))
            .child(move || {
                provide_context(
                    || text_style,
                    move || {
                        let inherited = use_context::<TextStyle>().expect("button text style");
                        if inherited.get() != text_style {
                            inherited.set(text_style);
                        }
                        if leading_icon.is_some() || trailing_icon.is_some() {
                            layout()
                                .policy(ButtonContentLayout {
                                    leading: leading_icon.is_some(),
                                    label: child.is_some(),
                                    trailing: trailing_icon.is_some(),
                                    icon_size: size.icon_size().to_px(),
                                    spacing: size.icon_spacing().to_px(),
                                })
                                .child(move || {
                                    if let Some(icon) = leading_icon {
                                        boxed()
                                            .modifier(
                                                Modifier::new()
                                                    .size(size.icon_size(), size.icon_size()),
                                            )
                                            .children(move || {
                                                provide_context(
                                                    || IconSize(size.icon_size()),
                                                    move || {
                                                        let inherited = use_context::<IconSize>()
                                                            .expect("button icon size");
                                                        if inherited.get().0 != size.icon_size() {
                                                            inherited
                                                                .set(IconSize(size.icon_size()));
                                                        }
                                                        icon.render();
                                                    },
                                                );
                                            });
                                    }
                                    if let Some(child) = child {
                                        boxed().children_shared(child);
                                    }
                                    if let Some(icon) = trailing_icon {
                                        boxed()
                                            .modifier(
                                                Modifier::new()
                                                    .size(size.icon_size(), size.icon_size()),
                                            )
                                            .children(move || {
                                                provide_context(
                                                    || IconSize(size.icon_size()),
                                                    move || {
                                                        let inherited = use_context::<IconSize>()
                                                            .expect("button icon size");
                                                        if inherited.get().0 != size.icon_size() {
                                                            inherited
                                                                .set(IconSize(size.icon_size()));
                                                        }
                                                        icon.render();
                                                    },
                                                );
                                            });
                                    }
                                });
                        } else if let Some(child) = child {
                            child.render();
                        }
                    },
                );
            });
    });
    let inherited_content_color = use_context::<ContentColor>()
        .map(|c| c.get().current)
        .unwrap_or(ContentColor::default().current);

    let container_color = if button_args.enabled {
        button_args.color
    } else {
        button_args.disabled_container_color
    };

    let content_color = if button_args.enabled {
        button_args.content_color.unwrap_or_else(|| {
            content_color_for(button_args.color, &scheme).unwrap_or(inherited_content_color)
        })
    } else {
        button_args.disabled_content_color
    };

    let style = if button_args.border_width.to_pixels_f32() > 0.0 {
        let border_color = if button_args.enabled {
            button_args.border_color.unwrap_or(container_color)
        } else {
            button_args.disabled_border_color
        };
        SurfaceStyle::FilledOutlined {
            fill_color: container_color,
            border_color,
            border_width: button_args.border_width,
        }
    } else {
        SurfaceStyle::Filled {
            color: container_color,
        }
    };
    let on_click = button_args.on_click;

    surface()
        .style(style)
        .shape(button_args.shape)
        .modifier(
            button_args
                .modifier
                .clone()
                .default_min_size(ButtonDefaults::MIN_WIDTH, size.height()),
        )
        .ripple_color(button_args.ripple_color)
        .content_alignment(Alignment::Center)
        .content_color(content_color)
        .enabled(button_args.enabled)
        .tonal_elevation(button_args.tonal_elevation)
        .interaction_state(interaction_state)
        .accessibility_toggled_optional(selected.map(|selected| {
            if selected {
                Toggled::True
            } else {
                Toggled::False
            }
        }))
        .accessibility_role(Role::Button)
        .accessibility_focusable(button_args.enabled)
        .elevation_optional(button_args.elevation)
        .on_click_optional(on_click)
        .accessibility_label_optional(button_args.accessibility_label)
        .accessibility_description_optional(button_args.accessibility_description)
        .child_shared(child);
}

#[derive(Clone, Copy, PartialEq)]
struct ButtonContentLayout {
    leading: bool,
    label: bool,
    trailing: bool,
    icon_size: Px,
    spacing: Px,
}

impl tessera_ui::LayoutModifierNode for ButtonContentLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let children = input.children();
        let expected =
            usize::from(self.leading) + usize::from(self.label) + usize::from(self.trailing);
        if children.len() != expected {
            return Err(MeasurementError::MeasureFnFailed(
                "Button slots require one grouped child each".to_string(),
            ));
        }
        let parent = *input.parent_constraint().as_ref();
        let icons = i32::from(self.leading) + i32::from(self.trailing);
        let gaps = expected.saturating_sub(1) as i32;
        let icon_budget = parent
            .width
            .resolve_max()
            .map_or(self.icon_size * icons, |max| {
                max.min(self.icon_size * icons)
            });
        let icon_width = icon_budget / icons;
        let spacing = if gaps == 0 {
            Px::ZERO
        } else {
            parent.width.resolve_max().map_or(self.spacing, |max| {
                self.spacing.min((max - icon_budget).max(Px::ZERO) / gaps)
            })
        };
        let mut sizes = vec![
            ComputedData {
                width: Px::ZERO,
                height: Px::ZERO
            };
            expected
        ];
        let mut icon_total = Px::ZERO;
        for index in 0..expected {
            if (self.leading && index == 0) || (self.trailing && index == expected - 1) {
                sizes[index] = children[index]
                    .measure(
                        input,
                        &Constraint::new(
                            AxisConstraint::new(Px::ZERO, Some(icon_width)),
                            parent.height.without_min(),
                        ),
                    )?
                    .size();
                icon_total += sizes[index].width;
            }
        }
        if self.label {
            let index = usize::from(self.leading);
            sizes[index] = children[index]
                .measure(
                    input,
                    &Constraint::new(
                        AxisConstraint::new(
                            Px::ZERO,
                            parent
                                .width
                                .resolve_max()
                                .map(|max| (max - icon_total - spacing * gaps).max(Px::ZERO)),
                        ),
                        parent.height.without_min(),
                    ),
                )?
                .size();
        }
        let content_width = sizes
            .iter()
            .fold(spacing * gaps, |width, size| width + size.width);
        let content_height = sizes
            .iter()
            .fold(Px::ZERO, |height, size| height.max(size.height));
        let width = parent.width.clamp(content_width);
        let height = parent.height.clamp(content_height);
        let mut result = LayoutResult::new(ComputedData { width, height });
        let mut x = Px::ZERO;
        for (child, size) in children.into_iter().zip(sizes) {
            result.place_child(
                child,
                PxPosition::new(x, (height - size.height).max(Px::ZERO) / 2),
            );
            x += size.width + spacing;
        }
        Ok(result)
    }
}

impl ButtonBuilder {
    /// Uses fully rounded ends in the unselected resting state.
    pub fn round(self) -> Self {
        self.shape_style(ButtonShape::Round)
    }

    /// Uses the size-specific square corners in the unselected resting state.
    pub fn square(self) -> Self {
        self.shape_style(ButtonShape::Square)
    }

    /// Applies the standard "Filled" button preset (High emphasis).
    ///
    /// Uses the theme Primary container with OnPrimary content.
    pub fn filled(self) -> Self {
        self.variant(ButtonVariant::Filled)
    }

    /// Applies the "Elevated" button preset (Medium emphasis).
    ///
    /// Uses the theme SurfaceContainerLow container with a shadow.
    pub fn elevated(self) -> Self {
        self.variant(ButtonVariant::Elevated)
    }

    /// Applies the "Tonal" button preset (Medium emphasis).
    ///
    /// Uses the theme SecondaryContainer container with OnSecondaryContainer
    /// content.
    pub fn tonal(self) -> Self {
        self.variant(ButtonVariant::Tonal)
    }

    /// Applies the "Outlined" button preset (Medium emphasis).
    ///
    /// Uses a transparent container with an Outline border.
    pub fn outlined(self) -> Self {
        self.variant(ButtonVariant::Outlined)
    }

    /// Applies the "Text" button preset (Low emphasis).
    ///
    /// Uses a transparent container and no border.
    pub fn text(self) -> Self {
        self.variant(ButtonVariant::Text)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

    use tessera_ui::{
        HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition, State,
        remember,
    };

    use crate::{
        column::column, icon::icon, material_icons::filled, modifier::SemanticsArgs,
        spacer::spacer, text::text, theme::material_theme,
    };

    use super::*;

    const SIZES: [ButtonSize; 5] = [
        ButtonSize::ExtraSmall,
        ButtonSize::Small,
        ButtonSize::Medium,
        ButtonSize::Large,
        ButtonSize::ExtraLarge,
    ];

    #[derive(Clone, Copy, Default)]
    struct Settings {
        disabled: bool,
        variant: ButtonVariant,
        size: Option<ButtonSize>,
        color: Option<Color>,
        content: Option<Color>,
        shape: ButtonShape,
        selected: Option<bool>,
        custom_shape: Option<Shape>,
    }

    thread_local! {
        static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
        static INTERACTIONS: RefCell<[Option<State<InteractionState>>; 5]> = const { RefCell::new([None; 5]) };
        static ACTIVATED: Cell<Option<State<Vec<usize>>>> = const { Cell::new(None) };
        static CLICKS: Cell<Option<State<usize>>> = const { Cell::new(None) };
        static STYLES: RefCell<[Option<TextStyle>; 5]> = const { RefCell::new([None; 5]) };
        static COLORS: RefCell<[Option<Color>; 5]> = const { RefCell::new([None; 5]) };
    }

    #[tessera]
    fn label(index: usize) {
        let style = use_context::<TextStyle>().expect("button text style").get();
        STYLES.with(|styles| styles.borrow_mut()[index] = Some(style));
        let color = use_context::<ContentColor>().unwrap().get().current;
        COLORS.with(|colors| colors.borrow_mut()[index] = Some(color));
        text().content("Go");
    }

    #[tessera]
    fn icon_probe(index: usize) {
        boxed()
            .modifier(Modifier::new().semantics(SemanticsArgs {
                role: Some(Role::Image),
                label: Some(format!("Icon {index}")),
                ..Default::default()
            }))
            .children(|| {
                icon().vector(filled::ADD_SVG);
            });
    }

    #[tessera]
    fn fixture() {
        let settings = remember(Settings::default);
        let clicks = remember(|| 0usize);
        let activated = remember(Vec::<usize>::new);
        ACTIVATED.set(Some(activated));
        SETTINGS.set(Some(settings));
        CLICKS.set(Some(clicks));
        material_theme().child(move || {
            boxed()
                .modifier(Modifier::new().fill_max_size().background(Color::WHITE))
                .children(move || {
                    column().children(move || {
                        for (index, size) in SIZES.into_iter().enumerate() {
                            let value = settings.get();
                            let interaction = remember(InteractionState::new);
                            INTERACTIONS
                                .with(|states| states.borrow_mut()[index] = Some(interaction));
                            button()
                                .size(if index == 2 {
                                    value.size.unwrap_or(size)
                                } else {
                                    size
                                })
                                .modifier(Modifier::new().width(Dp(260.0)))
                                .variant(value.variant)
                                .shape_style(value.shape)
                                .shape_optional(value.custom_shape)
                                .selected_optional(value.selected)
                                .interaction_state(interaction)
                                .color_optional(value.color)
                                .content_color_optional(value.content)
                                .enabled(!value.disabled)
                                .accessibility_label(format!("Button {index}"))
                                .on_click(move || {
                                    clicks.with_mut(|value| *value += 1);
                                    activated.with_mut(|items| items.push(index));
                                    settings.with_mut(|settings| {
                                        if let Some(selected) = &mut settings.selected {
                                            *selected = !*selected;
                                        }
                                    });
                                })
                                .leading_icon(move || {
                                    icon_probe(index);
                                })
                                .trailing_icon(move || {
                                    icon_probe(index + 5);
                                })
                                .child(move || {
                                    label(index);
                                });
                            spacer().modifier(Modifier::new().height(Dp(8.0)));
                        }
                    });
                });
        });
    }

    fn bounds(
        tree: &tessera_ui::accesskit::TreeUpdate,
        index: usize,
    ) -> tessera_ui::accesskit::Rect {
        tree.nodes
            .iter()
            .find(|(_, node)| node.label() == Some(format!("Button {index}").as_str()))
            .expect("button node")
            .1
            .bounds()
            .expect("button bounds")
    }

    fn click(renderer: &mut HeadlessRenderer, rect: tessera_ui::accesskit::Rect) {
        let point = PxPosition::from_f32_xy(
            ((rect.x0 + rect.x1) * 0.5) as f32,
            ((rect.y0 + rect.y1) * 0.5) as f32,
        );
        renderer.pointer_move(MOUSE_POINTER_ID, point);
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.step_frame();
        renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(8);
    }

    fn check_sizes(samples: u32) {
        let mut renderer = HeadlessRenderer::new(
            Box::new(|| {
                fixture();
            }),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: 400,
                height: 640,
                sample_count: samples,
                ..Default::default()
            },
        )
        .expect("button renderer");
        renderer.render_frames(8);
        let tree = renderer
            .accessibility_snapshot()
            .expect("button accessibility");
        let pixels = renderer.read_pixels().expect("button pixels");
        if let Some(directory) = std::env::var_os("TESSERA_BUTTON_QA_DIR") {
            let directory = std::path::PathBuf::from(directory);
            std::fs::create_dir_all(&directory).unwrap();
            tessera_ui::headless::write_png_file(
                directory.join(format!("button-sizes-msaa{samples}.png")),
                400,
                640,
                &pixels,
            )
            .unwrap();
            std::fs::write(
                directory.join(format!("button-sizes-msaa{samples}.json")),
                renderer.snapshot().to_string(),
            )
            .unwrap();
        }
        for (index, height) in [32usize, 40, 56, 96, 176].into_iter().enumerate() {
            let rect = bounds(&tree, index);
            assert_eq!(rect.width(), 260.0);
            assert_eq!(rect.height(), height.max(48) as f64);
            let ink: Vec<_> = (rect.y0 as usize..rect.y1 as usize)
                .filter(|y| {
                    let start = (y * 400 + 130) * 4;
                    let background = (y * 400 + 300) * 4;
                    pixels[start..start + 3] != pixels[background..background + 3]
                })
                .collect();
            assert_eq!(
                ink.last().unwrap() - ink.first().unwrap() + 1,
                height,
                "visual height of size {index}; style {:?}",
                STYLES.with(|styles| styles.borrow()[index])
            );
            for icon in [index, index + 5] {
                let icon_bounds = tree
                    .nodes
                    .iter()
                    .find(|(_, node)| node.label() == Some(format!("Icon {icon}").as_str()))
                    .expect("sized icon")
                    .1
                    .bounds()
                    .unwrap();
                assert_eq!(icon_bounds.width(), [20.0, 20.0, 24.0, 32.0, 40.0][index]);
                assert_eq!(icon_bounds.height(), icon_bounds.width());
            }
            let style = STYLES.with(|styles| styles.borrow()[index].unwrap());
            let label = tree
                .nodes
                .iter()
                .filter(|(_, node)| node.label() == Some("Go"))
                .filter_map(|(_, node)| node.bounds())
                .find(|bounds| bounds.y0 >= rect.y0 && bounds.y1 <= rect.y1)
                .expect("button label");
            assert!(label.x0 >= rect.x0 && label.x1 <= rect.x1);
            assert_eq!(label.height(), [20.0, 20.0, 24.0, 32.0, 80.0][index]);
            assert_eq!(style.font_size, Dp([14.0, 14.0, 16.0, 24.0, 32.0][index]));
            assert_eq!(
                style.line_height,
                Some(Dp([20.0, 20.0, 24.0, 32.0, 40.0][index]))
            );
        }
        let settings = SETTINGS.get().unwrap();
        settings.with_mut(|settings| settings.variant = ButtonVariant::Elevated);
        renderer.render_frames(8);
        let scheme = MaterialTheme::default().color_scheme;
        COLORS.with(|colors| {
            assert_eq!(
                *colors.borrow(),
                [Some(scheme.primary); 5],
                "elevated labels retain their primary color"
            )
        });
        let elevated = renderer.read_pixels().unwrap();
        let edge = (2 * 400 + 130) * 4;
        let background = (2 * 400 + 300) * 4;
        let fringe: u32 = elevated[edge..edge + 3]
            .iter()
            .zip(&elevated[background..background + 3])
            .map(|(a, b)| u32::from(a.abs_diff(*b)))
            .sum();
        assert!(
            fringe <= 9,
            "the shadow must follow the 32dp visual container, not the 48dp touch target: {fringe}"
        );
        settings.with_mut(|settings| settings.color = Some(scheme.secondary_container));
        renderer.render_frames(8);
        COLORS.with(|colors| {
            assert_eq!(
                *colors.borrow(),
                [Some(scheme.on_secondary_container); 5],
                "custom containers infer a matching content color"
            )
        });
        settings.with_mut(|settings| settings.content = Some(Color::RED));
        renderer.render_frames(8);
        COLORS.with(|colors| {
            assert_eq!(
                *colors.borrow(),
                [Some(Color::RED); 5],
                "explicit content color wins"
            )
        });
        settings.with_mut(|settings| {
            settings.variant = ButtonVariant::Filled;
            settings.color = None;
            settings.content = None;
        });
        renderer.render_frames(8);
        let clicks = CLICKS.get().unwrap();
        click(&mut renderer, bounds(&tree, 0));
        assert_eq!(clicks.get(), 1);
        settings.with_mut(|settings| settings.disabled = true);
        renderer.render_frames(8);
        let disabled = renderer.accessibility_snapshot().unwrap();
        for index in 0..5 {
            assert_eq!(
                bounds(&disabled, index),
                bounds(&tree, index),
                "disabling preserves layout"
            );
        }
        click(&mut renderer, bounds(&disabled, 0));
        assert_eq!(clicks.get(), 1);
        settings.with_mut(|settings| {
            settings.disabled = false;
            settings.variant = ButtonVariant::Outlined;
        });
        renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(390.0, 600.0));
        renderer.render_frames(120);
        COLORS.with(|colors| {
            assert_eq!(
                *colors.borrow(),
                [Some(scheme.on_surface_variant); 5],
                "outlined labels use on-surface-variant"
            )
        });
        let outlined = renderer.read_pixels().unwrap();
        for (index, width) in [1usize, 1, 1, 2, 3].into_iter().enumerate() {
            let rect = bounds(&tree, index);
            let y = ((rect.y0 + rect.y1) * 0.5) as usize;
            let start = (y * 400 + width - 1) * 4;
            let contrast: u32 = outlined[start..start + 3]
                .iter()
                .map(|channel| 255 - u32::from(*channel))
                .sum();
            assert!(
                contrast > 35,
                "outline thickness for size {index}: {contrast}"
            );
            let inner = (y * 400 + width + 2) * 4;
            let fill = (y * 400 + 50) * 4;
            assert_eq!(&outlined[inner..inner + 4], &outlined[fill..fill + 4]);
        }
        settings.with_mut(|settings| settings.size = Some(ButtonSize::ExtraLarge));
        renderer.render_frames(8);
        assert_eq!(
            bounds(&renderer.accessibility_snapshot().unwrap(), 2).height(),
            176.0
        );
        assert_eq!(
            STYLES.with(|styles| styles.borrow()[2].unwrap().font_size),
            Dp(32.0)
        );
        let changed = renderer.accessibility_snapshot().unwrap();
        let icon = changed
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some("Icon 2"))
            .unwrap()
            .1
            .bounds()
            .unwrap();
        assert_eq!(icon.width(), 40.0);
        assert_eq!(icon.height(), 40.0);
    }

    #[test]
    fn shape_motion_keeps_velocity_and_resolves_against_actual_bounds() {
        use tessera_ui::{Px, PxSize};

        for size in SIZES {
            for style in [ButtonShape::Round, ButtonShape::Square] {
                let (fraction, radius) = ButtonMotionTarget::corner(size, style, false, false);
                let target = ButtonMotionTarget {
                    fraction,
                    radius,
                    elevation: 0.0,
                    outline: 1.0,
                    toggle: false,
                };
                let mut motion = ButtonMotion::new(target, 0);
                let (fraction, radius) = ButtonMotionTarget::corner(size, style, false, true);
                motion.retarget(
                    ButtonMotionTarget {
                        fraction,
                        radius,
                        ..target
                    },
                    0,
                );
                motion.advance(33_333_334);
                assert!(!motion.is_idle());
                let value = motion.fraction.value();
                let velocity = motion.fraction.velocity();
                motion.retarget(target, 33_333_334);
                assert_eq!(motion.fraction.value(), value);
                assert_eq!(motion.fraction.velocity(), velocity);
                for frame in 3..150 {
                    motion.advance(frame * 16_666_667);
                }
                assert!(motion.is_idle());
                for height in [16, 40, 200] {
                    let bounds = PxSize::new(Px(300), Px(height));
                    let Shape::RoundedRectangle { top_left, .. } = motion.shape() else {
                        unreachable!()
                    };
                    let expected = if style == ButtonShape::Round {
                        height as f32 / 2.0
                    } else {
                        size.square_radius()
                            .to_pixels_f32()
                            .min(height as f32 / 2.0)
                    };
                    assert_eq!(top_left.resolve(bounds), (expected, 2.0));
                }
                let (fraction, radius) = ButtonMotionTarget::corner(size, style, true, false);
                motion.retarget(
                    ButtonMotionTarget {
                        fraction,
                        radius,
                        toggle: true,
                        ..target
                    },
                    2_500_000_000,
                );
                for frame in 151..300 {
                    motion.advance(frame * 16_666_667);
                }
                assert!(motion.is_idle());
                assert_eq!(
                    motion.fraction.value(),
                    if style == ButtonShape::Round {
                        0.0
                    } else {
                        0.5
                    }
                );
            }
        }
    }

    fn check_shape_and_toggle_input(samples: u32) {
        let mut renderer = HeadlessRenderer::new(
            Box::new(|| {
                fixture();
            }),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: 400,
                height: 640,
                sample_count: samples,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(8);
        let initial = renderer.accessibility_snapshot().unwrap();
        let round = renderer.read_pixels().unwrap();
        let settings = SETTINGS.get().unwrap();
        settings.with_mut(|settings| {
            settings.shape = ButtonShape::Square;
            settings.selected = Some(false);
        });
        renderer.render_frames(120);
        let square = renderer.read_pixels().unwrap();
        assert_ne!(round, square, "square preset changes visible corners");
        let before = renderer.accessibility_snapshot().unwrap();
        let find = |tree: &tessera_ui::accesskit::TreeUpdate| {
            tree.nodes
                .iter()
                .find(|(_, node)| node.label() == Some("Button 2"))
                .unwrap()
                .clone()
        };
        let (identity, node) = find(&before);
        assert_eq!(node.role(), Role::Button);
        assert_eq!(node.toggled(), Some(Toggled::False));
        assert!(node.supports_action(tessera_ui::accesskit::Action::Click));
        let rect = bounds(&before, 2);
        let point = PxPosition::from_f32_xy(
            ((rect.x0 + rect.x1) / 2.0) as f32,
            ((rect.y0 + rect.y1) / 2.0) as f32,
        );
        renderer.pointer_move(MOUSE_POINTER_ID, point);
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(20);
        assert!(
            INTERACTIONS
                .with(|states| states.borrow()[2].unwrap())
                .with(InteractionState::is_pressed)
        );
        assert_eq!(CLICKS.get().unwrap().get(), 0);
        assert_ne!(square, renderer.read_pixels().unwrap());
        renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.step_frame();
        renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(390.0, 630.0));
        renderer.render_frames(120);
        assert_eq!(settings.get().selected, Some(true));
        assert_eq!(CLICKS.get().unwrap().get(), 1);
        let selected = renderer.accessibility_snapshot().unwrap();
        let (selected_identity, node) = find(&selected);
        assert_eq!(identity, selected_identity);
        assert_eq!(node.toggled(), Some(Toggled::True));
        for index in 0..5 {
            assert_eq!(bounds(&initial, index), bounds(&selected, index));
        }
        let scheme = MaterialTheme::default().color_scheme;
        COLORS.with(|colors| assert_eq!(*colors.borrow(), [Some(scheme.on_primary); 5]));
        settings.with_mut(|settings| settings.disabled = true);
        renderer.render_frames(120);
        let disabled = renderer.accessibility_snapshot().unwrap();
        let (disabled_identity, node) = find(&disabled);
        assert_eq!(identity, disabled_identity);
        assert_eq!(node.toggled(), Some(Toggled::True));
        assert!(node.is_disabled());
        assert!(!node.supports_action(tessera_ui::accesskit::Action::Click));
        click(&mut renderer, rect);
        assert_eq!(CLICKS.get().unwrap().get(), 1);
        settings.with_mut(|settings| {
            settings.disabled = false;
            settings.selected = None;
        });
        renderer.render_frames(120);
        assert_eq!(
            find(&renderer.accessibility_snapshot().unwrap())
                .1
                .toggled(),
            None
        );
        renderer.pointer_move(MOUSE_POINTER_ID, point);
        renderer.step_frame();
        let pressed_at = tessera_ui::time::Instant::now();
        renderer.inject_pointer(tessera_ui::PointerChange::new(
            pressed_at,
            MOUSE_POINTER_ID,
            tessera_ui::CursorEventContent::Pressed(PressKeyEventType::Left),
            tessera_ui::GestureState::TapCandidate,
        ));
        renderer.step_frame();
        renderer.inject_pointer(tessera_ui::PointerChange::new(
            pressed_at + std::time::Duration::from_secs(2),
            MOUSE_POINTER_ID,
            tessera_ui::CursorEventContent::Released(PressKeyEventType::Left),
            tessera_ui::GestureState::TapCandidate,
        ));
        renderer.step_frame();
        assert_eq!(
            CLICKS.get().unwrap().get(),
            2,
            "an ordinary button still activates after a long hold"
        );
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.step_frame();
        let outside = PxPosition::from_f32_xy(390.0, 630.0);
        renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.pointer_move(MOUSE_POINTER_ID, outside);
        renderer.step_frame();
        assert_eq!(
            CLICKS.get().unwrap().get(),
            3,
            "release inside survives later movement in the same frame"
        );
        let button_point = |index| {
            let rect = bounds(&initial, index);
            PxPosition::from_f32_xy(
                ((rect.x0 + rect.x1) / 2.0) as f32,
                ((rect.y0 + rect.y1) / 2.0) as f32,
            )
        };
        for index in [0, 4] {
            let position = button_point(index);
            renderer.pointer_move(MOUSE_POINTER_ID, position);
            renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
            renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        }
        renderer.pointer_move(MOUSE_POINTER_ID, outside);
        renderer.render_frames(2);
        assert_eq!(
            ACTIVATED.get().unwrap().get(),
            [2, 2, 2, 0, 4],
            "rapid queued clicks target each button in order"
        );
        renderer.pointer_press(11, button_point(0), PressKeyEventType::Left);
        renderer.pointer_press(22, button_point(4), PressKeyEventType::Left);
        renderer.render_frames(2);
        for index in [0, 4] {
            assert!(
                INTERACTIONS
                    .with(|states| states.borrow()[index].unwrap())
                    .with(InteractionState::is_pressed),
                "independent held pointer on button {index}"
            );
        }
        renderer.pointer_release(11, button_point(0), PressKeyEventType::Left);
        renderer.pointer_release(22, button_point(4), PressKeyEventType::Left);
        renderer.pointer_move(MOUSE_POINTER_ID, outside);
        renderer.render_frames(2);
        assert_eq!(ACTIVATED.get().unwrap().get(), [2, 2, 2, 0, 4, 0, 4]);
        if let Some(directory) = std::env::var_os("TESSERA_BUTTON_QA_DIR") {
            let directory = std::path::PathBuf::from(directory);
            std::fs::create_dir_all(&directory).unwrap();
            tessera_ui::headless::write_png_file(
                directory.join(format!("button-square-msaa{samples}.png")),
                400,
                640,
                &square,
            )
            .unwrap();
        }
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn expressive_shape_toggle_semantics_and_input_msaa1() {
        check_shape_and_toggle_input(1);
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn expressive_shape_toggle_semantics_and_input_msaa4() {
        check_shape_and_toggle_input(4);
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn expressive_size_geometry_typography_outline_and_disabled_layout_msaa1() {
        check_sizes(1);
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn expressive_size_geometry_typography_outline_and_disabled_layout_msaa4() {
        check_sizes(4);
    }
}
