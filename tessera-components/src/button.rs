//! Material-styled button components.
//!
//! ## Usage
//!
//! Trigger actions, submit forms, or navigate.
use tessera_ui::{
    Callback, Color, Dp, Modifier, RenderSlot, accesskit::Role, layout::layout, tessera,
    use_context,
};

use crate::{
    alignment::Alignment,
    modifier::ModifierExt,
    shape_def::Shape,
    surface::{SurfaceStyle, surface},
    theme::{
        ContentColor, MaterialAlpha, MaterialColorScheme, MaterialTheme, content_color_for,
        provide_text_style,
    },
};

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
    let button_args = ButtonResolvedArgs {
        enabled: enabled.unwrap_or(true),
        modifier: modifier.unwrap_or_default(),
        color: color.unwrap_or(scheme.primary),
        content_color,
        shape: shape.unwrap_or(Shape::CAPSULE),
        padding: padding.unwrap_or(ButtonDefaults::CONTENT_VERTICAL_PADDING),
        on_click,
        ripple_color: ripple_color.unwrap_or(scheme.on_primary),
        border_width: border_width.unwrap_or(Dp(0.0)),
        border_color,
        elevation,
        tonal_elevation: tonal_elevation.unwrap_or(Dp(0.0)),
        disabled_container_color: disabled_container_color
            .unwrap_or_else(|| ButtonDefaults::disabled_container_color(&scheme)),
        disabled_content_color: disabled_content_color
            .unwrap_or_else(|| ButtonDefaults::disabled_content_color(&scheme)),
        disabled_border_color: disabled_border_color
            .unwrap_or_else(|| ButtonDefaults::disabled_border_color(&scheme)),
        accessibility_label,
        accessibility_description,
        child,
    };
    let child = button_args.child;
    let typography = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .typography;

    create_surface_builder(&button_args).child(move || {
        let child = child;
        let modifier = Modifier::new().padding_all(button_args.padding);
        layout().modifier(modifier).child(move || {
            if let Some(child) = child.as_ref() {
                let child = *child;
                provide_text_style(typography.label_large, move || child.render());
            }
        });
    });
}

fn create_surface_builder(args: &ButtonResolvedArgs) -> crate::surface::SurfaceBuilder {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let inherited_content_color = use_context::<ContentColor>()
        .map(|c| c.get().current)
        .unwrap_or(ContentColor::default().current);

    let container_color = if args.enabled {
        args.color
    } else {
        args.disabled_container_color
    };

    let content_color = if args.enabled {
        args.content_color.unwrap_or_else(|| {
            content_color_for(args.color, &scheme).unwrap_or(inherited_content_color)
        })
    } else {
        args.disabled_content_color
    };

    let style = if args.border_width.to_pixels_f32() > 0.0 {
        let border_color = if args.enabled {
            args.border_color.unwrap_or(container_color)
        } else {
            args.disabled_border_color
        };
        SurfaceStyle::FilledOutlined {
            fill_color: container_color,
            border_color,
            border_width: args.border_width,
        }
    } else {
        SurfaceStyle::Filled {
            color: container_color,
        }
    };

    let mut builder = surface();

    if let Some(elevation) = args.elevation {
        builder = builder.elevation(elevation);
    }

    if args.enabled
        && let Some(on_click) = args.on_click
    {
        builder = builder.on_click_shared(on_click);
    }

    if let Some(label) = args.accessibility_label.clone() {
        builder = builder.accessibility_label(label);
    }

    if let Some(description) = args.accessibility_description.clone() {
        builder = builder.accessibility_description(description);
    }

    builder
        .style(style)
        .shape(args.shape)
        .modifier(args.modifier.clone().size_in(
            Some(ButtonDefaults::MIN_WIDTH),
            None,
            Some(ButtonDefaults::MIN_HEIGHT),
            None,
        ))
        .ripple_color(args.ripple_color)
        .content_alignment(Alignment::Center)
        .content_color(content_color)
        .enabled(args.enabled)
        .tonal_elevation(args.tonal_elevation)
        .accessibility_role(Role::Button)
        .accessibility_focusable(true)
}

impl ButtonBuilder {
    /// Applies the standard "Filled" button preset.
    /// Create a standard "Filled" button (High emphasis).
    /// Uses Primary color for container and OnPrimary for content.
    pub fn filled(self) -> Self {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        self.color(scheme.primary)
            .content_color(scheme.on_primary)
            .ripple_color(scheme.on_primary)
            .disabled_container_color(
                scheme
                    .on_surface
                    .with_alpha(ButtonDefaults::FILLED_DISABLED_CONTAINER_ALPHA),
            )
            .disabled_content_color(
                scheme
                    .on_surface_variant
                    .with_alpha(ButtonDefaults::DISABLED_LABEL_ALPHA),
            )
    }

    /// Applies the "Elevated" button preset.
    /// Create an "Elevated" button (Medium emphasis).
    /// Uses Surface color (or SurfaceContainerLow if available) with a shadow.
    pub fn elevated(self) -> Self {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        self.color(scheme.surface_container_low)
            .content_color(scheme.primary)
            .ripple_color(scheme.primary)
            .elevation(Dp(1.0))
            .disabled_container_color(
                scheme
                    .on_surface
                    .with_alpha(ButtonDefaults::FILLED_DISABLED_CONTAINER_ALPHA),
            )
            .disabled_content_color(
                scheme
                    .on_surface_variant
                    .with_alpha(ButtonDefaults::DISABLED_LABEL_ALPHA),
            )
    }

    /// Applies the "Tonal" button preset.
    /// Create a "Tonal" button (Medium emphasis).
    /// Uses SecondaryContainer color for container and OnSecondaryContainer for
    /// content.
    pub fn tonal(self) -> Self {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        self.color(scheme.secondary_container)
            .content_color(scheme.on_secondary_container)
            .ripple_color(scheme.on_secondary_container)
            .disabled_container_color(
                scheme
                    .on_surface
                    .with_alpha(ButtonDefaults::DISABLED_CONTAINER_ALPHA),
            )
            .disabled_content_color(
                scheme
                    .on_surface
                    .with_alpha(ButtonDefaults::DISABLED_CONTENT_ALPHA),
            )
    }

    /// Applies the "Outlined" button preset.
    /// Create an "Outlined" button (Medium emphasis).
    /// Transparent container with an Outline border.
    pub fn outlined(self) -> Self {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        self.color(Color::TRANSPARENT)
            .content_color(scheme.on_surface_variant)
            .disabled_container_color(Color::TRANSPARENT)
            .ripple_color(scheme.on_surface_variant)
            .border_width(Dp(1.0))
            .border_color(scheme.outline_variant)
            .disabled_border_color(
                scheme
                    .outline_variant
                    .with_alpha(ButtonDefaults::FILLED_DISABLED_CONTAINER_ALPHA),
            )
            .disabled_content_color(
                scheme
                    .on_surface_variant
                    .with_alpha(ButtonDefaults::DISABLED_LABEL_ALPHA),
            )
    }

    /// Applies the "Text" button preset.
    /// Create a "Text" button (Low emphasis).
    /// Transparent container and no border.
    pub fn text(self) -> Self {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        self.color(Color::TRANSPARENT)
            .content_color(scheme.primary)
            .disabled_container_color(Color::TRANSPARENT)
            .ripple_color(scheme.primary)
            .disabled_content_color(
                scheme
                    .on_surface_variant
                    .with_alpha(ButtonDefaults::DISABLED_LABEL_ALPHA),
            )
    }
}
