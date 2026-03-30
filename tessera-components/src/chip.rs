//! Material Design chip components.
//!
//! ## Usage
//!
//! Present compact actions, filters, or input tokens in dense UIs.
use tessera_ui::{
    Callback, Color, Dp, Modifier, accesskit::Role, layout::layout_primitive, tessera, use_context,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment},
    icon::{IconContent, icon},
    modifier::{ModifierExt as _, Padding, ShadowArgs},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    surface::{SurfaceStyle, surface},
    text::text,
    theme::{MaterialAlpha, MaterialTheme, provide_text_style},
};

/// Visual variants supported by [`chip`].
#[derive(Clone, PartialEq, Copy, Debug, Default)]
pub enum ChipVariant {
    /// A compact action chip for context-aware suggestions.
    #[default]
    Assist,
    /// A chip for offering suggested actions or next steps.
    Suggestion,
    /// A selectable chip for filtering content.
    Filter,
    /// A selectable chip representing input with a remove affordance.
    Input,
}

/// Container styles for chips.
#[derive(Clone, PartialEq, Copy, Debug, Default)]
pub enum ChipStyle {
    /// Flat, outlined style.
    #[default]
    Flat,
    /// Elevated style with a subtle shadow.
    Elevated,
}

/// Represents the container and content colors used in a chip in different
/// states.
#[derive(Clone, PartialEq, Copy, Debug)]
pub struct ChipColors {
    /// Container color when enabled and not selected.
    pub container_color: Color,
    /// Label color when enabled and not selected.
    pub label_color: Color,
    /// Leading icon color when enabled and not selected.
    pub leading_icon_color: Color,
    /// Trailing icon color when enabled and not selected.
    pub trailing_icon_color: Color,
    /// Container color when disabled and not selected.
    pub disabled_container_color: Color,
    /// Label color when disabled and not selected.
    pub disabled_label_color: Color,
    /// Leading icon color when disabled and not selected.
    pub disabled_leading_icon_color: Color,
    /// Trailing icon color when disabled and not selected.
    pub disabled_trailing_icon_color: Color,
    /// Container color when enabled and selected.
    pub selected_container_color: Color,
    /// Label color when enabled and selected.
    pub selected_label_color: Color,
    /// Leading icon color when enabled and selected.
    pub selected_leading_icon_color: Color,
    /// Trailing icon color when enabled and selected.
    pub selected_trailing_icon_color: Color,
    /// Container color when disabled and selected.
    pub disabled_selected_container_color: Color,
    /// Label color when disabled and selected.
    pub disabled_selected_label_color: Color,
    /// Leading icon color when disabled and selected.
    pub disabled_selected_leading_icon_color: Color,
    /// Trailing icon color when disabled and selected.
    pub disabled_selected_trailing_icon_color: Color,
}

impl ChipColors {
    fn container_color(self, enabled: bool, selected: bool) -> Color {
        if enabled {
            if selected {
                self.selected_container_color
            } else {
                self.container_color
            }
        } else if selected {
            self.disabled_selected_container_color
        } else {
            self.disabled_container_color
        }
    }

    fn label_color(self, enabled: bool, selected: bool) -> Color {
        if enabled {
            if selected {
                self.selected_label_color
            } else {
                self.label_color
            }
        } else if selected {
            self.disabled_selected_label_color
        } else {
            self.disabled_label_color
        }
    }

    fn leading_icon_color(self, enabled: bool, selected: bool) -> Color {
        if enabled {
            if selected {
                self.selected_leading_icon_color
            } else {
                self.leading_icon_color
            }
        } else if selected {
            self.disabled_selected_leading_icon_color
        } else {
            self.disabled_leading_icon_color
        }
    }

    fn trailing_icon_color(self, enabled: bool, selected: bool) -> Color {
        if enabled {
            if selected {
                self.selected_trailing_icon_color
            } else {
                self.trailing_icon_color
            }
        } else if selected {
            self.disabled_selected_trailing_icon_color
        } else {
            self.disabled_trailing_icon_color
        }
    }
}

/// Represents a border stroke for chip containers.
#[derive(Clone, PartialEq, Copy, Debug)]
pub struct ChipBorder {
    /// Border width when enabled and not selected.
    pub width: Dp,
    /// Border color when enabled and not selected.
    pub color: Color,
    /// Border width when enabled and selected.
    pub selected_width: Dp,
    /// Border color when enabled and selected.
    pub selected_color: Color,
    /// Border color when disabled and not selected.
    pub disabled_color: Color,
    /// Border color when disabled and selected.
    pub disabled_selected_color: Color,
}

