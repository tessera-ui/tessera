//! Material Design split buttons for primary and secondary actions.
//!
//! ## Usage
//!
//! Pair a primary action with a related secondary action or menu.

use tessera_ui::{
    AxisConstraint, Callback, Color, ComputedData, Constraint, Dp, LayoutResult, MeasurementError,
    Modifier, Px, PxPosition, RenderSlot, State,
    layout::{MeasureScope, layout},
    tessera, use_context,
};

use crate::{
    button::ButtonDefaults,
    modifier::{InteractionState, Padding},
    shape_def::{RoundedCorner, Shape},
    theme::{MaterialColorScheme, MaterialTheme, TextStyle},
};

use item::split_button_item;

mod item;

const CORNER_G2_VALUE: f32 = 2.0;

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

impl SplitButtonSize {
    fn text_style(self, theme: &MaterialTheme) -> TextStyle {
        match self {
            Self::ExtraSmall | Self::Small => theme.typography.label_large,
            Self::Medium => theme.typography.title_medium,
            Self::Large => theme.typography.headline_small,
            Self::ExtraLarge => theme.typography.headline_large,
        }
    }
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

    /// Returns the leading icon size for the selected container height.
    pub const fn leading_icon_size(size: SplitButtonSize) -> Dp {
        Dp(match size {
            SplitButtonSize::ExtraSmall | SplitButtonSize::Small => 20.0,
            SplitButtonSize::Medium => 24.0,
            SplitButtonSize::Large => 32.0,
            SplitButtonSize::ExtraLarge => 40.0,
        })
    }

    /// Returns the recommended space between a leading icon and its label.
    pub const fn icon_spacing(size: SplitButtonSize) -> Dp {
        Dp(match size {
            SplitButtonSize::ExtraSmall | SplitButtonSize::Small | SplitButtonSize::Medium => 8.0,
            SplitButtonSize::Large => 12.0,
            SplitButtonSize::ExtraLarge => 16.0,
        })
    }

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

    /// Returns the inner corner radius for hover, focus, and press feedback.
    pub const fn active_inner_corner_size(size: SplitButtonSize) -> Dp {
        Dp(match size {
            SplitButtonSize::ExtraSmall | SplitButtonSize::Small | SplitButtonSize::Medium => 12.0,
            SplitButtonSize::Large | SplitButtonSize::ExtraLarge => 20.0,
        })
    }

