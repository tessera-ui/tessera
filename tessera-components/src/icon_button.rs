//! Icon actions with Expressive sizes, widths, and selection states.
//!
//! ## Usage
//!
//! Offer compact actions and toggles in toolbars, navigation, and media
//! controls.
use tessera_ui::{
    Callback, Color, Dp, FrameNanosControl, Modifier, State,
    accesskit::{Role, Toggled},
    current_frame_nanos, receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    alignment::Alignment,
    animation::Animatable,
    boxed::boxed,
    modifier::{InteractionState, InteractionVisualState, ModifierExt as _},
    painter::Painter,
    shape_def::{RoundedCorner, Shape},
    surface::{RipplePresentation, SurfaceStyle, surface},
    theme::{ContentColor, MaterialColorScheme, MaterialTheme, content_color_for},
};

/// Visual emphasis of an icon action or toggle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IconButtonVariant {
    /// Transparent container with no outline.
    Standard,
    /// Primary container, or a surface container for an unselected toggle.
    #[default]
    Filled,
    /// Secondary container with higher emphasis when selected.
    FilledTonal,
    /// Outlined container that becomes filled when selected.
    Outlined,
}

/// Expressive visual dimensions of an icon button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IconButtonSize {
    /// A 32dp container with a 20dp icon.
    ExtraSmall,
    /// A 40dp container with a 24dp icon.
    #[default]
    Small,
    /// A 56dp container with a 24dp icon.
    Medium,
    /// A 96dp container with a 32dp icon.
    Large,
    /// A 136dp container with a 40dp icon.
    ExtraLarge,
}

/// Horizontal proportions of an icon button's visual container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IconButtonWidth {
    /// Reduced horizontal insets.
    Narrow,
    /// Equal visual width and height.
    #[default]
    Uniform,
    /// Expanded horizontal insets.
    Wide,
}

/// Resting shape of an icon button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IconButtonShape {
    /// Fully rounded corners, changing to size-specific corners when selected.
    #[default]
    Round,
    /// Size-specific corners, changing to fully rounded corners when selected.
    Square,
}

/// Color source for standard and outlined icon buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IconButtonColorStyle {
    /// Recommended theme colors with a primary selected standard icon.
    #[default]
    Vibrant,
    /// Inherit the surrounding content color for unselected icons and outlines.
    Inherited,
}

impl IconButtonSize {
    /// Returns the visual height, excluding minimum touch-target expansion.
    pub const fn height(self) -> Dp {
        Dp(match self {
            Self::ExtraSmall => 32.0,
            Self::Small => 40.0,
            Self::Medium => 56.0,
            Self::Large => 96.0,
            Self::ExtraLarge => 136.0,
        })
    }

    /// Returns the recommended icon size.
    pub const fn icon_size(self) -> Dp {
        Dp(match self {
            Self::ExtraSmall => 20.0,
            Self::Small | Self::Medium => 24.0,
            Self::Large => 32.0,
            Self::ExtraLarge => 40.0,
        })
    }

    /// Returns the visual width for the requested proportions.
    pub const fn width(self, width: IconButtonWidth) -> Dp {
        let (narrow, uniform, wide) = match self {
            Self::ExtraSmall => (28.0, 32.0, 40.0),
            Self::Small => (32.0, 40.0, 52.0),
            Self::Medium => (48.0, 56.0, 72.0),
            Self::Large => (64.0, 96.0, 128.0),
            Self::ExtraLarge => (104.0, 136.0, 184.0),
        };
        Dp(match width {
            IconButtonWidth::Narrow => narrow,
            IconButtonWidth::Uniform => uniform,
            IconButtonWidth::Wide => wide,
        })
    }

    /// Returns the outline thickness.
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

    /// Returns the pressed corner radius for either shape preset.
    pub const fn pressed_radius(self) -> Dp {
        Dp(match self {
            Self::ExtraSmall | Self::Small => 8.0,
            Self::Medium => 12.0,
            Self::Large | Self::ExtraLarge => 16.0,
        })
    }
}

impl IconButtonBuilder {
    /// Uses fully rounded corners in the unselected resting state.
    pub fn round(self) -> Self {
        self.shape_style(IconButtonShape::Round)
    }

    /// Uses size-specific corners in the unselected resting state.
    pub fn square(self) -> Self {
        self.shape_style(IconButtonShape::Square)
    }

    /// Applies the standard icon button preset.
    pub fn standard(self) -> Self {
        self.variant(IconButtonVariant::Standard)
    }

    /// Applies the filled icon button preset.
    pub fn filled(self) -> Self {
        self.variant(IconButtonVariant::Filled)
    }

    /// Applies the filled tonal icon button preset.
    pub fn filled_tonal(self) -> Self {
        self.variant(IconButtonVariant::FilledTonal)
    }