impl ChipBorder {
    fn resolve(self, enabled: bool, selected: bool) -> Option<(Dp, Color)> {
        let width = if selected {
            self.selected_width
        } else {
            self.width
        };

        let color = if enabled {
            if selected {
                self.selected_color
            } else {
                self.color
            }
        } else if selected {
            self.disabled_selected_color
        } else {
            self.disabled_color
        };

        if width.0 <= 0.0 || color.a <= 0.0 {
            None
        } else {
            Some((width, color))
        }
    }
}

/// Default values for chip components.
pub struct ChipDefaults;

impl ChipDefaults {
    /// Minimum height for chips.
    pub const HEIGHT: Dp = Dp(32.0);
    /// Default icon size used inside chips.
    pub const ICON_SIZE: Dp = Dp(18.0);
    /// Spacing between icons and label content.
    pub const ELEMENT_SPACING: Dp = Dp(8.0);
    /// Horizontal padding applied to most chip variants.
    pub const HORIZONTAL_PADDING: Dp = Dp(8.0);
    /// Input chip horizontal padding when no leading or trailing icon exists.
    pub const INPUT_EDGE_PADDING: Dp = Dp(4.0);
    /// Input chip horizontal padding when leading or trailing icon is present.
    pub const INPUT_ICON_PADDING: Dp = Dp(8.0);

    /// Default shape for chip containers.
    pub fn shape() -> Shape {
        use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .shapes
            .small
    }

    /// Default colors for chips by variant and style.
    pub fn colors(variant: ChipVariant, style: ChipStyle) -> ChipColors {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        let disabled_content = scheme
            .on_surface
            .with_alpha(MaterialAlpha::DISABLED_CONTENT);
        let disabled_container = match style {
            ChipStyle::Elevated => scheme
                .on_surface
                .with_alpha(MaterialAlpha::DISABLED_CONTAINER),
            ChipStyle::Flat => Color::TRANSPARENT,
        };

        match variant {
            ChipVariant::Assist => {
                let container = match style {
                    ChipStyle::Elevated => scheme.surface_container_low,
                    ChipStyle::Flat => Color::TRANSPARENT,
                };
                let label = scheme.on_surface;
                let icon = scheme.primary;
                ChipColors {
                    container_color: container,
                    label_color: label,
                    leading_icon_color: icon,
                    trailing_icon_color: icon,
                    disabled_container_color: disabled_container,
                    disabled_label_color: disabled_content,
                    disabled_leading_icon_color: disabled_content,
                    disabled_trailing_icon_color: disabled_content,
                    selected_container_color: container,
                    selected_label_color: label,
                    selected_leading_icon_color: icon,
                    selected_trailing_icon_color: icon,
                    disabled_selected_container_color: disabled_container,
                    disabled_selected_label_color: disabled_content,
                    disabled_selected_leading_icon_color: disabled_content,
                    disabled_selected_trailing_icon_color: disabled_content,
                }
            }
            ChipVariant::Suggestion => {
                let container = match style {
                    ChipStyle::Elevated => scheme.surface_container_low,
                    ChipStyle::Flat => Color::TRANSPARENT,
                };
                let label = scheme.on_surface_variant;
                let icon = scheme.primary;
                ChipColors {
                    container_color: container,
                    label_color: label,
                    leading_icon_color: icon,
                    trailing_icon_color: icon,
                    disabled_container_color: disabled_container,
                    disabled_label_color: disabled_content,
                    disabled_leading_icon_color: disabled_content,
                    disabled_trailing_icon_color: disabled_content,
                    selected_container_color: container,
                    selected_label_color: label,
                    selected_leading_icon_color: icon,
                    selected_trailing_icon_color: icon,
                    disabled_selected_container_color: disabled_container,
                    disabled_selected_label_color: disabled_content,
                    disabled_selected_leading_icon_color: disabled_content,
                    disabled_selected_trailing_icon_color: disabled_content,
                }
            }
            ChipVariant::Filter => {
                let container = match style {
                    ChipStyle::Elevated => scheme.surface_container_low,
                    ChipStyle::Flat => Color::TRANSPARENT,
                };
                let disabled_selected_container = scheme
                    .on_surface
                    .with_alpha(MaterialAlpha::DISABLED_CONTAINER);
                ChipColors {
                    container_color: container,
                    label_color: scheme.on_surface_variant,
                    leading_icon_color: scheme.primary,
                    trailing_icon_color: scheme.on_surface_variant,
                    disabled_container_color: disabled_container,
                    disabled_label_color: disabled_content,
                    disabled_leading_icon_color: disabled_content,
                    disabled_trailing_icon_color: disabled_content,
                    selected_container_color: scheme.secondary_container,
                    selected_label_color: scheme.on_secondary_container,
                    selected_leading_icon_color: scheme.on_secondary_container,
                    selected_trailing_icon_color: scheme.on_secondary_container,
                    disabled_selected_container_color: disabled_selected_container,
                    disabled_selected_label_color: disabled_content,
                    disabled_selected_leading_icon_color: disabled_content,
                    disabled_selected_trailing_icon_color: disabled_content,
                }
            }
            ChipVariant::Input => {
                let container = match style {
                    ChipStyle::Elevated => scheme.surface_container_low,
                    ChipStyle::Flat => Color::TRANSPARENT,
                };
                let disabled_selected_container = scheme
                    .on_surface
                    .with_alpha(MaterialAlpha::DISABLED_CONTAINER);
                ChipColors {
                    container_color: container,
                    label_color: scheme.on_surface_variant,
                    leading_icon_color: scheme.on_surface_variant,
                    trailing_icon_color: scheme.on_surface_variant,
                    disabled_container_color: disabled_container,
                    disabled_label_color: disabled_content,
                    disabled_leading_icon_color: disabled_content,
                    disabled_trailing_icon_color: disabled_content,
                    selected_container_color: scheme.secondary_container,
                    selected_label_color: scheme.on_secondary_container,
                    selected_leading_icon_color: scheme.primary,
                    selected_trailing_icon_color: scheme.on_secondary_container,
                    disabled_selected_container_color: disabled_selected_container,
                    disabled_selected_label_color: disabled_content,
                    disabled_selected_leading_icon_color: disabled_content,
                    disabled_selected_trailing_icon_color: disabled_content,
                }
            }
        }
    }

