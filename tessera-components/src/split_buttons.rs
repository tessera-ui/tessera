//! Material Design split buttons for primary and secondary actions.
//!
//! ## Usage
//!
//! Pair a primary action with a related secondary action or menu.

use tessera_ui::{
    AxisConstraint, Callback, Color, ComputedData, Constraint, Dp, LayoutInput, LayoutOutput,
    LayoutPolicy, MeasurementError, Modifier, Px, PxPosition, RenderSlot, accesskit::Role,
    layout::layout, tessera, use_context,
};

use crate::{
    alignment::Alignment,
    button::ButtonDefaults,
    modifier::{ModifierExt as _, Padding},
    shape_def::{RoundedCorner, Shape},
    surface::{SurfaceStyle, surface},
    theme::{MaterialTheme, provide_text_style},
};

const CORNER_G2_VALUE: f32 = 3.0;

/// Sizes supported by split buttons.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SplitButtonSize {
    /// Extra small split buttons (32dp height).
    ExtraSmall,
    /// Small split buttons (40dp height).
    #[default]
    Small,
    /// Medium split buttons (56dp height).
    Medium,
    /// Large split buttons (96dp height).
    Large,
    /// Extra large split buttons (136dp height).
    ExtraLarge,
}

/// Visual emphasis variants for split buttons.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SplitButtonVariant {
    /// High-emphasis filled variant.
    #[default]
    Filled,
    /// Medium-emphasis tonal variant.
    Tonal,
    /// Elevated variant with shadow.
    Elevated,
    /// Medium-emphasis outlined variant.
    Outlined,
}

/// Color values for split buttons in different states.
#[derive(Clone, PartialEq, Copy, Debug)]
pub struct SplitButtonColors {
    /// Container color when enabled.
    pub container_color: Color,
    /// Content color when enabled.
    pub content_color: Color,
    /// Border color when enabled.
    pub border_color: Color,
    /// Container color when disabled.
    pub disabled_container_color: Color,
    /// Content color when disabled.
    pub disabled_content_color: Color,
    /// Border color when disabled.
    pub disabled_border_color: Color,
}

impl SplitButtonColors {
    fn container_color(self, enabled: bool) -> Color {
        if enabled {
            self.container_color
        } else {
            self.disabled_container_color
        }
    }

    fn content_color(self, enabled: bool) -> Color {
        if enabled {
            self.content_color
        } else {
            self.disabled_content_color
        }
    }

    fn border_color(self, enabled: bool) -> Color {
        if enabled {
            self.border_color
        } else {
            self.disabled_border_color
        }
    }
}

/// Defaults for split buttons.
pub struct SplitButtonDefaults;

impl SplitButtonDefaults {
    /// Minimum width of split button items.
    pub const MIN_WIDTH: Dp = Dp(48.0);
    /// Default spacing between leading and trailing buttons.
    pub const SPACING: Dp = Dp(2.0);
    /// Default leading icon size.
    pub const LEADING_ICON_SIZE: Dp = Dp(20.0);

    /// Returns the container height for the provided size.
    pub fn container_height(size: SplitButtonSize) -> Dp {
        match size {
            SplitButtonSize::ExtraSmall => Dp(32.0),
            SplitButtonSize::Small => Dp(40.0),
            SplitButtonSize::Medium => Dp(56.0),
            SplitButtonSize::Large => Dp(96.0),
            SplitButtonSize::ExtraLarge => Dp(136.0),
        }
    }

    /// Returns the inner corner size for the provided size.
    pub fn inner_corner_size(size: SplitButtonSize) -> Dp {
        match size {
            SplitButtonSize::ExtraSmall => Dp(4.0),
            SplitButtonSize::Small => Dp(4.0),
            SplitButtonSize::Medium => Dp(4.0),
            SplitButtonSize::Large => Dp(8.0),
            SplitButtonSize::ExtraLarge => Dp(12.0),
        }
    }

