//! Floating action button component.
//!
//! ## Usage
//!
//! Emphasize a primary action with a prominent floating button.
use tessera_ui::{
    Callback, Color, Dp, FrameNanosControl, Modifier, RenderSlot, State, accesskit::Role,
    current_frame_nanos, provide_context, receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    alignment::Alignment,
    animation::Animatable,
    modifier::{InteractionState, ModifierExt as _},
    shape_def::Shape,
    surface::{SurfaceStyle, surface},
    theme::{
        ContentColor, MaterialAlpha, MaterialColorScheme, MaterialShapes, MaterialTheme,
        content_color_for,
    },
};

/// Sizes supported by [`floating_action_button`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FloatingActionButtonSize {
    /// Small floating action button (40dp).
    Small,
    /// Standard floating action button (56dp).
    #[default]
    Standard,
    /// Medium floating action button (80dp).
    Medium,
    /// Large floating action button (96dp).
    Large,
}

impl FloatingActionButtonSize {
    fn container_size(self) -> Dp {
        match self {
            FloatingActionButtonSize::Small => FloatingActionButtonDefaults::SMALL_SIZE,
            FloatingActionButtonSize::Standard => FloatingActionButtonDefaults::STANDARD_SIZE,
            FloatingActionButtonSize::Medium => FloatingActionButtonDefaults::MEDIUM_SIZE,
            FloatingActionButtonSize::Large => FloatingActionButtonDefaults::LARGE_SIZE,
        }
    }

    fn icon_size(self) -> Dp {
        match self {
            FloatingActionButtonSize::Small => FloatingActionButtonDefaults::SMALL_ICON_SIZE,
            FloatingActionButtonSize::Standard => FloatingActionButtonDefaults::STANDARD_ICON_SIZE,
            FloatingActionButtonSize::Medium => FloatingActionButtonDefaults::MEDIUM_ICON_SIZE,
            FloatingActionButtonSize::Large => FloatingActionButtonDefaults::LARGE_ICON_SIZE,
        }
    }
}

/// Elevation values used by floating action buttons across interaction states.
#[derive(Clone, PartialEq, Copy, Debug)]
pub struct FloatingActionButtonElevation {
    default_elevation: Dp,
    pressed_elevation: Dp,
    hovered_elevation: Dp,
    focused_elevation: Dp,
}

impl FloatingActionButtonElevation {
    /// Creates a custom elevation profile for a floating action button.
    pub const fn new(
        default_elevation: Dp,
        pressed_elevation: Dp,
        hovered_elevation: Dp,
        focused_elevation: Dp,
    ) -> Self {
        Self {
            default_elevation,
            pressed_elevation,
            hovered_elevation,
            focused_elevation,
        }
    }

    fn tonal_elevation(self) -> Dp {
        self.default_elevation
    }

    fn shadow_elevation(
        self,
        enabled: bool,
        interaction_state: Option<State<InteractionState>>,
    ) -> Dp {
        if !enabled {
            return self.default_elevation;
        }

        let Some(state) = interaction_state else {
            return self.default_elevation;
        };

        state.with(|state| {
            if state.is_pressed() {
                self.pressed_elevation
            } else if state.is_focused() {
                self.focused_elevation
            } else if state.is_hovered() {
                self.hovered_elevation
            } else {
                self.default_elevation
            }
        })
    }
}

struct ElevationMotion {
    motion: Animatable,
    frame_nanos: u64,
}

impl ElevationMotion {
    fn new(elevation: Dp, frame_nanos: u64) -> Self {
        Self {
            motion: Animatable::spring(elevation.0 as f32, 3800.0_f32.sqrt(), 1.0),
            frame_nanos,
        }
    }

    fn advance(&mut self, frame_nanos: u64) {
        if frame_nanos > self.frame_nanos {
            self.motion
                .tick((frame_nanos - self.frame_nanos) as f32 / 1_000_000_000.0);
            self.frame_nanos = frame_nanos;
        }
    }
}

/// Material Design defaults for [`floating_action_button`].
pub struct FloatingActionButtonDefaults;

