//! Material Design chip components.
//!
//! ## Usage
//!
//! Present compact actions, filters, or input tokens in dense UIs.
use std::sync::Arc;

use derive_setters::Setters;
use tessera_ui::{Color, Dp, Modifier, accesskit::Role, tessera, use_context};

use crate::{
    alignment::{Alignment, CrossAxisAlignment},
    icon::{IconArgs, icon},
    modifier::{ModifierExt as _, Padding},
    row::{RowArgs, row},
    shape_def::Shape,
    spacer::spacer,
    surface::{SurfaceArgs, SurfaceStyle, surface},
    text::{TextArgs, text},
    theme::{MaterialAlpha, MaterialTheme, provide_text_style},
};

/// Visual variants supported by [`chip`].
#[derive(Clone, Copy, Debug, Default)]
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
#[derive(Clone, Copy, Debug, Default)]
pub enum ChipStyle {
    /// Flat, outlined style.
    #[default]
    Flat,
    /// Elevated style with a subtle shadow.
    Elevated,
}

/// Represents the container and content colors used in a chip in different
/// states.
#[derive(Clone, Copy, Debug)]
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
#[derive(Clone, Copy, Debug)]
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
        use_context::<MaterialTheme>().get().shapes.small
    }

    /// Default colors for chips by variant and style.
    pub fn colors(variant: ChipVariant, style: ChipStyle) -> ChipColors {
        let scheme = use_context::<MaterialTheme>().get().color_scheme;
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

        let scheme = use_context::<MaterialTheme>().get().color_scheme;
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

/// Arguments for the [`chip`] component.
#[derive(Clone, Setters)]
pub struct ChipArgs {
    /// Variant of the chip.
    pub variant: ChipVariant,
    /// Visual style of the chip.
    pub style: ChipStyle,
    /// Text label rendered inside the chip.
    #[setters(into)]
    pub label: String,
    /// Optional leading icon shown before the label.
    #[setters(strip_option, into)]
    pub leading_icon: Option<IconArgs>,
    /// Optional trailing icon shown after the label.
    #[setters(strip_option, into)]
    pub trailing_icon: Option<IconArgs>,
    /// Whether the chip is selected (used by selectable variants).
    pub selected: bool,
    /// Whether the chip is enabled.
    pub enabled: bool,
    /// Optional modifier chain applied to the chip subtree.
    pub modifier: Modifier,
    /// Optional colors override.
    #[setters(strip_option)]
    pub colors: Option<ChipColors>,
    /// Optional border override.
    #[setters(strip_option)]
    pub border: Option<ChipBorder>,
    /// Shape of the chip container.
    pub shape: Shape,
    /// Optional elevation override.
    #[setters(strip_option)]
    pub elevation: Option<Dp>,
    /// Optional click handler for the chip.
    #[setters(skip)]
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Optional accessibility label announced by assistive technologies.
    #[setters(strip_option, into)]
    pub accessibility_label: Option<String>,
    /// Optional accessibility description announced by assistive technologies.
    #[setters(strip_option, into)]
    pub accessibility_description: Option<String>,
}

impl ChipArgs {
    /// Sets the on_click handler.
    pub fn on_click<F>(mut self, on_click: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Arc::new(on_click));
        self
    }

    /// Sets the on_click handler using a shared callback.
    pub fn on_click_shared(mut self, on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.on_click = Some(on_click);
        self
    }
}

impl ChipArgs {
    /// Creates a default assist chip configuration with the provided label.
    pub fn assist(label: impl Into<String>) -> Self {
        ChipArgs::default()
            .variant(ChipVariant::Assist)
            .label(label)
    }

    /// Creates a default suggestion chip configuration with the provided label.
    pub fn suggestion(label: impl Into<String>) -> Self {
        ChipArgs::default()
            .variant(ChipVariant::Suggestion)
            .label(label)
    }