    /// Returns the default leading button padding for the provided size.
    pub fn leading_content_padding(size: SplitButtonSize) -> Padding {
        let (start, end) = match size {
            SplitButtonSize::ExtraSmall => (Dp(12.0), Dp(10.0)),
            SplitButtonSize::Small => (Dp(16.0), Dp(12.0)),
            SplitButtonSize::Medium => (Dp(24.0), Dp(24.0)),
            SplitButtonSize::Large => (Dp(48.0), Dp(48.0)),
            SplitButtonSize::ExtraLarge => (Dp(64.0), Dp(64.0)),
        };
        Padding::new(start, Dp(0.0), end, Dp(0.0))
    }

    /// Returns the default trailing button padding for the provided size.
    pub fn trailing_content_padding(size: SplitButtonSize) -> Padding {
        let (start, end) = match size {
            SplitButtonSize::ExtraSmall => (Dp(13.0), Dp(13.0)),
            SplitButtonSize::Small => (Dp(13.0), Dp(13.0)),
            SplitButtonSize::Medium => (Dp(15.0), Dp(15.0)),
            SplitButtonSize::Large => (Dp(29.0), Dp(29.0)),
            SplitButtonSize::ExtraLarge => (Dp(43.0), Dp(43.0)),
        };
        Padding::new(start, Dp(0.0), end, Dp(0.0))
    }

    /// Returns the trailing icon size for the provided size.
    pub fn trailing_icon_size(size: SplitButtonSize) -> Dp {
        match size {
            SplitButtonSize::ExtraSmall => Dp(22.0),
            SplitButtonSize::Small => Dp(22.0),
            SplitButtonSize::Medium => Dp(26.0),
            SplitButtonSize::Large => Dp(38.0),
            SplitButtonSize::ExtraLarge => Dp(50.0),
        }
    }

    /// Returns the default leading button shape for the provided size.
    pub fn leading_shape(size: SplitButtonSize) -> Shape {
        let inner = RoundedCorner::manual(Self::inner_corner_size(size), CORNER_G2_VALUE);
        Shape::RoundedRectangle {
            top_left: RoundedCorner::Capsule,
            top_right: inner,
            bottom_right: inner,
            bottom_left: RoundedCorner::Capsule,
        }
    }

    /// Returns the default trailing button shape for the provided size.
    pub fn trailing_shape(size: SplitButtonSize) -> Shape {
        let inner = RoundedCorner::manual(Self::inner_corner_size(size), CORNER_G2_VALUE);
        Shape::RoundedRectangle {
            top_left: inner,
            top_right: RoundedCorner::Capsule,
            bottom_right: RoundedCorner::Capsule,
            bottom_left: inner,
        }
    }