impl FloatingActionButtonDefaults {
    /// Default container size for a standard floating action button.
    pub const STANDARD_SIZE: Dp = Dp(56.0);
    /// Container size for a small floating action button.
    pub const SMALL_SIZE: Dp = Dp(40.0);
    /// Container size for a medium floating action button.
    pub const MEDIUM_SIZE: Dp = Dp(80.0);
    /// Container size for a large floating action button.
    pub const LARGE_SIZE: Dp = Dp(96.0);
    /// Recommended icon size for a standard floating action button.
    pub const STANDARD_ICON_SIZE: Dp = Dp(24.0);
    /// Recommended icon size for a small floating action button.
    pub const SMALL_ICON_SIZE: Dp = Dp(24.0);
    /// Recommended icon size for a medium floating action button.
    pub const MEDIUM_ICON_SIZE: Dp = Dp(28.0);
    /// Recommended icon size for a large floating action button.
    pub const LARGE_ICON_SIZE: Dp = Dp(36.0);
    /// Default resting elevation for floating action buttons.
    pub const DEFAULT_ELEVATION: Dp = Dp(6.0);
    /// Elevation used while the button is pressed.
    pub const PRESSED_ELEVATION: Dp = Dp(6.0);
    /// Elevation used while the button is hovered.
    pub const HOVERED_ELEVATION: Dp = Dp(8.0);
    /// Elevation used while the button is focused.
    pub const FOCUSED_ELEVATION: Dp = Dp(6.0);

    /// Returns the container size for the provided FAB size.
    pub fn container_size(size: FloatingActionButtonSize) -> Dp {
        size.container_size()
    }

    /// Returns the recommended icon size for the provided FAB size.
    pub fn icon_size(size: FloatingActionButtonSize) -> Dp {
        size.icon_size()
    }

    /// Returns the default shape for the provided FAB size.
    pub fn shape(size: FloatingActionButtonSize) -> Shape {
        shape_from_size(size, &MaterialShapes::default())
    }

    /// Default container color for floating action buttons.
    pub fn container_color(scheme: &MaterialColorScheme) -> Color {
        scheme.primary_container
    }

    /// Default content color for floating action buttons.
    pub fn content_color(scheme: &MaterialColorScheme) -> Color {
        scheme.on_primary_container
    }

    /// Default disabled container color for floating action buttons.
    pub fn disabled_container_color(scheme: &MaterialColorScheme) -> Color {
        scheme
            .on_surface
            .with_alpha(MaterialAlpha::DISABLED_CONTAINER)
    }

    /// Default disabled content color for floating action buttons.
    pub fn disabled_content_color(scheme: &MaterialColorScheme) -> Color {
        scheme
            .on_surface
            .with_alpha(MaterialAlpha::DISABLED_CONTENT)
    }

    /// Default elevation profile for floating action buttons.
    pub fn elevation() -> FloatingActionButtonElevation {
        FloatingActionButtonElevation::new(
            Self::DEFAULT_ELEVATION,
            Self::PRESSED_ELEVATION,
            Self::HOVERED_ELEVATION,
            Self::FOCUSED_ELEVATION,
        )
    }
}

#[derive(Clone)]
struct FloatingActionButtonResolvedArgs {
    size: FloatingActionButtonSize,
    modifier: Modifier,
    enabled: bool,
    container_color: Color,
    content_color: Option<Color>,
    shape: Option<Shape>,
    elevation: FloatingActionButtonElevation,
    on_click: Option<Callback>,
    disabled_container_color: Color,
    disabled_content_color: Color,
    interaction_state: Option<State<InteractionState>>,
    ripple_color: Option<Color>,
    accessibility_label: Option<String>,
    accessibility_description: Option<String>,
    content: Option<RenderSlot>,
}

