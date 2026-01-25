//! Material Design split buttons for primary and secondary actions.
//!
//! ## Usage
//!
//! Pair a primary action with a related secondary action or menu.

use std::sync::Arc;

use derive_setters::Setters;

use tessera_ui::{
    Color, ComputedData, Constraint, DimensionValue, Dp, LayoutInput, LayoutOutput, LayoutSpec,
    MeasurementError, Modifier, Px, PxPosition, accesskit::Role, tessera, use_context,
};

use crate::{
    alignment::Alignment,
    button::ButtonDefaults,
    modifier::{ModifierExt as _, Padding},
    shape_def::{RoundedCorner, Shape},
    surface::{SurfaceArgs, SurfaceStyle, surface},
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
#[derive(Clone, Copy, Debug)]
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

/// Arguments for [`split_button_layout`].
#[derive(Clone, Setters)]
pub struct SplitButtonLayoutArgs {
    /// Modifier chain applied to the split button layout.
    pub modifier: Modifier,
    /// Spacing between leading and trailing buttons.
    pub spacing: Dp,
}

impl Default for SplitButtonLayoutArgs {
    fn default() -> Self {
        Self {
            modifier: Modifier::new()
                .constrain(Some(DimensionValue::WRAP), Some(DimensionValue::WRAP)),
            spacing: SplitButtonDefaults::SPACING,
        }
    }
}

/// Arguments for [`split_leading_button`].
#[derive(Clone, Setters)]
pub struct SplitButtonLeadingArgs {
    /// Whether the button is enabled.
    pub enabled: bool,
    /// Modifier chain applied to the leading button.
    pub modifier: Modifier,
    /// Shape of the leading button.
    pub shape: Shape,
    /// Colors used to render the leading button.
    pub colors: SplitButtonColors,
    /// Border width for the leading button.
    pub border_width: Dp,
    /// Inner padding for the leading button content.
    pub content_padding: Padding,
    /// Minimum width for the leading button.
    pub min_width: Dp,
    /// Container height for the leading button.
    pub container_height: Dp,
    /// Optional shadow elevation.
    #[setters(strip_option)]
    pub elevation: Option<Dp>,
    /// Tonal elevation applied to the surface.
    pub tonal_elevation: Dp,
    /// Click handler for the button.
    #[setters(skip)]
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Optional accessibility label.
    #[setters(strip_option, into)]
    pub accessibility_label: Option<String>,
    /// Optional accessibility description.
    #[setters(strip_option, into)]
    pub accessibility_description: Option<String>,
}

impl SplitButtonLeadingArgs {
    fn with_variant(variant: SplitButtonVariant) -> Self {
        let size = SplitButtonSize::default();
        Self {
            enabled: true,
            modifier: Modifier::new(),
            shape: SplitButtonDefaults::leading_shape(size),
            colors: SplitButtonDefaults::colors(variant),
            border_width: SplitButtonDefaults::border_width(variant),
            content_padding: SplitButtonDefaults::leading_content_padding(size),
            min_width: SplitButtonDefaults::MIN_WIDTH,
            container_height: SplitButtonDefaults::container_height(size),
            elevation: SplitButtonDefaults::elevation(variant),
            tonal_elevation: SplitButtonDefaults::tonal_elevation(variant),
            on_click: None,
            accessibility_label: None,
            accessibility_description: None,
        }
    }

    /// Create a filled leading split button configuration.
    pub fn filled(on_click: impl Fn() + Send + Sync + 'static) -> Self {
        Self::with_variant(SplitButtonVariant::Filled).on_click(on_click)
    }

    /// Create a tonal leading split button configuration.
    pub fn tonal(on_click: impl Fn() + Send + Sync + 'static) -> Self {
        Self::with_variant(SplitButtonVariant::Tonal).on_click(on_click)
    }

    /// Create an elevated leading split button configuration.
    pub fn elevated(on_click: impl Fn() + Send + Sync + 'static) -> Self {
        Self::with_variant(SplitButtonVariant::Elevated).on_click(on_click)
    }

    /// Create an outlined leading split button configuration.
    pub fn outlined(on_click: impl Fn() + Send + Sync + 'static) -> Self {
        Self::with_variant(SplitButtonVariant::Outlined).on_click(on_click)
    }

    /// Update the visual variant for the leading split button.
    pub fn variant(mut self, variant: SplitButtonVariant) -> Self {
        self.colors = SplitButtonDefaults::colors(variant);
        self.border_width = SplitButtonDefaults::border_width(variant);
        self.elevation = SplitButtonDefaults::elevation(variant);
        self.tonal_elevation = SplitButtonDefaults::tonal_elevation(variant);
        self
    }

    /// Update the size-related defaults for the leading split button.
    pub fn size(mut self, size: SplitButtonSize) -> Self {
        self.shape = SplitButtonDefaults::leading_shape(size);
        self.content_padding = SplitButtonDefaults::leading_content_padding(size);
        self.container_height = SplitButtonDefaults::container_height(size);
        self
    }

    /// Set the click handler.
    pub fn on_click(mut self, on_click: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_click = Some(Arc::new(on_click));
        self
    }

    /// Set the click handler using a shared callback.
    pub fn on_click_shared(mut self, on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.on_click = Some(on_click);
        self
    }
}

impl Default for SplitButtonLeadingArgs {
    fn default() -> Self {
        Self::with_variant(SplitButtonVariant::Filled)
    }
}

/// Arguments for [`split_trailing_button`].
#[derive(Clone, Setters)]
pub struct SplitButtonTrailingArgs {
    /// Whether the button is enabled.
    pub enabled: bool,
    /// Modifier chain applied to the trailing button.
    pub modifier: Modifier,
    /// Shape of the trailing button.
    pub shape: Shape,
    /// Colors used to render the trailing button.
    pub colors: SplitButtonColors,
    /// Border width for the trailing button.
    pub border_width: Dp,
    /// Inner padding for the trailing button content.
    pub content_padding: Padding,
    /// Minimum width for the trailing button.
    pub min_width: Dp,
    /// Container height for the trailing button.
    pub container_height: Dp,
    /// Optional shadow elevation.
    #[setters(strip_option)]
    pub elevation: Option<Dp>,
    /// Tonal elevation applied to the surface.
    pub tonal_elevation: Dp,
    /// Click handler for the button.
    #[setters(skip)]
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Optional accessibility label.
    #[setters(strip_option, into)]
    pub accessibility_label: Option<String>,
    /// Optional accessibility description.
    #[setters(strip_option, into)]
    pub accessibility_description: Option<String>,
}

impl SplitButtonTrailingArgs {
    fn with_variant(variant: SplitButtonVariant) -> Self {
        let size = SplitButtonSize::default();
        Self {
            enabled: true,
            modifier: Modifier::new(),
            shape: SplitButtonDefaults::trailing_shape(size),
            colors: SplitButtonDefaults::colors(variant),
            border_width: SplitButtonDefaults::border_width(variant),
            content_padding: SplitButtonDefaults::trailing_content_padding(size),
            min_width: SplitButtonDefaults::MIN_WIDTH,
            container_height: SplitButtonDefaults::container_height(size),
            elevation: SplitButtonDefaults::elevation(variant),
            tonal_elevation: SplitButtonDefaults::tonal_elevation(variant),
            on_click: None,
            accessibility_label: None,
            accessibility_description: None,
        }
    }

    /// Create a filled trailing split button configuration.
    pub fn filled(on_click: impl Fn() + Send + Sync + 'static) -> Self {
        Self::with_variant(SplitButtonVariant::Filled).on_click(on_click)
    }

    /// Create a tonal trailing split button configuration.
    pub fn tonal(on_click: impl Fn() + Send + Sync + 'static) -> Self {
        Self::with_variant(SplitButtonVariant::Tonal).on_click(on_click)
    }

    /// Create an elevated trailing split button configuration.
    pub fn elevated(on_click: impl Fn() + Send + Sync + 'static) -> Self {
        Self::with_variant(SplitButtonVariant::Elevated).on_click(on_click)
    }

    /// Create an outlined trailing split button configuration.
    pub fn outlined(on_click: impl Fn() + Send + Sync + 'static) -> Self {
        Self::with_variant(SplitButtonVariant::Outlined).on_click(on_click)
    }

    /// Update the visual variant for the trailing split button.
    pub fn variant(mut self, variant: SplitButtonVariant) -> Self {
        self.colors = SplitButtonDefaults::colors(variant);
        self.border_width = SplitButtonDefaults::border_width(variant);
        self.elevation = SplitButtonDefaults::elevation(variant);
        self.tonal_elevation = SplitButtonDefaults::tonal_elevation(variant);
        self
    }

    /// Update the size-related defaults for the trailing split button.
    pub fn size(mut self, size: SplitButtonSize) -> Self {
        self.shape = SplitButtonDefaults::trailing_shape(size);
        self.content_padding = SplitButtonDefaults::trailing_content_padding(size);
        self.container_height = SplitButtonDefaults::container_height(size);
        self
    }

    /// Set the click handler.
    pub fn on_click(mut self, on_click: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_click = Some(Arc::new(on_click));
        self
    }

    /// Set the click handler using a shared callback.
    pub fn on_click_shared(mut self, on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.on_click = Some(on_click);
        self
    }
}

impl Default for SplitButtonTrailingArgs {
    fn default() -> Self {
        Self::with_variant(SplitButtonVariant::Filled)
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
/// - `args` — configures spacing and modifier; see [`SplitButtonLayoutArgs`].
/// - `leading_button` — renders the primary button content.
/// - `trailing_button` — renders the secondary button content.
///
/// ## Examples
///
/// ```
/// # use std::sync::Arc;
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::{
///     split_buttons::{
///         SplitButtonLayoutArgs, SplitButtonLeadingArgs, SplitButtonTrailingArgs,
///         SplitButtonVariant, split_button_layout, split_leading_button, split_trailing_button,
///     },
///     text::text,
///     theme::{MaterialTheme, material_theme},
/// };
/// use tessera_ui::remember;
///
/// material_theme(
///     || MaterialTheme::default(),
///     || {
///         let primary_clicked = remember(|| false);
///         let secondary_clicked = remember(|| false);
///         let primary = Arc::new(move || primary_clicked.set(true));
///         let secondary = Arc::new(move || secondary_clicked.set(true));
///         let primary_for_layout = primary.clone();
///         let secondary_for_layout = secondary.clone();
///
///         split_button_layout(
///             SplitButtonLayoutArgs::default(),
///             move || {
///                 split_leading_button(
///                     SplitButtonLeadingArgs::default()
///                         .variant(SplitButtonVariant::Filled)
///                         .on_click_shared(primary_for_layout.clone()),
///                     || text("Create"),
///                 );
///             },
///             move || {
///                 split_trailing_button(
///                     SplitButtonTrailingArgs::default()
///                         .variant(SplitButtonVariant::Filled)
///                         .on_click_shared(secondary_for_layout.clone()),
///                     || text("More"),
///                 );
///             },
///         );
///
///         primary();
///         secondary();
///         assert!(primary_clicked.get());
///         assert!(secondary_clicked.get());
///     },
/// );
/// # }
/// # component();
/// ```
#[tessera]
pub fn split_button_layout(
    args: impl Into<SplitButtonLayoutArgs>,
    leading_button: impl FnOnce() + Send + Sync + 'static,
    trailing_button: impl FnOnce() + Send + Sync + 'static,
) {
    let args: SplitButtonLayoutArgs = args.into();
    let modifier = args.modifier;
    modifier.run(move || split_button_layout_inner(args, leading_button, trailing_button));
}

#[tessera]
fn split_button_layout_inner(
    args: SplitButtonLayoutArgs,
    leading_button: impl FnOnce() + Send + Sync + 'static,
    trailing_button: impl FnOnce() + Send + Sync + 'static,
) {
    let spacing = Px::from(args.spacing).max(Px::ZERO);
    layout(SplitButtonLayoutSpec { spacing });
    leading_button();
    trailing_button();
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
/// - `args` — configures appearance, sizing, and interaction; see
///   [`SplitButtonLeadingArgs`].
/// - `content` — renders the leading button content.
///
/// ## Examples
///
/// ```
/// # use std::sync::Arc;
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::{
///     split_buttons::{SplitButtonLeadingArgs, SplitButtonVariant, split_leading_button},
///     text::text,
///     theme::{MaterialTheme, material_theme},
/// };
/// use tessera_ui::remember;
///
/// material_theme(
///     || MaterialTheme::default(),
///     || {
///         let invoked = remember(|| false);
///         let action = Arc::new(move || invoked.set(true));
///
///         split_leading_button(
///             SplitButtonLeadingArgs::default()
///                 .variant(SplitButtonVariant::Filled)
///                 .on_click_shared(action.clone()),
///             || text("Create"),
///         );
///
///         action();
///         assert!(invoked.get());
///     },
/// );
/// # }
/// # component();
/// ```
#[tessera]
pub fn split_leading_button(
    args: impl Into<SplitButtonLeadingArgs>,
    content: impl FnOnce() + Send + Sync + 'static,
) {
    render_split_button(args.into().into(), content);
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
/// - `args` — configures appearance, sizing, and interaction; see
///   [`SplitButtonTrailingArgs`].
/// - `content` — renders the trailing button content.
///
/// ## Examples
///
/// ```
/// # use std::sync::Arc;
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::{
///     split_buttons::{SplitButtonTrailingArgs, SplitButtonVariant, split_trailing_button},
///     text::text,
///     theme::{MaterialTheme, material_theme},
/// };
/// use tessera_ui::remember;
///
/// material_theme(
///     || MaterialTheme::default(),
///     || {
///         let invoked = remember(|| false);
///         let action = Arc::new(move || invoked.set(true));
///
///         split_trailing_button(
///             SplitButtonTrailingArgs::default()
///                 .variant(SplitButtonVariant::Filled)
///                 .on_click_shared(action.clone()),
///             || text("More"),
///         );
///
///         action();
///         assert!(invoked.get());
///     },
/// );
/// # }
/// # component();
/// ```
#[tessera]
pub fn split_trailing_button(
    args: impl Into<SplitButtonTrailingArgs>,
    content: impl FnOnce() + Send + Sync + 'static,
) {
    render_split_button(args.into().into(), content);
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
    on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    accessibility_label: Option<String>,
    accessibility_description: Option<String>,
}

impl From<SplitButtonLeadingArgs> for SplitButtonItemArgs {
    fn from(args: SplitButtonLeadingArgs) -> Self {
        Self {
            enabled: args.enabled,
            modifier: args.modifier,
            shape: args.shape,
            colors: args.colors,
            border_width: args.border_width,
            content_padding: args.content_padding,
            min_width: args.min_width,
            container_height: args.container_height,
            elevation: args.elevation,
            tonal_elevation: args.tonal_elevation,
            on_click: args.on_click,
            accessibility_label: args.accessibility_label,
            accessibility_description: args.accessibility_description,
        }
    }
}

impl From<SplitButtonTrailingArgs> for SplitButtonItemArgs {
    fn from(args: SplitButtonTrailingArgs) -> Self {
        Self {
            enabled: args.enabled,
            modifier: args.modifier,
            shape: args.shape,
            colors: args.colors,
            border_width: args.border_width,
            content_padding: args.content_padding,
            min_width: args.min_width,
            container_height: args.container_height,
            elevation: args.elevation,
            tonal_elevation: args.tonal_elevation,
            on_click: args.on_click,
            accessibility_label: args.accessibility_label,
            accessibility_description: args.accessibility_description,
        }
    }
}

#[derive(Clone, PartialEq)]
struct SplitButtonLayoutSpec {
    spacing: Px,
}

impl LayoutSpec for SplitButtonLayoutSpec {
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

        let layout_constraint = Constraint::new(
            input.parent_constraint().width(),
            input.parent_constraint().height(),
        );
        let child_constraint = Constraint::new(
            DimensionValue::Wrap {
                min: None,
                max: layout_constraint.width.get_max(),
            },
            DimensionValue::Wrap {
                min: None,
                max: layout_constraint.height.get_max(),
            },
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
            .get_max()
            .map(|max| (max - leading_size.width - self.spacing).max(Px::ZERO));
        let trailing_constraint = Constraint::new(
            DimensionValue::Wrap {
                min: None,
                max: trailing_max_width,
            },
            DimensionValue::Fixed(leading_size.height),
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

fn resolve_dimension(constraint: DimensionValue, content: Px) -> Px {
    match constraint {
        DimensionValue::Fixed(value) => value,
        DimensionValue::Fill { min, max } => {
            if let Some(max) = max {
                let mut value = max;
                if let Some(min) = min {
                    value = value.max(min);
                }
                value
            } else {
                panic!(
                    "Fill size without max constraint is not supported in split button layouts."
                );
            }
        }
        DimensionValue::Wrap { min, max } => {
            let mut value = content;
            if let Some(min) = min {
                value = value.max(min);
            }
            if let Some(max) = max {
                value = value.min(max);
            }
            value
        }
    }
}

fn center_offset(child: Px, container: Px) -> Px {
    if container.0 > child.0 {
        (container - child) / 2
    } else {
        Px::ZERO
    }
}

fn render_split_button(args: SplitButtonItemArgs, content: impl FnOnce() + Send + Sync + 'static) {
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

    let mut surface_args = SurfaceArgs::default()
        .modifier(modifier.size_in(Some(min_width), None, Some(container_height), None))
        .style(style)
        .shape(shape)
        .content_alignment(Alignment::Center)
        .content_color(content_color)
        .enabled(enabled)
        .ripple_color(content_color)
        .tonal_elevation(tonal_elevation);

    if let Some(elevation) = elevation {
        surface_args = surface_args.elevation(elevation);
    }

    if let Some(on_click) = on_click {
        surface_args = surface_args
            .on_click_shared(on_click)
            .accessibility_role(Role::Button)
            .accessibility_focusable(true);
    }

    if let Some(label) = accessibility_label {
        surface_args = surface_args.accessibility_label(label);
    }
    if let Some(description) = accessibility_description {
        surface_args = surface_args.accessibility_description(description);
    }

    surface(surface_args, move || {
        provide_text_style(typography.label_large, move || {
            Modifier::new().padding(content_padding).run(content);
        });
    });
}