    /// Returns default colors for the requested variant.
    pub fn colors(variant: SplitButtonVariant) -> SplitButtonColors {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;

        match variant {
            SplitButtonVariant::Filled => SplitButtonColors {
                container_color: scheme.primary,
                content_color: scheme.on_primary,
                border_color: scheme.primary,
                disabled_container_color: scheme
                    .on_surface
                    .with_alpha(ButtonDefaults::FILLED_DISABLED_CONTAINER_ALPHA),
                disabled_content_color: scheme
                    .on_surface_variant
                    .with_alpha(ButtonDefaults::DISABLED_LABEL_ALPHA),
                disabled_border_color: ButtonDefaults::disabled_border_color(&scheme),
            },
            SplitButtonVariant::Tonal => SplitButtonColors {
                container_color: scheme.secondary_container,
                content_color: scheme.on_secondary_container,
                border_color: scheme.secondary_container,
                disabled_container_color: scheme
                    .on_surface
                    .with_alpha(ButtonDefaults::DISABLED_CONTAINER_ALPHA),
                disabled_content_color: scheme
                    .on_surface
                    .with_alpha(ButtonDefaults::DISABLED_CONTENT_ALPHA),
                disabled_border_color: ButtonDefaults::disabled_border_color(&scheme),
            },
            SplitButtonVariant::Elevated => SplitButtonColors {
                container_color: scheme.surface_container_low,
                content_color: scheme.primary,
                border_color: scheme.surface_container_low,
                disabled_container_color: scheme
                    .on_surface
                    .with_alpha(ButtonDefaults::FILLED_DISABLED_CONTAINER_ALPHA),
                disabled_content_color: scheme
                    .on_surface_variant
                    .with_alpha(ButtonDefaults::DISABLED_LABEL_ALPHA),
                disabled_border_color: ButtonDefaults::disabled_border_color(&scheme),
            },
            SplitButtonVariant::Outlined => SplitButtonColors {
                container_color: Color::TRANSPARENT,
                content_color: scheme.on_surface_variant,
                border_color: scheme.outline_variant,
                disabled_container_color: Color::TRANSPARENT,
                disabled_content_color: scheme
                    .on_surface_variant
                    .with_alpha(ButtonDefaults::DISABLED_LABEL_ALPHA),
                disabled_border_color: scheme
                    .outline_variant
                    .with_alpha(ButtonDefaults::FILLED_DISABLED_CONTAINER_ALPHA),
            },
        }
    }

    /// Returns the default border width for a split button variant.
    pub fn border_width(variant: SplitButtonVariant) -> Dp {
        match variant {
            SplitButtonVariant::Outlined => Dp(1.0),
            _ => Dp(0.0),
        }
    }

    /// Returns the default elevation for a split button variant.
    pub fn elevation(variant: SplitButtonVariant) -> Option<Dp> {
        match variant {
            SplitButtonVariant::Elevated => Some(Dp(1.0)),
            _ => None,
        }
    }

    /// Returns the tonal elevation for split button variants.
    pub fn tonal_elevation(_variant: SplitButtonVariant) -> Dp {
        Dp(0.0)
    }
}

#[allow(missing_docs)]
impl SplitLeadingButtonBuilder {
    pub fn filled(self) -> Self {
        self.variant(SplitButtonVariant::Filled).enabled(true)
    }

    pub fn tonal(self) -> Self {
        self.variant(SplitButtonVariant::Tonal).enabled(true)
    }

    pub fn elevated(self) -> Self {
        self.variant(SplitButtonVariant::Elevated).enabled(true)
    }

    pub fn outlined(self) -> Self {
        self.variant(SplitButtonVariant::Outlined).enabled(true)
    }
}

#[allow(missing_docs)]
impl SplitTrailingButtonBuilder {
    pub fn filled(self) -> Self {
        self.variant(SplitButtonVariant::Filled).enabled(true)
    }

    pub fn tonal(self) -> Self {
        self.variant(SplitButtonVariant::Tonal).enabled(true)
    }

    pub fn elevated(self) -> Self {
        self.variant(SplitButtonVariant::Elevated).enabled(true)
    }

    pub fn outlined(self) -> Self {
        self.variant(SplitButtonVariant::Outlined).enabled(true)
    }
}