/// # floating_action_button
///
/// Render a floating action button for primary actions and quick access tasks.
///
/// ## Usage
///
/// Use for a screen's most important action, typically shown above the main
/// content.
///
/// ## Parameters
///
/// - `size` - optional floating action button size variant.
/// - `modifier` - modifier chain applied to the button.
/// - `enabled` - optional enabled flag.
/// - `container_color` - optional container color override.
/// - `content_color` - optional content color override.
/// - `shape` - optional shape override.
/// - `elevation` - optional elevation profile override.
/// - `on_click` - optional click callback.
/// - `disabled_container_color` - optional disabled container color override.
/// - `disabled_content_color` - optional disabled content color override.
/// - `interaction_state` - optional shared interaction state.
/// - `ripple_color` - optional ripple tint override.
/// - `accessibility_label` - optional accessibility label.
/// - `accessibility_description` - optional accessibility description.
/// - `content` - optional content render slot, typically an icon.
///
/// ## Examples
///
/// ```
/// use tessera_components::floating_action_button::floating_action_button;
/// use tessera_ui::{Callback, remember, tessera};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// #[tessera]
/// fn component() {
/// #     material_theme()
/// #         .theme(|| MaterialTheme::default())
/// #         .child(|| {
///     let clicked = remember(|| false);
///     let on_click = Callback::new({ move || clicked.with_mut(|value| *value = true) });
///     let on_click_shared = on_click.clone();
///     on_click.call();
///
///     assert!(clicked.with(|value| *value));
///
///     floating_action_button()
///         .on_click_shared(on_click_shared)
///         .content(|| {});
/// #         });
/// }
///
/// component();
/// ```
#[tessera]
pub fn floating_action_button(
    size: Option<FloatingActionButtonSize>,
    modifier: Option<Modifier>,
    enabled: Option<bool>,
    container_color: Option<Color>,
    content_color: Option<Color>,
    shape: Option<Shape>,
    elevation: Option<FloatingActionButtonElevation>,
    on_click: Option<Callback>,
    disabled_container_color: Option<Color>,
    disabled_content_color: Option<Color>,
    interaction_state: Option<State<InteractionState>>,
    ripple_color: Option<Color>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    content: Option<RenderSlot>,
) {
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let args = FloatingActionButtonResolvedArgs {
        size: size.unwrap_or_default(),
        modifier: modifier.unwrap_or_default(),
        enabled: enabled.unwrap_or(true),
        container_color: container_color
            .unwrap_or_else(|| FloatingActionButtonDefaults::container_color(&theme.color_scheme)),
        content_color,
        shape,
        elevation: elevation.unwrap_or_else(FloatingActionButtonDefaults::elevation),
        on_click,
        disabled_container_color: disabled_container_color.unwrap_or_else(|| {
            FloatingActionButtonDefaults::disabled_container_color(&theme.color_scheme)
        }),
        disabled_content_color: disabled_content_color.unwrap_or_else(|| {
            FloatingActionButtonDefaults::disabled_content_color(&theme.color_scheme)
        }),
        interaction_state,
        ripple_color,
        accessibility_label,
        accessibility_description,
        content,
    };
    let content = args.content;
    let shape = args
        .shape
        .unwrap_or_else(|| shape_from_size(args.size, &theme.shapes));
    let typography = theme.typography;
    let scheme = theme.color_scheme;
    let inherited_content_color = use_context::<ContentColor>()
        .map(|c| c.get().current)
        .unwrap_or_else(|| ContentColor::default().current);
    let size = FloatingActionButtonDefaults::container_size(args.size);

    let container_color = if args.enabled {
        args.container_color
    } else {
        args.disabled_container_color
    };

    let content_color = if args.enabled {
        args.content_color.unwrap_or_else(|| {
            content_color_for(args.container_color, &scheme).unwrap_or(inherited_content_color)
        })
    } else {
        args.disabled_content_color
    };

    let ripple_color = args.ripple_color.unwrap_or(content_color);
    let elevation = args.elevation;
    let interactive = args.enabled && args.on_click.is_some();
    let interaction_state = if interactive {
        Some(
            args.interaction_state
                .unwrap_or_else(|| remember(InteractionState::new)),
        )
    } else {
        args.interaction_state
    };

    let target_elevation = elevation.shadow_elevation(args.enabled, interaction_state);
    let frame = current_frame_nanos();
    let elevation_motion = remember(|| ElevationMotion::new(target_elevation, frame));
    if elevation_motion.with(|motion| motion.motion.target()) != target_elevation.0 as f32 {
        elevation_motion.with_mut(|motion| {
            motion.advance(frame);
            motion.motion.set_target(target_elevation.0 as f32);
        });
    }
    if !elevation_motion.with(|motion| motion.motion.is_idle()) {
        receive_frame_nanos(move |frame| {
            elevation_motion.with_mut(|motion| motion.advance(frame));
            if elevation_motion.with(|motion| motion.motion.is_idle()) {
                FrameNanosControl::Stop
            } else {
                FrameNanosControl::Continue
            }
        });
    }
    let shadow_elevation = Dp(elevation_motion.with(|motion| motion.motion.value()) as f64);
    let tonal_elevation = elevation.tonal_elevation();

    surface()
        .modifier(args.modifier.default_min_size(size, size))
        .style(SurfaceStyle::Filled {
            color: container_color,
        })
        .shape(shape)
        .content_alignment(Alignment::Center)
        .content_color(content_color)
        .enabled(args.enabled)
        .ripple_color(ripple_color)
        .elevation(shadow_elevation)
        .tonal_elevation(tonal_elevation)
        .accessibility_role(Role::Button)
        .accessibility_focusable(args.enabled)
        .interaction_state_optional(interaction_state)
        .on_click_optional(args.on_click)
        .accessibility_label_optional(args.accessibility_label)
        .accessibility_description_optional(args.accessibility_description)
        .child({
            move || {
                provide_context(
                    || typography.label_large,
                    move || {
                        if let Some(content) = content.as_ref() {
                            content.render();
                        }
                    },
                );
            }
        });
}