    /// Default border stroke configuration for chips by variant and style.
    pub fn border(variant: ChipVariant, style: ChipStyle) -> Option<ChipBorder> {
        if matches!(style, ChipStyle::Elevated) {
            return None;
        }

        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        let outline = scheme.outline_variant;
        let disabled_outline = scheme
            .on_surface
            .with_alpha(MaterialAlpha::DISABLED_CONTAINER);

        let (selected_width, selected_color, disabled_selected_color) = match variant {
            ChipVariant::Assist | ChipVariant::Suggestion => (Dp(1.0), outline, disabled_outline),
            ChipVariant::Filter | ChipVariant::Input => {
                (Dp(0.0), Color::TRANSPARENT, Color::TRANSPARENT)
            }
        };

        Some(ChipBorder {
            width: Dp(1.0),
            color: outline,
            selected_width,
            selected_color,
            disabled_color: disabled_outline,
            disabled_selected_color,
        })
    }

    /// Default elevation used for elevated chips.
    pub fn elevation(style: ChipStyle) -> Option<Dp> {
        match style {
            ChipStyle::Flat => None,
            ChipStyle::Elevated => Some(Dp(1.0)),
        }
    }
}

#[derive(Clone)]
struct ChipResolvedArgs {
    variant: ChipVariant,
    style: ChipStyle,
    label: String,
    leading_icon: Option<IconContent>,
    trailing_icon: Option<IconContent>,
    selected: bool,
    enabled: bool,
    modifier: Modifier,
    colors: Option<ChipColors>,
    border: Option<ChipBorder>,
    shape: Option<Shape>,
    elevation: Option<Dp>,
    on_click: Option<Callback>,
    accessibility_label: Option<String>,
    accessibility_description: Option<String>,
}

impl ChipBuilder {
    /// Applies the assist chip preset and updates the visible label.
    pub fn assist(self, label: impl Into<String>) -> Self {
        self.variant(ChipVariant::Assist).label(label.into())
    }