/// # split_button_layout
///
/// Lay out a split button with leading and trailing actions.
///
/// ## Usage
///
/// Group a primary action with a related secondary action or menu.
///
/// ## Parameters
///
/// - `modifier` — optional modifier chain applied to the split button layout.
/// - `spacing` — optional spacing between the leading and trailing buttons.
/// - `leading_button` — optional leading button slot.
/// - `trailing_button` — optional trailing button slot.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::{
///     split_buttons::{split_button_layout, split_leading_button, split_trailing_button},
///     text::text,
///     theme::{MaterialTheme, material_theme},
/// };
///
/// material_theme()
///     .theme(|| MaterialTheme::default())
///     .child(|| {
///         split_button_layout()
///             .leading_button(|| {
///                 split_leading_button().filled().on_click(|| {}).content(|| {
///                     text().content("Create");
///                 });
///             })
///             .trailing_button(|| {
///                 split_trailing_button()
///                     .filled()
///                     .on_click(|| {})
///                     .content(|| {
///                         text().content("More");
///                     });
///             });
///     });
/// # }
/// # component();
/// ```
#[tessera]
pub fn split_button_layout(
    modifier: Option<Modifier>,
    spacing: Option<Dp>,
    leading_button: Option<RenderSlot>,
    trailing_button: Option<RenderSlot>,
) {
    let modifier = modifier.unwrap_or_default();
    let leading_button = leading_button.unwrap_or_else(RenderSlot::empty);
    let trailing_button = trailing_button.unwrap_or_else(RenderSlot::empty);
    let spacing = Px::from(spacing.unwrap_or(SplitButtonDefaults::SPACING)).max(Px::ZERO);
    layout()
        .modifier(modifier)
        .layout_policy(SplitButtonLayoutPolicy { spacing })
        .child(move || {
            leading_button.render();
            trailing_button.render();
        });
}

/// # split_leading_button
///
/// Render the leading button for a split button pair.
///
/// ## Usage
///
/// Use as the primary action within a split button.
///
/// ## Parameters
///
/// - `variant` — visual emphasis variant.
/// - `size` — split button size preset.
/// - `enabled` — optional enabled state; defaults to `true`.
/// - `modifier` — optional modifier chain applied to the leading button.
/// - `on_click` — optional click callback.
/// - `accessibility_label` — optional accessibility label.
/// - `accessibility_description` — optional accessibility description.
/// - `content` — optional slot rendered inside the leading button.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::{
///     split_buttons::split_leading_button,
///     text::text,
///     theme::{MaterialTheme, material_theme},
/// };
///
/// material_theme()
///     .theme(|| MaterialTheme::default())
///     .child(|| {
///         split_leading_button().filled().on_click(|| {}).content(|| {
///             text().content("Create");
///         });
///     });
/// # }
/// # component();
/// ```
#[tessera]
pub fn split_leading_button(
    variant: SplitButtonVariant,
    size: SplitButtonSize,
    enabled: Option<bool>,
    modifier: Option<Modifier>,
    on_click: Option<Callback>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    content: Option<RenderSlot>,
) {
    let content = content.unwrap_or_else(RenderSlot::empty);
    render_split_button(
        SplitButtonItemArgs::leading(
            variant,
            size,
            enabled.unwrap_or(true),
            modifier.unwrap_or_default(),
            on_click,
            accessibility_label,
            accessibility_description,
        ),
        content,
    );
}

/// # split_trailing_button
///
/// Render the trailing button for a split button pair.
///
/// ## Usage
///
/// Use as a secondary action or menu affordance within a split button.
///
/// ## Parameters
///
/// - `variant` — visual emphasis variant.
/// - `size` — split button size preset.
/// - `enabled` — optional enabled state; defaults to `true`.
/// - `modifier` — optional modifier chain applied to the trailing button.
/// - `on_click` — optional click callback.
/// - `accessibility_label` — optional accessibility label.
/// - `accessibility_description` — optional accessibility description.
/// - `content` — optional slot rendered inside the trailing button.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::{
///     split_buttons::split_trailing_button,
///     text::text,
///     theme::{MaterialTheme, material_theme},
/// };
///
/// material_theme()
///     .theme(|| MaterialTheme::default())
///     .child(|| {
///         split_trailing_button()
///             .filled()
///             .on_click(|| {})
///             .content(|| {
///                 text().content("More");
///             });
///     });
/// # }
/// # component();
/// ```
#[tessera]
pub fn split_trailing_button(
    variant: SplitButtonVariant,
    size: SplitButtonSize,
    enabled: Option<bool>,
    modifier: Option<Modifier>,
    on_click: Option<Callback>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    content: Option<RenderSlot>,
) {
    let content = content.unwrap_or_else(RenderSlot::empty);
    render_split_button(
        SplitButtonItemArgs::trailing(
            variant,
            size,
            enabled.unwrap_or(true),
            modifier.unwrap_or_default(),
            on_click,
            accessibility_label,
            accessibility_description,
        ),
        content,
    );
}
#[derive(Clone)]
struct SplitButtonItemArgs {
    enabled: bool,
    modifier: Modifier,
    shape: Shape,
    colors: SplitButtonColors,
    border_width: Dp,
    content_padding: Padding,
    min_width: Dp,
    container_height: Dp,
    elevation: Option<Dp>,
    tonal_elevation: Dp,
    on_click: Option<Callback>,
    accessibility_label: Option<String>,
    accessibility_description: Option<String>,
}