impl FloatingActionButtonBuilder {
    /// Creates a configuration with the required click handler.
    pub fn new(on_click: impl Fn() + Send + Sync + 'static) -> Self {
        Self::default().on_click(on_click)
    }

    /// Applies the small floating action button preset.
    pub fn small(self, on_click: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_click(on_click)
            .size(FloatingActionButtonSize::Small)
    }

    /// Applies the large floating action button preset.
    pub fn large(self, on_click: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_click(on_click)
            .size(FloatingActionButtonSize::Large)
    }

    /// Applies the medium floating action button preset.
    pub fn medium(self, on_click: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_click(on_click)
            .size(FloatingActionButtonSize::Medium)
    }
}

fn shape_from_size(size: FloatingActionButtonSize, shapes: &MaterialShapes) -> Shape {
    match size {
        FloatingActionButtonSize::Small => shapes.medium,
        FloatingActionButtonSize::Standard => shapes.large,
        FloatingActionButtonSize::Medium => shapes.large_increased,
        FloatingActionButtonSize::Large => shapes.extra_large,
    }
}

#[cfg(test)]
mod tests {
    use tessera_ui::{Px, PxSize};

    use crate::shape_def::ResolvedShape;

    use super::*;

    #[test]
    fn medium_fab_uses_an_80dp_container_28dp_icon_and_20dp_corners() {
        assert_eq!(
            FloatingActionButtonDefaults::container_size(FloatingActionButtonSize::Medium),
            Dp(80.0)
        );
        assert_eq!(
            FloatingActionButtonDefaults::icon_size(FloatingActionButtonSize::Medium),
            Dp(28.0)
        );
        let ResolvedShape::Rounded {
            corner_radii,
            corner_g2,
        } = FloatingActionButtonDefaults::shape(FloatingActionButtonSize::Medium)
            .resolve_for_size(PxSize::new(Px::new(80), Px::new(80)))
        else {
            panic!("expected rounded rectangle");
        };
        assert_eq!(corner_radii, [20.0; 4]);
        assert_eq!(corner_g2, [2.0; 4]);
    }

    #[test]
    fn elevation_effect_retargets_continuously_and_settles_without_overshoot() {
        let mut motion = ElevationMotion::new(Dp(6.0), 0);
        motion.motion.set_target(8.0);
        motion.advance(16_666_667);
        let initial = motion.motion.value();
        assert!(initial > 6.0 && initial < 8.0);
        motion.motion.set_target(6.0);
        assert_eq!(motion.motion.value(), initial);
        for frame in 2..120 {
            motion.advance(frame * 16_666_667);
            assert!(motion.motion.value() >= 6.0 && motion.motion.value() <= 8.0);
        }
        assert!(motion.motion.is_idle());
        assert_eq!(motion.motion.value(), 6.0);
    }
}