    /// Returns the menu icon's resting optical offset from the geometric
    /// center.
    pub const fn trailing_optical_offset(size: SplitButtonSize) -> Dp {
        Dp(match size {
            SplitButtonSize::ExtraSmall | SplitButtonSize::Small => -1.0,
            SplitButtonSize::Medium => -2.0,
            SplitButtonSize::Large => -3.0,
            SplitButtonSize::ExtraLarge => -6.0,
        })
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

    /// Returns the minimum trailing width including both content insets.
    pub const fn trailing_min_width(size: SplitButtonSize) -> Dp {
        Dp(match size {
            SplitButtonSize::ExtraSmall | SplitButtonSize::Small => 48.0,
            SplitButtonSize::Medium => 56.0,
            SplitButtonSize::Large => 96.0,
            SplitButtonSize::ExtraLarge => 136.0,
        })
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
    pub fn colors(scheme: &MaterialColorScheme, variant: SplitButtonVariant) -> SplitButtonColors {
        let scheme = scheme.clone();

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
                    .with_alpha(ButtonDefaults::FILLED_DISABLED_CONTAINER_ALPHA),
                disabled_content_color: scheme
                    .on_surface_variant
                    .with_alpha(ButtonDefaults::DISABLED_LABEL_ALPHA),
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

    /// Returns the outline thickness for a variant and size.
    pub const fn border_width_for_size(variant: SplitButtonVariant, size: SplitButtonSize) -> Dp {
        if !matches!(variant, SplitButtonVariant::Outlined) {
            return Dp(0.0);
        }
        Dp(match size {
            SplitButtonSize::ExtraSmall | SplitButtonSize::Small | SplitButtonSize::Medium => 1.0,
            SplitButtonSize::Large => 2.0,
            SplitButtonSize::ExtraLarge => 3.0,
        })
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
        self.variant(SplitButtonVariant::Filled)
    }

    pub fn tonal(self) -> Self {
        self.variant(SplitButtonVariant::Tonal)
    }

    pub fn elevated(self) -> Self {
        self.variant(SplitButtonVariant::Elevated)
    }

    pub fn outlined(self) -> Self {
        self.variant(SplitButtonVariant::Outlined)
    }
}

#[allow(missing_docs)]
impl SplitTrailingButtonBuilder {
    pub fn filled(self) -> Self {
        self.variant(SplitButtonVariant::Filled)
    }

    pub fn tonal(self) -> Self {
        self.variant(SplitButtonVariant::Tonal)
    }

    pub fn elevated(self) -> Self {
        self.variant(SplitButtonVariant::Elevated)
    }

    pub fn outlined(self) -> Self {
        self.variant(SplitButtonVariant::Outlined)
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
/// use tessera_components::{
///     icon::icon,
///     material_icons::filled,
///     menus::MenuController,
///     split_buttons::{split_button_layout, split_leading_button, split_trailing_button},
///     text::text,
///     theme::material_theme,
/// };
/// use tessera_ui::{remember, tessera};
///
/// let mut menu = MenuController::new();
/// menu.open();
/// assert!(menu.is_open());
/// menu.close();
/// assert!(!menu.is_open());
///
/// #[tessera]
/// fn component() {
///     let menu = remember(MenuController::new);
///     material_theme().child(move || {
///         split_button_layout()
///             .leading_button(|| {
///                 split_leading_button().on_click(|| {}).content(|| {
///                     text().content("Send");
///                 });
///             })
///             .trailing_button(move || {
///                 split_trailing_button()
///                     .expanded(menu.with(MenuController::is_open))
///                     .accessibility_label("Send options")
///                     .on_click(move || menu.with_mut(MenuController::toggle))
///                     .content(|| {
///                         icon().painter(filled::ARROW_DROP_DOWN_SVG);
///                     });
///             });
///     });
/// }
/// component();
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
        .policy(SplitButton { spacing })
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
/// - `interaction_state` — optional shared pointer and focus state.
/// - `shape` — fixed shape override that suppresses automatic corner changes.
/// - `content` — leading icon and label, sized and placed in a row.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     split_buttons::split_leading_button, text::text, theme::material_theme,
/// };
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn component() {
///     let activations = remember(|| 0_u32);
///     material_theme().child(move || {
///         split_leading_button()
///             .accessibility_label("Send")
///             .on_click(move || activations.set(activations.get() + 1))
///             .content(|| {
///                 text().content("Send");
///             });
///     });
///     assert_eq!(activations.get(), 0);
/// }
/// component();
/// ```
#[tessera]
pub fn split_leading_button(
    variant: Option<SplitButtonVariant>,
    size: Option<SplitButtonSize>,
    enabled: Option<bool>,
    modifier: Option<Modifier>,
    on_click: Option<Callback>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    interaction_state: Option<State<InteractionState>>,
    shape: Option<Shape>,
    content: Option<RenderSlot>,
) {
    let size = size.unwrap_or_default();
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let args = SplitButtonItemArgs::leading(
        &scheme,
        variant.unwrap_or_default(),
        size,
        enabled.unwrap_or(true),
        modifier.unwrap_or_default(),
        on_click,
        accessibility_label,
        accessibility_description,
    );
    split_button_item()
        .args(args)
        .size(size)
        .leading(true)
        .interaction_state_optional(interaction_state)
        .shape_optional(shape)
        .content_optional(content);
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
/// - `interaction_state` — optional shared pointer and focus state.
/// - `shape` — fixed shape override that suppresses automatic corner changes.
/// - `expanded` — controlled menu disclosure state with popup semantics.
/// - `content` — menu icon, sized automatically for the preset.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     icon::icon, material_icons::filled, menus::MenuController,
///     split_buttons::split_trailing_button, theme::material_theme,
/// };
/// use tessera_ui::{remember, tessera};
///
/// let mut menu = MenuController::new();
/// menu.toggle();
/// assert!(menu.is_open());
/// menu.toggle();
/// assert!(!menu.is_open());
///
/// #[tessera]
/// fn component() {
///     let menu = remember(MenuController::new);
///     material_theme().child(move || {
///         split_trailing_button()
///             .expanded(menu.with(MenuController::is_open))
///             .accessibility_label("Send options")
///             .on_click(move || menu.with_mut(MenuController::toggle))
///             .content(|| {
///                 icon().painter(filled::ARROW_DROP_DOWN_SVG);
///             });
///     });
/// }
/// component();
/// ```
#[tessera]
pub fn split_trailing_button(
    variant: Option<SplitButtonVariant>,
    size: Option<SplitButtonSize>,
    enabled: Option<bool>,
    modifier: Option<Modifier>,
    on_click: Option<Callback>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    interaction_state: Option<State<InteractionState>>,
    shape: Option<Shape>,
    expanded: Option<bool>,
    content: Option<RenderSlot>,
) {
    let size = size.unwrap_or_default();
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let args = SplitButtonItemArgs::trailing(
        &scheme,
        variant.unwrap_or_default(),
        size,
        enabled.unwrap_or(true),
        modifier.unwrap_or_default(),
        on_click,
        accessibility_label,
        accessibility_description,
    );
    split_button_item()
        .args(args)
        .size(size)
        .leading(false)
        .interaction_state_optional(interaction_state)
        .shape_optional(shape)
        .expanded_optional(expanded)
        .content_optional(content);
}

#[derive(Clone, PartialEq)]
struct SplitButtonItemArgs {
    enabled: bool,
    modifier: Modifier,
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
        scheme: &MaterialColorScheme,
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
            colors: SplitButtonDefaults::colors(scheme, variant),
            border_width: SplitButtonDefaults::border_width_for_size(variant, size),
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
        scheme: &MaterialColorScheme,
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
            colors: SplitButtonDefaults::colors(scheme, variant),
            border_width: SplitButtonDefaults::border_width_for_size(variant, size),
            content_padding: SplitButtonDefaults::trailing_content_padding(size),
            min_width: SplitButtonDefaults::trailing_min_width(size),
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
struct SplitButton {
    spacing: Px,
}

impl tessera_ui::LayoutModifierNode for SplitButton {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        if children.len() != 2 {
            return Err(MeasurementError::MeasureFnFailed(
                "SplitButtonLayout requires exactly two children.".to_string(),
            ));
        }

        let layout_constraint = *input.parent_constraint().as_ref();
        let child_constraint = Constraint::new(
            AxisConstraint::new(Px::ZERO, layout_constraint.width.resolve_max()),
            AxisConstraint::new(Px::ZERO, layout_constraint.height.resolve_max()),
        );

        let trailing = children[1];
        let trailing_size = trailing.probe(input, &child_constraint)?.size();
        let spacing = layout_constraint
            .width
            .resolve_max()
            .map_or(self.spacing, |max| {
                self.spacing.min((max - trailing_size.width).max(Px::ZERO))
            });
        let leading_max_width = layout_constraint
            .width
            .resolve_max()
            .map(|max| (max - trailing_size.width - spacing).max(Px::ZERO));
        let leading_constraint = Constraint::new(
            AxisConstraint::new(Px::ZERO, leading_max_width),
            child_constraint.height,
        );
        let leading = children[0];
        let leading_visual = leading.probe_intrinsic(input, &leading_constraint)?.size();
        let trailing_visual = trailing.probe_intrinsic(input, &child_constraint)?.size();
        let visual_height = child_constraint
            .height
            .clamp(leading_visual.height.max(trailing_visual.height));
        let leading_constraint = Constraint::new(
            leading_constraint.width,
            AxisConstraint::new(visual_height, child_constraint.height.max),
        );
        let trailing_constraint =
            Constraint::new(child_constraint.width, leading_constraint.height);
        let leading_size = leading.measure(input, &leading_constraint)?.size();
        let trailing_size = trailing.measure(input, &trailing_constraint)?.size();
        let content_width = leading_size.width + trailing_size.width + spacing;
        let content_height = leading_size.height.max(trailing_size.height);
        let final_width = resolve_dimension(layout_constraint.width, content_width);
        let final_height = resolve_dimension(layout_constraint.height, content_height);

        result.place_child(
            leading,
            PxPosition::new(Px::ZERO, center_offset(leading_size.height, final_height)),
        );
        result.place_child(
            trailing,
            PxPosition::new(
                leading_size.width + spacing,
                center_offset(trailing_size.height, final_height),
            ),
        );

        Ok(result.with_size(ComputedData {
            width: final_width,
            height: final_height,
        }))
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

#[cfg(test)]
mod expressive_tests;

#[cfg(test)]
mod height_tests;

#[cfg(test)]
mod tests {
    use tessera_ui::{Modifier, tessera};

    use crate::{
        modifier::{ModifierExt as _, SemanticsArgs},
        split_buttons::{
            SplitButtonSize, split_button_layout, split_leading_button, split_trailing_button,
        },
        text::text,
        theme::{MaterialTheme, material_theme},
    };

    #[tessera]
    fn split_button_case() {
        material_theme()
            .theme(|| MaterialTheme::default())
            .child(|| {
                split_button_layout()
                    .leading_button(|| {
                        split_leading_button()
                            .filled()
                            .size(SplitButtonSize::Small)
                            .modifier(Modifier::new().semantics(SemanticsArgs {
                                test_tag: Some("split-leading".to_string()),
                                ..Default::default()
                            }))
                            .content(|| {
                                text().content("Send");
                            });
                    })
                    .trailing_button(|| {
                        split_trailing_button()
                            .filled()
                            .size(SplitButtonSize::ExtraLarge)
                            .modifier(Modifier::new().semantics(SemanticsArgs {
                                test_tag: Some("split-trailing".to_string()),
                                ..Default::default()
                            }))
                            .content(|| {
                                text().content("More");
                            });
                    });
            });
    }

    #[test]
    fn layout_preserves_the_trailing_segment_under_mixed_sizes() {
        tessera_ui::dp::set_scale_factor(1.0);
        tessera_ui::assert_layout! {
            viewport: (400, 200),
            content: {
                split_button_case();
            },
            expect: {
                node("split-leading").height(136);
                node("split-trailing").height(136);
            }
        }
    }
}
