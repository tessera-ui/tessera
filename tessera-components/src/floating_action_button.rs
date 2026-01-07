//! Floating action button component.
//!
//! ## Usage
//!
//! Emphasize a primary action with a prominent floating button.
use std::sync::Arc;

use derive_setters::Setters;
use tessera_ui::{Color, Dp, Modifier, State, accesskit::Role, remember, tessera, use_context};

use crate::{
    alignment::Alignment,
    modifier::{InteractionState, ModifierExt as _},
    shape_def::Shape,
    surface::{SurfaceArgs, SurfaceStyle, surface},
    theme::{
        ContentColor, MaterialAlpha, MaterialColorScheme, MaterialTheme, content_color_for,
        provide_text_style,
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
    /// Large floating action button (96dp).
    Large,
}

impl FloatingActionButtonSize {
    fn container_size(self) -> Dp {
        match self {
            FloatingActionButtonSize::Small => FloatingActionButtonDefaults::SMALL_SIZE,
            FloatingActionButtonSize::Standard => FloatingActionButtonDefaults::STANDARD_SIZE,
            FloatingActionButtonSize::Large => FloatingActionButtonDefaults::LARGE_SIZE,
        }
    }

    fn icon_size(self) -> Dp {
        match self {
            FloatingActionButtonSize::Small => FloatingActionButtonDefaults::SMALL_ICON_SIZE,
            FloatingActionButtonSize::Standard => FloatingActionButtonDefaults::STANDARD_ICON_SIZE,
            FloatingActionButtonSize::Large => FloatingActionButtonDefaults::LARGE_ICON_SIZE,
        }
    }
}

/// Elevation values used by floating action buttons across interaction states.
#[derive(Clone, Copy, Debug)]
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

/// Material Design defaults for [`floating_action_button`].
pub struct FloatingActionButtonDefaults;

impl FloatingActionButtonDefaults {
    /// Default container size for a standard floating action button.
    pub const STANDARD_SIZE: Dp = Dp(56.0);
    /// Container size for a small floating action button.
    pub const SMALL_SIZE: Dp = Dp(40.0);
    /// Container size for a large floating action button.
    pub const LARGE_SIZE: Dp = Dp(96.0);
    /// Recommended icon size for a standard floating action button.
    pub const STANDARD_ICON_SIZE: Dp = Dp(24.0);
    /// Recommended icon size for a small floating action button.
    pub const SMALL_ICON_SIZE: Dp = Dp(24.0);
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
        let theme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get();
        shape_from_size(size, &theme)
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

/// Arguments for configuring [`floating_action_button`].
#[derive(Clone, Setters)]
pub struct FloatingActionButtonArgs {
    /// The size variant of the floating action button.
    pub size: FloatingActionButtonSize,
    /// Optional modifier chain applied to the button.
    pub modifier: Modifier,
    /// Whether the button is enabled for interaction.
    pub enabled: bool,
    /// Container color when enabled.
    pub container_color: Color,
    /// Optional explicit content color override.
    #[setters(strip_option)]
    pub content_color: Option<Color>,
    /// Optional shape override for the button container.
    #[setters(strip_option)]
    pub shape: Option<Shape>,
    /// Elevation profile used for interaction states.
    pub elevation: FloatingActionButtonElevation,
    /// Optional click handler for the button.
    #[setters(skip)]
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Container color when disabled.
    pub disabled_container_color: Color,
    /// Content color when disabled.
    pub disabled_content_color: Color,
    /// Optional shared interaction state for hover/press feedback.
    #[setters(strip_option)]
    pub interaction_state: Option<State<InteractionState>>,
    /// Optional ripple color override.
    #[setters(strip_option)]
    pub ripple_color: Option<Color>,
    /// Optional accessibility label announced by assistive technologies.
    #[setters(strip_option, into)]
    pub accessibility_label: Option<String>,
    /// Optional accessibility description announced by assistive technologies.
    #[setters(strip_option, into)]
    pub accessibility_description: Option<String>,
}

impl FloatingActionButtonArgs {
    /// Creates a configuration with the required click handler.
    pub fn new(on_click: impl Fn() + Send + Sync + 'static) -> Self {
        Self::default().on_click(on_click)
    }

    /// Creates a small floating action button configuration.
    pub fn small(on_click: impl Fn() + Send + Sync + 'static) -> Self {
        Self::new(on_click).size(FloatingActionButtonSize::Small)
    }

    /// Creates a large floating action button configuration.
    pub fn large(on_click: impl Fn() + Send + Sync + 'static) -> Self {
        Self::new(on_click).size(FloatingActionButtonSize::Large)
    }

    /// Sets the click handler.
    pub fn on_click<F>(mut self, on_click: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Arc::new(on_click));
        self
    }

    /// Sets the click handler using a shared callback.
    pub fn on_click_shared(mut self, on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.on_click = Some(on_click);
        self
    }
}

impl Default for FloatingActionButtonArgs {
    fn default() -> Self {
        let theme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get();
        let scheme = theme.color_scheme;
        Self {
            size: FloatingActionButtonSize::default(),
            modifier: Modifier::new(),
            enabled: true,
            container_color: FloatingActionButtonDefaults::container_color(&scheme),
            content_color: None,
            shape: None,
            elevation: FloatingActionButtonDefaults::elevation(),
            on_click: None,
            disabled_container_color: FloatingActionButtonDefaults::disabled_container_color(
                &scheme,
            ),
            disabled_content_color: FloatingActionButtonDefaults::disabled_content_color(&scheme),
            interaction_state: None,
            ripple_color: None,
            accessibility_label: None,
            accessibility_description: None,
        }
    }
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
/// - `args` - configures size, colors, elevation, and click behavior; see
///   [`FloatingActionButtonArgs`].
/// - `content` - closure that renders the FAB content, typically an icon.
///
/// ## Examples
///
/// ```
/// use std::sync::Arc;
/// use tessera_components::floating_action_button::{
///     FloatingActionButtonArgs, floating_action_button,
/// };
/// use tessera_ui::{remember, tessera};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// #[tessera]
/// fn component() {
/// #     material_theme(
/// #         || MaterialTheme::default(),
/// #         || {
///     let clicked = remember(|| false);
///     let on_click: Arc<dyn Fn() + Send + Sync> = {
///         let clicked = clicked;
///         Arc::new(move || clicked.with_mut(|value| *value = true))
///     };
///     let args = FloatingActionButtonArgs::default().on_click_shared(on_click.clone());
///     on_click();
///
///     assert!(clicked.with(|value| *value));
///
///     floating_action_button(args, || {});
/// #         },
/// #     );
/// }
///
/// component();
/// ```
#[tessera]
pub fn floating_action_button(
    args: impl Into<FloatingActionButtonArgs>,
    content: impl FnOnce() + Send + Sync + 'static,
) {
    let args: FloatingActionButtonArgs = args.into();
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let shape = args
        .shape
        .unwrap_or_else(|| shape_from_size(args.size, &theme));
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

    let shadow_elevation = elevation.shadow_elevation(args.enabled, interaction_state);
    let tonal_elevation = elevation.tonal_elevation();

    let mut surface_args = SurfaceArgs::default()
        .modifier(args.modifier.size_in(Some(size), None, Some(size), None))
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
        .accessibility_focusable(true);

    if let Some(state) = interaction_state {
        surface_args = surface_args.interaction_state(state);
    }

    if let Some(on_click) = args.on_click {
        surface_args = surface_args.on_click_shared(on_click);
    }

    if let Some(label) = args.accessibility_label {
        surface_args = surface_args.accessibility_label(label);
    }

    if let Some(description) = args.accessibility_description {
        surface_args = surface_args.accessibility_description(description);
    }

    surface(surface_args, move || {
        provide_text_style(typography.label_large, move || {
            content();
        });
    });
}

fn shape_from_size(size: FloatingActionButtonSize, theme: &MaterialTheme) -> Shape {
    match size {
        FloatingActionButtonSize::Small => theme.shapes.medium,
        FloatingActionButtonSize::Standard => theme.shapes.large,
        FloatingActionButtonSize::Large => theme.shapes.extra_large,
    }
}