    /// Applies the suggestion chip preset and updates the visible label.
    pub fn suggestion(self, label: impl Into<String>) -> Self {
        self.variant(ChipVariant::Suggestion).label(label.into())
    }

    /// Applies the filter chip preset and updates the visible label.
    pub fn filter(self, label: impl Into<String>) -> Self {
        self.variant(ChipVariant::Filter).label(label.into())
    }

    /// Applies the input chip preset and updates the visible label.
    pub fn input(self, label: impl Into<String>) -> Self {
        self.variant(ChipVariant::Input).label(label.into())
    }

    /// Sets the leading icon content using any supported icon source.
    pub fn leading_icon(mut self, icon: impl Into<IconContent>) -> Self {
        self.props.leading_icon = Some(icon.into());
        self
    }

    /// Clears the leading icon content.
    pub fn clear_leading_icon(mut self) -> Self {
        self.props.leading_icon = None;
        self
    }

    /// Sets the trailing icon content using any supported icon source.
    pub fn trailing_icon(mut self, icon: impl Into<IconContent>) -> Self {
        self.props.trailing_icon = Some(icon.into());
        self
    }

    /// Clears the trailing icon content.
    pub fn clear_trailing_icon(mut self) -> Self {
        self.props.trailing_icon = None;
        self
    }
}

fn render_chip_icon(icon_content: IconContent, tint: Color) {
    let builder = match icon_content {
        IconContent::Vector(data) => icon().vector(data),
        IconContent::Raster(data) => icon().raster(data),
    };

    builder.size(ChipDefaults::ICON_SIZE).tint(tint);
}