impl SplitButtonItemArgs {
    fn leading(
        variant: SplitButtonVariant,
        size: SplitButtonSize,
        enabled: bool,
        modifier: Modifier,
        on_click: Option<Callback>,
        accessibility_label: Option<String>,
        accessibility_description: Option<String>,
    ) -> Self {
        Self {
            enabled,
            modifier,
            shape: SplitButtonDefaults::leading_shape(size),
            colors: SplitButtonDefaults::colors(variant),
            border_width: SplitButtonDefaults::border_width(variant),
            content_padding: SplitButtonDefaults::leading_content_padding(size),
            min_width: SplitButtonDefaults::MIN_WIDTH,
            container_height: SplitButtonDefaults::container_height(size),
            elevation: SplitButtonDefaults::elevation(variant),
            tonal_elevation: SplitButtonDefaults::tonal_elevation(variant),
            on_click,
            accessibility_label,
            accessibility_description,
        }
    }

    fn trailing(
        variant: SplitButtonVariant,
        size: SplitButtonSize,
        enabled: bool,
        modifier: Modifier,
        on_click: Option<Callback>,
        accessibility_label: Option<String>,
        accessibility_description: Option<String>,
    ) -> Self {
        Self {
            enabled,
            modifier,
            shape: SplitButtonDefaults::trailing_shape(size),
            colors: SplitButtonDefaults::colors(variant),
            border_width: SplitButtonDefaults::border_width(variant),
            content_padding: SplitButtonDefaults::trailing_content_padding(size),
            min_width: SplitButtonDefaults::MIN_WIDTH,
            container_height: SplitButtonDefaults::container_height(size),
            elevation: SplitButtonDefaults::elevation(variant),
            tonal_elevation: SplitButtonDefaults::tonal_elevation(variant),
            on_click,
            accessibility_label,
            accessibility_description,
        }
    }
}

#[derive(Clone, PartialEq)]
struct SplitButtonLayoutPolicy {
    spacing: Px,
}

impl LayoutPolicy for SplitButtonLayoutPolicy {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let child_ids = input.children_ids();
        if child_ids.len() != 2 {
            return Err(MeasurementError::MeasureFnFailed(
                "SplitButtonLayout requires exactly two children.".to_string(),
            ));
        }

        let layout_constraint = *input.parent_constraint().as_ref();
        let child_constraint = Constraint::new(
            AxisConstraint::new(Px::ZERO, layout_constraint.width.resolve_max()),
            AxisConstraint::new(Px::ZERO, layout_constraint.height.resolve_max()),
        );

        let leading_id = child_ids[0];
        let leading_size = input
            .measure_children(vec![(leading_id, child_constraint)])?
            .get(&leading_id)
            .copied()
            .unwrap_or(ComputedData::ZERO);

        let trailing_id = child_ids[1];
        let trailing_max_width = layout_constraint
            .width
            .resolve_max()
            .map(|max| (max - leading_size.width - self.spacing).max(Px::ZERO));
        let trailing_constraint = Constraint::new(
            AxisConstraint::new(Px::ZERO, trailing_max_width),
            leading_size.height,
        );
        let trailing_size = input
            .measure_children(vec![(trailing_id, trailing_constraint)])?
            .get(&trailing_id)
            .copied()
            .unwrap_or(ComputedData::ZERO);