    /// Creates a default filter chip configuration with the provided label.
    pub fn filter(label: impl Into<String>) -> Self {
        ChipArgs::default()
            .variant(ChipVariant::Filter)
            .label(label)
    }

    /// Creates a default input chip configuration with the provided label.
    pub fn input(label: impl Into<String>) -> Self {
        ChipArgs::default().variant(ChipVariant::Input).label(label)
    }
}

impl Default for ChipArgs {
    fn default() -> Self {
        Self {
            variant: ChipVariant::default(),
            style: ChipStyle::default(),
            label: String::new(),
            leading_icon: None,
            trailing_icon: None,
            selected: false,
            enabled: true,
            modifier: Modifier::new(),
            colors: None,
            border: None,
            shape: ChipDefaults::shape(),
            elevation: None,
            on_click: None,
            accessibility_label: None,
            accessibility_description: None,
        }
    }
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
/// - `args` — configures the chip label, variant, and appearance; see
///   [`ChipArgs`].
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_ui::remember;
/// use tessera_ui_basic_components::chip::{ChipArgs, chip};
///
/// let selected = remember(|| false);
/// selected.with_mut(|value| *value = true);
/// let args = ChipArgs::filter("Favorites").selected(selected.with(|value| *value));
/// assert!(args.selected);
/// chip(args);
/// # }
/// # component();
/// ```
#[tessera]
pub fn chip(args: impl Into<ChipArgs>) {
    let args: ChipArgs = args.into();
    let theme = use_context::<MaterialTheme>().get();
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

    let mut surface_args = SurfaceArgs::default()
        .modifier(
            args.modifier
                .size_in(None, None, Some(ChipDefaults::HEIGHT), None),
        )
        .style(surface_style)
        .shape(args.shape)
        .content_alignment(Alignment::Center)
        .content_color(label_color)
        .enabled(args.enabled)
        .ripple_color(label_color);

    if let Some(elevation) = elevation {
        surface_args = surface_args.elevation(elevation);
    }

    if let Some(on_click) = args.on_click {
        surface_args = surface_args
            .on_click_shared(on_click)
            .accessibility_role(Role::Button)
            .accessibility_focusable(true);
    }

    if let Some(label) = accessibility_label {
        surface_args = surface_args.accessibility_label(label);
    }
    if let Some(description) = args.accessibility_description {
        surface_args = surface_args.accessibility_description(description);
    }

    let leading_icon = args.leading_icon;
    let trailing_icon = args.trailing_icon;
    let has_label = !label.is_empty();

    surface(surface_args, move || {
        provide_text_style(typography.label_large, move || {
            Modifier::new().padding(padding).run(move || {
                row(
                    RowArgs::default().cross_axis_alignment(CrossAxisAlignment::Center),
                    move |scope| {
                        let spacing = ChipDefaults::ELEMENT_SPACING;
                        let mut item_count = 0;

                        if let Some(mut icon_args) = leading_icon {
                            if item_count > 0 {
                                scope.child(move || spacer(Modifier::new().width(spacing)));
                            }
                            item_count += 1;
                            icon_args.size = ChipDefaults::ICON_SIZE;
                            icon_args.tint = leading_icon_color;
                            scope.child(move || icon(icon_args));
                        }

                        if has_label {
                            if item_count > 0 {
                                scope.child(move || spacer(Modifier::new().width(spacing)));
                            }
                            item_count += 1;
                            scope.child(move || {
                                text(TextArgs::default().text(label));
                            });
                        }

                        if let Some(mut icon_args) = trailing_icon {
                            if item_count > 0 {
                                scope.child(move || spacer(Modifier::new().width(spacing)));
                            }
                            icon_args.size = ChipDefaults::ICON_SIZE;
                            icon_args.tint = trailing_icon_color;
                            scope.child(move || icon(icon_args));
                        }
                    },
                );
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
            Padding::only(start, Dp(0.0), end, Dp(0.0))
        }
        _ => Padding::symmetric(ChipDefaults::HORIZONTAL_PADDING, Dp(0.0)),
    }
}