/// # chip
///
/// Renders a compact chip for actions, filters, and input tokens with optional
/// icons.
///
/// ## Usage
///
/// Use for filters, suggestions, and compact actions in dense layouts.
///
/// ## Parameters
///
/// - `variant` — optional chip variant controlling default tokens and behavior.
/// - `style` — optional chip container style.
/// - `label` — visible chip label text.
/// - `leading_icon` — optional leading icon content.
/// - `trailing_icon` — optional trailing icon content.
/// - `selected` — optional selected state used by selectable variants.
/// - `enabled` — optional enabled flag.
/// - `modifier` — modifier chain applied to the chip subtree.
/// - `colors` — optional chip color override set.
/// - `border` — optional chip border override.
/// - `shape` — optional chip shape override.
/// - `elevation` — optional chip elevation override.
/// - `on_click` — optional click callback.
/// - `accessibility_label` — optional accessibility label.
/// - `accessibility_description` — optional accessibility description.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::chip::chip;
/// use tessera_ui::remember;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// let selected = remember(|| false);
/// selected.with_mut(|value| *value = true);
/// chip()
///     .filter("Favorites")
///     .selected(selected.with(|value| *value));
/// #     });
/// # }
/// # component();
/// ```
#[tessera]
pub fn chip(
    variant: Option<ChipVariant>,
    style: Option<ChipStyle>,
    #[prop(into)] label: String,
    #[prop(skip_setter)] leading_icon: Option<IconContent>,
    #[prop(skip_setter)] trailing_icon: Option<IconContent>,
    selected: Option<bool>,
    enabled: Option<bool>,
    modifier: Modifier,
    colors: Option<ChipColors>,
    border: Option<ChipBorder>,
    shape: Option<Shape>,
    elevation: Option<Dp>,
    on_click: Option<Callback>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let args = ChipResolvedArgs {
        variant: variant.unwrap_or_default(),
        style: style.unwrap_or_default(),
        label,
        leading_icon,
        trailing_icon,
        selected: selected.unwrap_or(false),
        enabled: enabled.unwrap_or(true),
        modifier,
        colors,
        border,
        shape,
        elevation,
        on_click,
        accessibility_label,
        accessibility_description,
    };
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let typography = theme.typography;
    let variant = args.variant;
    let style = args.style;
    let selectable = matches!(variant, ChipVariant::Filter | ChipVariant::Input);
    let selected = args.selected && selectable;
    let colors = args
        .colors
        .unwrap_or_else(|| ChipDefaults::colors(variant, style));
    let border = args.border.or_else(|| ChipDefaults::border(variant, style));
    let elevation = args.elevation.or_else(|| ChipDefaults::elevation(style));
    let shape = args.shape.unwrap_or_else(ChipDefaults::shape);
    let padding = chip_padding(
        variant,
        args.leading_icon.is_some(),
        args.trailing_icon.is_some(),
    );

    let container_color = colors.container_color(args.enabled, selected);
    let label_color = colors.label_color(args.enabled, selected);
    let leading_icon_color = colors.leading_icon_color(args.enabled, selected);
    let trailing_icon_color = colors.trailing_icon_color(args.enabled, selected);

    let border_style = border.and_then(|border| border.resolve(args.enabled, selected));
    let surface_style = match border_style {
        Some((border_width, border_color)) => SurfaceStyle::FilledOutlined {
            fill_color: container_color,
            border_color,
            border_width,
        },
        None => SurfaceStyle::Filled {
            color: container_color,
        },
    };

    let label = args.label;
    let accessibility_label = args
        .accessibility_label
        .or_else(|| (!label.is_empty()).then(|| label.clone()));

    let mut modifier = args
        .modifier
        .size_in(None, None, Some(ChipDefaults::HEIGHT), None);
    if matches!(style, ChipStyle::Elevated)
        && let Some(elevation) = elevation
    {
        modifier = modifier.shadow(&ShadowArgs {
            elevation,
            shape,
            ambient_color: Some(theme.color_scheme.shadow.with_alpha(0.12)),
            spot_color: Some(Color::TRANSPARENT),
            ..Default::default()
        });
    }

    let mut surface_builder = surface()
        .modifier(modifier)
        .style(surface_style)
        .shape(shape)
        .content_alignment(Alignment::Center)
        .content_color(label_color)
        .enabled(args.enabled)
        .ripple_color(label_color);

    if !matches!(style, ChipStyle::Elevated)
        && let Some(elevation) = elevation
    {
        surface_builder = surface_builder.elevation(elevation);
    }

    if let Some(on_click) = args.on_click {
        surface_builder = surface_builder
            .on_click_shared(on_click)
            .accessibility_role(Role::Button)
            .accessibility_focusable(true);
    }

    if let Some(label) = accessibility_label {
        surface_builder = surface_builder.accessibility_label(label);
    }
    if let Some(description) = args.accessibility_description {
        surface_builder = surface_builder.accessibility_description(description);
    }

    let leading_icon = args.leading_icon;
    let trailing_icon = args.trailing_icon;
    let has_label = !label.is_empty();

    surface_builder.child(move || {
        let leading_icon = leading_icon.clone();
        let trailing_icon = trailing_icon.clone();
        let label = label.clone();
        provide_text_style(typography.label_large, move || {
            layout_primitive()
                .modifier(Modifier::new().padding(padding))
                .child(move || {
                    let leading_icon = leading_icon.clone();
                    let trailing_icon = trailing_icon.clone();
                    let label = label.clone();
                    row()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children(move || {
                            let spacing = ChipDefaults::ELEMENT_SPACING;
                            let mut item_count = 0;

                            if let Some(icon_content) = leading_icon.clone() {
                                if item_count > 0 {
                                    spacer().modifier(Modifier::new().width(spacing));
                                }
                                item_count += 1;
                                render_chip_icon(icon_content.clone(), leading_icon_color);
                            }

                            if has_label {
                                if item_count > 0 {
                                    spacer().modifier(Modifier::new().width(spacing));
                                }
                                item_count += 1;
                                text().content(label.clone());
                            }

                            if let Some(icon_content) = trailing_icon.clone() {
                                if item_count > 0 {
                                    spacer().modifier(Modifier::new().width(spacing));
                                }
                                render_chip_icon(icon_content.clone(), trailing_icon_color);
                            }
                        });
                });
        });
    });
}

fn chip_padding(variant: ChipVariant, has_leading_icon: bool, has_trailing_icon: bool) -> Padding {
    match variant {
        ChipVariant::Input => {
            let start = if has_leading_icon {
                ChipDefaults::INPUT_ICON_PADDING
            } else {
                ChipDefaults::INPUT_EDGE_PADDING
            };
            let end = if has_trailing_icon {
                ChipDefaults::INPUT_ICON_PADDING
            } else {
                ChipDefaults::INPUT_EDGE_PADDING
            };
            Padding::new(start, Dp(0.0), end, Dp(0.0))
        }
        _ => Padding::symmetric(ChipDefaults::HORIZONTAL_PADDING, Dp(0.0)),
    }
}