        let content_width = leading_size.width + trailing_size.width + self.spacing;
        let content_height = leading_size.height.max(trailing_size.height);
        let final_width = resolve_dimension(layout_constraint.width, content_width);
        let final_height = resolve_dimension(layout_constraint.height, content_height);

        output.place_child(
            leading_id,
            PxPosition::new(Px::ZERO, center_offset(leading_size.height, final_height)),
        );
        output.place_child(
            trailing_id,
            PxPosition::new(
                leading_size.width + self.spacing,
                center_offset(trailing_size.height, final_height),
            ),
        );

        Ok(ComputedData {
            width: final_width,
            height: final_height,
        })
    }
}

fn resolve_dimension(constraint: AxisConstraint, content: Px) -> Px {
    constraint.clamp(content)
}

fn center_offset(child: Px, container: Px) -> Px {
    if container.0 > child.0 {
        (container - child) / 2
    } else {
        Px::ZERO
    }
}

struct SplitButtonSurfaceArgs {
    modifier: Modifier,
    style: SurfaceStyle,
    shape: Shape,
    content_color: Color,
    enabled: bool,
    tonal_elevation: Dp,
    elevation: Option<Dp>,
    on_click: Option<Callback>,
    accessibility_label: Option<String>,
    accessibility_description: Option<String>,
}

fn split_button_surface(args: SplitButtonSurfaceArgs) -> crate::surface::SurfaceBuilder {
    let builder = surface()
        .modifier(args.modifier)
        .style(args.style)
        .shape(args.shape)
        .content_alignment(Alignment::Center)
        .content_color(args.content_color)
        .enabled(args.enabled)
        .ripple_color(args.content_color)
        .tonal_elevation(args.tonal_elevation);

    let builder = if let Some(elevation) = args.elevation {
        builder.elevation(elevation)
    } else {
        builder
    };

    let builder = if let Some(on_click) = args.on_click {
        builder
            .on_click_shared(on_click)
            .accessibility_role(Role::Button)
            .accessibility_focusable(true)
    } else {
        builder
    };

    let builder = if let Some(label) = args.accessibility_label {
        builder.accessibility_label(label)
    } else {
        builder
    };

    if let Some(description) = args.accessibility_description {
        builder.accessibility_description(description)
    } else {
        builder
    }
}

fn render_split_button(args: SplitButtonItemArgs, content: RenderSlot) {
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let typography = theme.typography;
    let SplitButtonItemArgs {
        enabled,
        modifier,
        shape,
        colors,
        border_width,
        content_padding,
        min_width,
        container_height,
        elevation,
        tonal_elevation,
        on_click,
        accessibility_label,
        accessibility_description,
    } = args;

    let container_color = colors.container_color(enabled);
    let content_color = colors.content_color(enabled);
    let border_color = colors.border_color(enabled);
    let style = if border_width.to_pixels_f32() > 0.0 {
        if container_color == Color::TRANSPARENT {
            SurfaceStyle::Outlined {
                color: border_color,
                width: border_width,
            }
        } else {
            SurfaceStyle::FilledOutlined {
                fill_color: container_color,
                border_color,
                border_width,
            }
        }
    } else {
        SurfaceStyle::Filled {
            color: container_color,
        }
    };

    split_button_surface(SplitButtonSurfaceArgs {
        modifier: modifier.size_in(Some(min_width), None, Some(container_height), None),
        style,
        shape,
        content_color,
        enabled,
        tonal_elevation,
        elevation,
        on_click,
        accessibility_label,
        accessibility_description,
    })
    .with_child(move || {
        let content = content;
        provide_text_style(typography.label_large, move || {
            layout()
                .modifier(Modifier::new().padding(content_padding))
                .child(move || {
                    content.render();
                });
        });
    });
}
