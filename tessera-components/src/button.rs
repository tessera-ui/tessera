//! Material-styled button components.
//!
//! ## Usage
//!
//! Trigger actions, submit forms, or navigate.
use tessera_ui::{
    Callback, Color, Dp, Modifier, RenderSlot, accesskit::Role, context::provide_context,
    layout::layout, tessera, use_context,
};

use crate::{
    alignment::Alignment,
    modifier::ModifierExt,
    shape_def::Shape,
    surface::{SurfaceStyle, surface},
    theme::{ContentColor, MaterialAlpha, MaterialColorScheme, MaterialTheme, content_color_for},
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
    pub const CONTENT_VERTICAL_PADDING: Dp = Dp(8.0);

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

#[derive(Clone)]
struct ButtonResolvedArgs {
    enabled: bool,
    modifier: Modifier,
    color: Color,
    content_color: Option<Color>,
    shape: Shape,
    padding: Dp,
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
/// - `enabled` — optional enabled flag.
/// - `modifier` — modifier chain applied to the button subtree.
/// - `color` — optional container color override.
/// - `content_color` — optional content color override.
/// - `shape` — optional shape override.
/// - `padding` — optional internal padding.
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
/// # #[tessera]
/// # fn component() {
/// use tessera_components::{button::button, text::text};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// button().filled().on_click(|| {}).child(|| {
///     text().content("Click Me");
/// });
/// #     });
/// # }
/// # component();
/// ```
/// Renders a Material button.
#[tessera]
pub fn button(
    variant: Option<ButtonVariant>,
    enabled: Option<bool>,
    modifier: Option<Modifier>,
    color: Option<Color>,
    content_color: Option<Color>,
    shape: Option<Shape>,
    padding: Option<Dp>,
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
            scheme.primary,
            scheme.primary,
            Dp(1.0),
            Some(scheme.outline),
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
    let container_color = color.unwrap_or(default_container_color);
    let content_color = content_color.unwrap_or_else(|| {
        content_color_for(container_color, &scheme).unwrap_or(default_content_color)
    });
    let button_args = ButtonResolvedArgs {
        enabled: enabled.unwrap_or(true),
        modifier: modifier.unwrap_or_default(),
        color: container_color,
        content_color: Some(content_color),
        shape: shape.unwrap_or(Shape::CAPSULE),
        padding: padding.unwrap_or(ButtonDefaults::CONTENT_VERTICAL_PADDING),
        on_click,
        ripple_color: ripple_color.unwrap_or(default_ripple_color),
        border_width: border_width.unwrap_or(default_border_width),
        border_color: border_color.or(default_border_color),
        elevation: elevation.or(default_elevation),
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
    let typography = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .typography;
    let child = RenderSlot::new(move || {
        let child = child;
        let modifier = Modifier::new().padding_all(padding);
        layout().modifier(modifier).child(move || {
            if let Some(child) = child.as_ref() {
                let child = *child;
                provide_context(|| typography.label_large, move || child.render());
            }
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
    let on_click = button_args
        .enabled
        .then_some(button_args.on_click)
        .flatten();

    surface()
        .style(style)
        .shape(button_args.shape)
        .modifier(button_args.modifier.clone().size_in(
            Some(ButtonDefaults::MIN_WIDTH),
            None,
            Some(ButtonDefaults::MIN_HEIGHT),
            None,
        ))
        .ripple_color(button_args.ripple_color)
        .content_alignment(Alignment::Center)
        .content_color(content_color)
        .enabled(button_args.enabled)
        .tonal_elevation(button_args.tonal_elevation)
        .accessibility_role(Role::Button)
        .accessibility_focusable(true)
        .elevation_optional(button_args.elevation)
        .on_click_optional(on_click)
        .accessibility_label_optional(button_args.accessibility_label)
        .accessibility_description_optional(button_args.accessibility_description)
        .child_shared(child);
}

impl ButtonBuilder {
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
