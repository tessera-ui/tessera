//! Material Design chip components.
//!
//! ## Usage
//!
//! Present compact actions, filters, or input tokens in dense UIs.
use tessera_ui::{
    Callback, Color, Dp, LayoutResult, MeasurementError, Modifier, Px, PxPosition,
    accesskit::{Role, Toggled},
    current_frame_nanos,
    layout::{MeasureScope, layout},
    provide_context, receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment},
    animation::shape_motion::ShapeMotion,
    boxed::boxed,
    icon::icon,
    image::image,
    material_icons::filled,
    modifier::{InteractionState, ModifierExt as _, Padding},
    painter::Painter,
    row::row,
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    surface::{SurfaceStyle, surface},
    text::text,
    theme::{MaterialAlpha, MaterialColorScheme, MaterialTheme, TextStyle},
};

use motion::{Elevation, Interaction};

mod motion;

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

/// Shape behavior for selectable chips.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ChipShapeStyle {
    /// Medium corners at rest, rounded ends when selected, and small pressed
    /// corners.
    #[default]
    Expressive,
    /// Fixed small corners in every selection and interaction state.
    Standard,
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
    /// Diameter of an input chip avatar.
    pub const AVATAR_SIZE: Dp = Dp(24.0);
    /// Spacing between icons and label content.
    pub const ELEMENT_SPACING: Dp = Dp(8.0);
    /// Horizontal padding applied to most chip variants.
    pub const HORIZONTAL_PADDING: Dp = Dp(8.0);
    /// Input chip horizontal padding when no leading or trailing icon exists.
    pub const INPUT_EDGE_PADDING: Dp = Dp(4.0);
    /// Input chip horizontal padding when leading or trailing icon is present.
    pub const INPUT_ICON_PADDING: Dp = Dp(8.0);

    /// Fixed standard shape for chip containers.
    pub fn shape() -> Shape {
        Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(Dp(8.0), 2.0),
            top_right: RoundedCorner::manual(Dp(8.0), 2.0),
            bottom_right: RoundedCorner::manual(Dp(8.0), 2.0),
            bottom_left: RoundedCorner::manual(Dp(8.0), 2.0),
        }
    }

    /// Default colors for chips by variant and style.
    pub fn colors(
        scheme: &MaterialColorScheme,
        variant: ChipVariant,
        style: ChipStyle,
    ) -> ChipColors {
        let scheme = scheme.clone();
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
    pub fn border(
        scheme: &MaterialColorScheme,
        variant: ChipVariant,
        style: ChipStyle,
    ) -> Option<ChipBorder> {
        if matches!(style, ChipStyle::Elevated) {
            return None;
        }

        let scheme = scheme.clone();
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
    leading_icon: Option<Painter>,
    avatar: Option<Painter>,
    trailing_icon: Option<Painter>,
    selected: bool,
    enabled: bool,
    modifier: Modifier,
    colors: Option<ChipColors>,
    border: Option<ChipBorder>,
    shape: Option<Shape>,
    shape_style: ChipShapeStyle,
    elevation: Option<Dp>,
    on_click: Option<Callback>,
    on_remove: Option<Callback>,
    remove_accessibility_label: Option<String>,
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
    pub fn leading_icon(mut self, icon: impl Into<Painter>) -> Self {
        self.props.leading_icon = Some(icon.into());
        self
    }

    /// Clears the leading icon content.
    pub fn clear_leading_icon(mut self) -> Self {
        self.props.leading_icon = None;
        self
    }

    /// Sets the trailing icon content using any supported icon source.
    pub fn trailing_icon(mut self, icon: impl Into<Painter>) -> Self {
        self.props.trailing_icon = Some(icon.into());
        self
    }

    /// Clears the trailing icon content.
    pub fn clear_trailing_icon(mut self) -> Self {
        self.props.trailing_icon = None;
        self
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
/// - `variant` — optional chip variant controlling default tokens and behavior.
/// - `style` — optional chip container style.
/// - `label` — visible chip label text.
/// - `leading_icon` — optional leading icon content.
/// - `trailing_icon` — optional trailing icon content.
/// - `avatar` — optional raster or vector input avatar; takes precedence over
///   the leading icon and is ignored by other variants.
/// - `selected` — optional selected state used by selectable variants.
/// - `enabled` — optional enabled flag.
/// - `modifier` — modifier chain applied to the chip subtree.
/// - `colors` — optional chip color override set.
/// - `border` — optional chip border override.
/// - `shape` — optional chip shape override.
/// - `shape_style` — optional selectable-chip shape behavior; defaults to
///   Expressive and is replaced by an explicit `shape`.
/// - `elevation` — optional chip elevation override.
/// - `on_click` — optional body click callback.
/// - `on_remove` — optional input-chip removal action; independent from body
///   activation when both callbacks are provided.
/// - `remove_accessibility_label` — optional removal label; defaults to "Remove
///   {label}".
/// - `accessibility_label` — optional accessibility label.
/// - `accessibility_description` — optional accessibility description.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::{chip::chip, image::ImageData};
/// use tessera_ui::{Callback, remember};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// let selected = remember(|| false);
/// let toggle = Callback::new(move || selected.set(!selected.get()));
/// toggle.call();
/// assert!(selected.get());
/// let avatar = ImageData::from_rgba(vec![255, 128, 0, 255], 1, 1);
/// chip()
///     .input("Alex")
///     .avatar(avatar)
///     .selected(selected.get())
///     .on_click_shared(toggle);
/// #     });
/// # }
/// # component();
/// ```
#[tessera]
pub fn chip(
    variant: Option<ChipVariant>,
    style: Option<ChipStyle>,
    #[prop(into)] label: Option<String>,
    #[prop(skip_setter)] leading_icon: Option<Painter>,
    #[prop(skip_setter)] trailing_icon: Option<Painter>,
    #[prop(into)] avatar: Option<Painter>,
    selected: Option<bool>,
    enabled: Option<bool>,
    modifier: Option<Modifier>,
    colors: Option<ChipColors>,
    border: Option<ChipBorder>,
    shape: Option<Shape>,
    shape_style: Option<ChipShapeStyle>,
    elevation: Option<Dp>,
    on_click: Option<Callback>,
    on_remove: Option<Callback>,
    #[prop(into)] remove_accessibility_label: Option<String>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let label = label.unwrap_or_default();
    let modifier = modifier.unwrap_or_default();
    let mut args = ChipResolvedArgs {
        variant: variant.unwrap_or_default(),
        style: style.unwrap_or_default(),
        label,
        leading_icon,
        avatar,
        trailing_icon,
        selected: selected.unwrap_or(false),
        enabled: enabled.unwrap_or(true),
        modifier,
        colors,
        border,
        shape,
        shape_style: shape_style.unwrap_or_default(),
        elevation,
        on_click,
        on_remove,
        remove_accessibility_label,
        accessibility_label,
        accessibility_description,
    };
    let on_remove = if args.variant == ChipVariant::Input {
        args.on_remove
    } else {
        None
    };
    let separate_remove = on_remove.is_some() && args.on_click.is_some();
    let remove_only = on_remove.is_some() && args.on_click.is_none();
    if on_remove.is_some() && args.trailing_icon.is_none() {
        args.trailing_icon = Some(filled::CLOSE_SVG.into());
    }
    if remove_only {
        args.on_click = on_remove;
    }
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let typography = theme.typography;
    let variant = args.variant;
    let style = args.style;
    let selectable = matches!(variant, ChipVariant::Filter | ChipVariant::Input) && !remove_only;
    let selected = args.selected && selectable;
    let custom_colors = args.colors.is_some();
    let colors = args
        .colors
        .unwrap_or_else(|| ChipDefaults::colors(&theme.color_scheme, variant, style));
    let custom_border = args.border.is_some();
    let mut border = args
        .border
        .or_else(|| ChipDefaults::border(&theme.color_scheme, variant, style));
    let interaction_state = remember(InteractionState::new);
    if (!args.enabled || args.on_click.is_none())
        && interaction_state.get() != InteractionState::default()
    {
        interaction_state.set(InteractionState::default());
    }
    let interaction = Interaction::from_state(interaction_state.get());
    if !custom_border && args.enabled && interaction == Interaction::Focus {
        if let Some(border) = &mut border {
            border.color = if variant == ChipVariant::Assist {
                theme.color_scheme.on_surface
            } else {
                theme.color_scheme.on_surface_variant
            };
        }
    }
    let target_elevation = if !args.enabled {
        Dp::ZERO
    } else if let Some(elevation) = args.elevation {
        elevation
    } else if interaction == Interaction::Drag {
        Dp(8.0)
    } else if style == ChipStyle::Elevated {
        if interaction == Interaction::Hover {
            Dp(3.0)
        } else {
            Dp(1.0)
        }
    } else if variant == ChipVariant::Filter && selected && interaction == Interaction::Hover {
        Dp(1.0)
    } else {
        Dp::ZERO
    };
    let elevation = remember(|| Elevation::new(target_elevation));
    let mut next = elevation.get();
    next.update(
        target_elevation,
        interaction,
        args.enabled,
        current_frame_nanos(),
    );
    if next != elevation.get() {
        elevation.set(next);
    }
    if next.is_animating() {
        receive_frame_nanos(move |now| {
            let mut next = elevation.get();
            next.advance(now);
            if next != elevation.get() {
                elevation.set(next);
            }
            if next.is_animating() {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }
    let shadow_elevation = elevation.get().value();
    let target_shape = args.shape.unwrap_or_else(|| {
        if !selectable || args.shape_style == ChipShapeStyle::Standard {
            theme.shapes.small
        } else if interaction == Interaction::Press {
            theme.shapes.small
        } else if selected {
            Shape::CAPSULE
        } else {
            theme.shapes.medium
        }
    });
    let shape_motion = remember(|| ShapeMotion::new(target_shape));
    let mut next = shape_motion.get();
    next.retarget(
        target_shape,
        args.enabled
            && args.shape.is_none()
            && selectable
            && args.shape_style == ChipShapeStyle::Expressive,
        current_frame_nanos(),
    );
    if next != shape_motion.get() {
        shape_motion.set(next);
    }
    if next.is_animating() {
        receive_frame_nanos(move |now| {
            let mut next = shape_motion.get();
            next.advance(now);
            if next != shape_motion.get() {
                shape_motion.set(next);
            }
            if next.is_animating() {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }
    let shape = shape_motion.get().shape();
    let avatar = if variant == ChipVariant::Input {
        args.avatar
    } else {
        None
    };
    let padding = chip_padding(
        variant,
        avatar.is_some(),
        args.leading_icon.is_some(),
        args.trailing_icon.is_some(),
    );

    let container_color = colors.container_color(args.enabled, selected);
    let label_color = colors.label_color(args.enabled, selected);
    let leading_icon_color = if !custom_colors
        && args.enabled
        && variant == ChipVariant::Input
        && matches!(
            interaction,
            Interaction::Hover | Interaction::Focus | Interaction::Press
        ) {
        theme.color_scheme.primary
    } else {
        colors.leading_icon_color(args.enabled, selected)
    };
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
    let remove_label = args
        .remove_accessibility_label
        .unwrap_or_else(|| format!("Remove {label}"));
    let removal_icon = args.trailing_icon.clone();
    let accessibility_label = args.accessibility_label.or_else(|| {
        if remove_only {
            Some(remove_label.clone())
        } else {
            (!label.is_empty()).then(|| label.clone())
        }
    });

    let modifier = args
        .modifier
        .default_min_size(
            if separate_remove { Dp(88.0) } else { Dp::ZERO },
            ChipDefaults::HEIGHT,
        )
        .minimum_interactive_component_size();
    let has_on_click = args.on_click.is_some();
    let leading_icon = args.leading_icon;
    let trailing_icon = args.trailing_icon;
    let has_label = !label.is_empty();
    let avatar_alpha = if args.enabled {
        1.0
    } else {
        MaterialAlpha::DISABLED_CONTENT
    };

    layout()
        .policy(ChipRemovalLayout {
            separate_remove,
            trailing_padding: padding.right,
        })
        .child(move || {
            let avatar = avatar.clone();
            let leading_icon = leading_icon.clone();
            let trailing_icon = trailing_icon.clone();
            let label = label.clone();
            surface()
                .modifier(modifier.clone())
                .style(surface_style.clone())
                .shape(shape)
                .content_alignment(Alignment::Center)
                .content_color(label_color)
                .enabled(args.enabled)
                .ripple_color(label_color)
                .elevation(shadow_elevation)
                .interaction_state(interaction_state)
                .on_click_optional(args.on_click)
                .accessibility_role_optional((has_on_click || selectable).then_some(
                    if selectable {
                        Role::CheckBox
                    } else {
                        Role::Button
                    },
                ))
                .accessibility_toggled_optional(selectable.then_some(if selected {
                    Toggled::True
                } else {
                    Toggled::False
                }))
                .accessibility_focusable_optional(has_on_click.then_some(true))
                .accessibility_label_optional(accessibility_label.clone())
                .accessibility_description_optional(args.accessibility_description.clone())
                .child(move || {
                    let avatar = avatar.clone();
                    let leading_icon = leading_icon.clone();
                    let trailing_icon = trailing_icon.clone();
                    let label = label.clone();
                    provide_context(
                        || typography.label_large,
                        move || {
                            let inherited = use_context::<TextStyle>().expect("chip text style");
                            if inherited.get() != typography.label_large {
                                inherited.set(typography.label_large);
                            }
                            layout()
                                .modifier(Modifier::new().padding(padding))
                                .child(move || {
                                    let avatar = avatar.clone();
                                    let leading_icon = leading_icon.clone();
                                    let trailing_icon = trailing_icon.clone();
                                    let label = label.clone();
                                    row()
                                        .cross_axis_alignment(CrossAxisAlignment::Center)
                                        .children(move || {
                                            let spacing = ChipDefaults::ELEMENT_SPACING;
                                            let mut item_count = 0;

                                            if let Some(avatar) = avatar.clone() {
                                                item_count += 1;
                                                chip_avatar().painter(avatar).alpha(avatar_alpha);
                                            } else if let Some(icon_content) = leading_icon.clone()
                                            {
                                                if item_count > 0 {
                                                    spacer()
                                                        .modifier(Modifier::new().width(spacing));
                                                }
                                                item_count += 1;
                                                icon()
                                                    .painter(icon_content.clone())
                                                    .size(ChipDefaults::ICON_SIZE)
                                                    .tint(leading_icon_color);
                                            }

                                            if has_label {
                                                if item_count > 0 {
                                                    spacer()
                                                        .modifier(Modifier::new().width(spacing));
                                                }
                                                item_count += 1;
                                                text().content(label.clone());
                                            }

                                            if let Some(icon_content) = trailing_icon.clone() {
                                                if item_count > 0 {
                                                    spacer()
                                                        .modifier(Modifier::new().width(spacing));
                                                }
                                                if separate_remove {
                                                    spacer().modifier(Modifier::new().size(
                                                        ChipDefaults::ICON_SIZE,
                                                        ChipDefaults::ICON_SIZE,
                                                    ));
                                                } else {
                                                    icon()
                                                        .painter(icon_content.clone())
                                                        .size(ChipDefaults::ICON_SIZE)
                                                        .tint(trailing_icon_color);
                                                }
                                            }
                                        });
                                });
                        },
                    );
                });
            if separate_remove {
                chip_remove_action(remove_label.clone(), on_remove.expect("input chip removal"))
                    .icon_optional(removal_icon.clone())
                    .enabled(args.enabled)
                    .color(trailing_icon_color);
            }
        });
}

#[derive(Clone, Copy, PartialEq)]
struct ChipRemovalLayout {
    separate_remove: bool,
    trailing_padding: Dp,
}

impl tessera_ui::LayoutModifierNode for ChipRemovalLayout {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let children = scope.children();
        let constraints = scope.parent_constraint().without_min();
        let body = children[0];
        let size = body.measure(scope, &constraints)?.size();
        let mut result = LayoutResult::new(size);
        result.place_child(body, PxPosition::ZERO);
        if self.separate_remove {
            let remove = children[1];
            let remove_size = remove.measure(scope, &constraints)?.size();
            let center = size.width
                - Px::from(self.trailing_padding)
                - Px::from(ChipDefaults::ICON_SIZE) / 2;
            result.place_child(
                remove,
                PxPosition::new(
                    center - remove_size.width / 2,
                    (size.height - remove_size.height) / 2,
                ),
            );
        }
        Ok(result)
    }
}

#[tessera]
fn chip_remove_action(
    icon: Option<Painter>,
    enabled: Option<bool>,
    color: Option<Color>,
    label: String,
    on_remove: Callback,
) {
    let enabled = enabled.unwrap_or(true);
    surface()
        .modifier(Modifier::new().size(Dp(48.0), Dp(48.0)))
        .shape(Shape::Ellipse)
        .style(Color::TRANSPARENT.into())
        .content_alignment(Alignment::Center)
        .enabled(enabled)
        .accessibility_role(Role::Button)
        .accessibility_label(label)
        .accessibility_focusable(enabled)
        .ripple_color(color.unwrap_or(Color::BLACK))
        .on_click_shared(on_remove)
        .child(move || {
            if let Some(icon) = icon.clone() {
                crate::icon::icon()
                    .painter(icon)
                    .size(ChipDefaults::ICON_SIZE)
                    .tint_optional(color);
            }
        });
}

#[tessera]
fn chip_avatar(painter: Option<Painter>, alpha: Option<f32>) {
    let painter = painter.expect("chip avatar painter");
    let (width, height) = match &painter {
        Painter::Raster(data) => (f64::from(data.width), f64::from(data.height)),
        Painter::Vector(data) => (
            f64::from(data.viewport_width),
            f64::from(data.viewport_height),
        ),
    };
    let shortest = width.min(height);
    let scale = if shortest.is_finite() && shortest > 0.0 {
        ChipDefaults::AVATAR_SIZE.0 / shortest
    } else {
        1.0
    };
    let width = Dp(width * scale);
    let height = Dp(height * scale);
    boxed()
        .modifier(
            Modifier::new()
                .size(ChipDefaults::AVATAR_SIZE, ChipDefaults::AVATAR_SIZE)
                .clip_shape(Shape::Ellipse)
                .alpha(alpha.unwrap_or(1.0)),
        )
        .alignment(Alignment::Center)
        .children(move || {
            image()
                .painter(painter.clone())
                .modifier(Modifier::new().required_size(width, height));
        });
}

fn chip_padding(
    variant: ChipVariant,
    has_avatar: bool,
    has_leading_icon: bool,
    has_trailing_icon: bool,
) -> Padding {
    let (start, end) = match variant {
        ChipVariant::Input => {
            let start = if has_leading_icon && !has_avatar {
                ChipDefaults::INPUT_ICON_PADDING
            } else {
                ChipDefaults::INPUT_EDGE_PADDING
            };
            let end = if has_trailing_icon {
                ChipDefaults::INPUT_ICON_PADDING
            } else {
                ChipDefaults::INPUT_EDGE_PADDING
            };
            (start, end)
        }
        _ => (
            ChipDefaults::HORIZONTAL_PADDING,
            ChipDefaults::HORIZONTAL_PADDING,
        ),
    };
    // Missing icons still reserve the label-to-edge spacing.
    let start = Dp(start.0
        + if has_leading_icon || has_avatar {
            0.0
        } else {
            ChipDefaults::ELEMENT_SPACING.0
        });
    let end = Dp(end.0
        + if has_trailing_icon {
            0.0
        } else {
            ChipDefaults::ELEMENT_SPACING.0
        });
    Padding::new(start, Dp::ZERO, end, Dp::ZERO)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod appearance_tests;

#[cfg(test)]
mod avatar_tests;

#[cfg(test)]
mod shape_tests;

#[cfg(all(test, not(target_family = "wasm")))]
mod remove_tests;