    /// Applies the outlined icon button preset.
    pub fn outlined(self) -> Self {
        self.variant(IconButtonVariant::Outlined)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct IconButtonCorner {
    fraction: f32,
    radius: f32,
}

impl IconButtonCorner {
    fn new(size: IconButtonSize, shape: IconButtonShape, selected: bool, pressed: bool) -> Self {
        let (fraction, radius) = if pressed {
            (0.0, size.pressed_radius().0 as f32)
        } else if (shape == IconButtonShape::Round) != selected {
            (0.5, 0.0)
        } else {
            (0.0, size.square_radius().0 as f32)
        };
        Self { fraction, radius }
    }
}

struct IconButtonMotion {
    fraction: Animatable,
    radius: Animatable,
    target: IconButtonCorner,
    frame_nanos: u64,
}

impl IconButtonMotion {
    fn new(target: IconButtonCorner, frame_nanos: u64) -> Self {
        Self {
            fraction: Animatable::spring(target.fraction, 40.0, 1.0),
            radius: Animatable::spring(target.radius, 40.0, 1.0),
            target,
            frame_nanos,
        }
    }

    fn retarget(&mut self, target: IconButtonCorner, frame_nanos: u64) {
        self.advance(frame_nanos);
        self.fraction.set_target(target.fraction);
        self.radius.set_target(target.radius);
        self.target = target;
    }

    fn advance(&mut self, frame_nanos: u64) {
        if frame_nanos > self.frame_nanos {
            let dt = (frame_nanos - self.frame_nanos) as f32 / 1_000_000_000.0;
            self.fraction.tick(dt);
            self.radius.tick(dt);
            self.frame_nanos = frame_nanos;
        }
    }

    fn is_idle(&self) -> bool {
        self.fraction.is_idle() && self.radius.is_idle()
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

#[derive(Clone, Copy, Debug, PartialEq)]
struct IconButtonColors {
    container: Color,
    content: Color,
    border: Option<Color>,
}

fn resolve_colors(
    scheme: &MaterialColorScheme,
    variant: IconButtonVariant,
    selected: Option<bool>,
    enabled: bool,
    inherited: Option<Color>,
) -> IconButtonColors {
    let checked = selected == Some(true);
    let outlined = variant == IconButtonVariant::Outlined && !checked;
    let container = if !enabled {
        if matches!(
            variant,
            IconButtonVariant::Filled | IconButtonVariant::FilledTonal
        ) || (variant == IconButtonVariant::Outlined && checked)
        {
            scheme.on_surface.with_alpha(0.1)
        } else {
            Color::TRANSPARENT
        }
    } else {
        match (variant, selected) {
            (IconButtonVariant::Filled, Some(false)) => scheme.surface_container,
            (IconButtonVariant::Filled, _) => scheme.primary,
            (IconButtonVariant::FilledTonal, Some(true)) => scheme.secondary,
            (IconButtonVariant::FilledTonal, _) => scheme.secondary_container,
            (IconButtonVariant::Outlined, Some(true)) => scheme.inverse_surface,
            _ => Color::TRANSPARENT,
        }
    };
    let content = if !enabled {
        let base = if matches!(
            variant,
            IconButtonVariant::Standard | IconButtonVariant::Outlined
        ) {
            inherited.unwrap_or(scheme.on_surface)
        } else {
            scheme.on_surface
        };
        base.with_alpha(0.38)
    } else {
        match (variant, selected) {
            (IconButtonVariant::Filled, Some(false)) => scheme.on_surface_variant,
            (IconButtonVariant::Filled, _) => scheme.on_primary,
            (IconButtonVariant::FilledTonal, Some(true)) => scheme.on_secondary,
            (IconButtonVariant::FilledTonal, _) => scheme.on_secondary_container,
            (IconButtonVariant::Outlined, Some(true)) => scheme.inverse_on_surface,
            (IconButtonVariant::Standard, Some(true)) => scheme.primary,
            _ => inherited.unwrap_or(scheme.on_surface_variant),
        }
    };
    let border = outlined.then(|| {
        let base = inherited.unwrap_or(scheme.outline_variant);
        if enabled { base } else { base.with_alpha(0.38) }
    });
    IconButtonColors {
        container,
        content,
        border,
    }
}

/// # icon_button
///
/// Display a compact action or controlled toggle with Expressive sizes and
/// shapes.
///
/// ## Usage
///
/// Offer named icon actions and persistent choices in toolbars and media
/// controls.
///
/// ## Parameters
///
/// - `variant` — visual emphasis, defaulting to filled.
/// - `size` / `width` — visual dimensions, defaulting to small and uniform.
/// - `shape_style` — round or square resting corners; selection swaps the
///   shapes.
/// - `selected` — controlled toggle state; omit for an ordinary action.
/// - `icon` — optional vector or raster icon, centered at the size's icon
///   dimensions.
/// - `on_click` — activation callback; update `selected` here when using a
///   toggle.
/// - `enabled` — whether activation, focus, and interaction feedback are
///   enabled.
/// - `modifier` — outer constraints and node-local behavior.
/// - `shape` — fixed shape override that disables automatic corner transitions.
/// - `color_style` — vibrant or inherited colors for standard and outlined
///   variants.
/// - `color` / `content_color` — enabled container and icon color overrides.
/// - `interaction_state` — optional shared pointer and focus state.
/// - `presentation_ripple` — optional static circular ripple for pressed visual
///   presentation.
/// - `presentation_state` — optional visual interaction without changing input
///   or focus.
/// - `accessibility_label` / `accessibility_description` — action name and
///   supporting help.
///
/// Interactive targets expand to at least 48dp when parent constraints permit.
/// Disabled controls retain their target geometry and toggle state.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     icon_button::icon_button, material_icons::filled, theme::material_theme,
/// };
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn component() {
///     let selected = remember(|| false);
///     assert!(!selected.get());
///     material_theme().child(move || {
///         icon_button()
///             .icon(filled::STAR_SVG)
///             .filled_tonal()
///             .selected(selected.get())
///             .accessibility_label("Favorite")
///             .on_click(move || selected.set(!selected.get()));
///     });
/// }
/// component();
/// ```
#[tessera]
pub fn icon_button(
    variant: Option<IconButtonVariant>,
    size: Option<IconButtonSize>,
    width: Option<IconButtonWidth>,
    shape_style: Option<IconButtonShape>,
    selected: Option<bool>,
    #[prop(into)] icon: Option<Painter>,
    on_click: Option<Callback>,
    enabled: Option<bool>,
    modifier: Option<Modifier>,
    shape: Option<Shape>,
    color_style: Option<IconButtonColorStyle>,
    color: Option<Color>,
    content_color: Option<Color>,
    interaction_state: Option<State<InteractionState>>,
    presentation_state: Option<InteractionVisualState>,
    presentation_ripple: Option<RipplePresentation>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let variant = variant.unwrap_or_default();
    let size = size.unwrap_or_default();
    let width = width.unwrap_or_default();
    let enabled = enabled.unwrap_or(true);
    let interactive = enabled && on_click.is_some();
    let interaction_state = if let Some(state) = interaction_state {
        state
    } else {
        remember(InteractionState::new)
    };
    if !interactive && interaction_state.with(|state| *state != InteractionState::new()) {
        interaction_state.set(InteractionState::new());
    }
    let visual =
        interaction_state.with(|state| state.with_presentation(presentation_state, enabled));
    let target = if shape.is_some() {
        IconButtonCorner {
            fraction: 0.0,
            radius: 0.0,
        }
    } else {
        IconButtonCorner::new(
            size,
            shape_style.unwrap_or_default(),
            selected.unwrap_or(false),
            visual.is_pressed(),
        )
    };
    let frame = current_frame_nanos();
    let motion = remember(|| IconButtonMotion::new(target, frame));
    if shape.is_some() {
        if !motion.with(IconButtonMotion::is_idle) || motion.with(|motion| motion.target) != target
        {
            motion.set(IconButtonMotion::new(target, frame));
        }
    } else if motion.with(|motion| motion.target) != target {
        motion.with_mut(|motion| motion.retarget(target, frame));
    }
    if !motion.with(IconButtonMotion::is_idle) {
        receive_frame_nanos(move |frame| {
            motion.with_mut(|motion| motion.advance(frame));
            if motion.with(IconButtonMotion::is_idle) {
                FrameNanosControl::Stop
            } else {
                FrameNanosControl::Continue
            }
        });
    }
    let inherited = if color_style.unwrap_or_default() == IconButtonColorStyle::Inherited {
        Some(
            use_context::<ContentColor>()
                .map(|color| color.get().current)
                .unwrap_or(scheme.on_surface),
        )
    } else {
        None
    };
    let defaults = resolve_colors(&scheme, variant, selected, enabled, inherited);
    let container = if enabled {
        color.unwrap_or(defaults.container)
    } else {
        defaults.container
    };
    let content = if enabled {
        content_color
            .or_else(|| color.and_then(|color| content_color_for(color, &scheme)))
            .unwrap_or(defaults.content)
    } else {
        defaults.content
    };
    let style = if let Some(border_color) = defaults.border {
        SurfaceStyle::FilledOutlined {
            fill_color: container,
            border_color,
            border_width: size.outline_width(),
        }
    } else {
        SurfaceStyle::Filled { color: container }
    };
    surface()
        .modifier(modifier.unwrap_or_default())
        .shape(shape.unwrap_or_else(|| motion.with(IconButtonMotion::shape)))
        .style(style)
        .content_alignment(Alignment::Center)
        .content_color(content)
        .enabled(enabled)
        .interaction_state(interaction_state)
        .presentation_state_optional(presentation_state)
        .presentation_ripple_optional(presentation_ripple)
        .show_focus_indicator(true)
        .ripple_color(content)
        .accessibility_role(Role::Button)
        .accessibility_toggled_optional(
            selected.map(|value| if value { Toggled::True } else { Toggled::False }),
        )
        .accessibility_focusable(interactive)
        .accessibility_label_optional(accessibility_label)
        .accessibility_description_optional(accessibility_description)
        .on_click_optional(on_click)
        .child(move || {
            let icon = icon.clone();
            boxed()
                .modifier(Modifier::new().size(size.width(width), size.height()))
                .alignment(Alignment::Center)
                .children(move || {
                    if let Some(icon) = &icon {
                        crate::icon::icon()
                            .painter(icon.clone())
                            .size(size.icon_size());
                    }
                });
        });
}

#[cfg(test)]
mod tests;
