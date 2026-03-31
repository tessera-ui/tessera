//! Material Design list item component.
//!
//! ## Usage
//!
//! Present rows of content in settings, inboxes, or selection lists.

use tessera_ui::{
    Callback, Color, DimensionValue, Dp, Modifier, Px, RenderSlot, State, accesskit::Role,
    provide_context, tessera, use_context,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::boxed,
    column::column,
    modifier::{InteractionState, ModifierExt as _, Padding},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    surface::{SurfaceStyle, surface},
    text::text,
    theme::{ContentColor, MaterialAlpha, MaterialTheme, provide_text_style},
};

/// Colors used by list items in different states.
#[derive(Clone, PartialEq, Copy, Debug)]
pub struct ListItemColors {
    /// Container color when enabled and not selected.
    pub container_color: Color,
    /// Headline text color when enabled and not selected.
    pub headline_color: Color,
    /// Overline text color when enabled and not selected.
    pub overline_color: Color,
    /// Supporting text color when enabled and not selected.
    pub supporting_color: Color,
    /// Leading slot content color when enabled and not selected.
    pub leading_color: Color,
    /// Trailing slot content color when enabled and not selected.
    pub trailing_color: Color,
    /// Container color when disabled.
    pub disabled_container_color: Color,
    /// Headline text color when disabled.
    pub disabled_headline_color: Color,
    /// Overline text color when disabled.
    pub disabled_overline_color: Color,
    /// Supporting text color when disabled.
    pub disabled_supporting_color: Color,
    /// Leading slot content color when disabled.
    pub disabled_leading_color: Color,
    /// Trailing slot content color when disabled.
    pub disabled_trailing_color: Color,
    /// Container color when selected.
    pub selected_container_color: Color,
    /// Headline text color when selected.
    pub selected_headline_color: Color,
    /// Overline text color when selected.
    pub selected_overline_color: Color,
    /// Supporting text color when selected.
    pub selected_supporting_color: Color,
    /// Leading slot content color when selected.
    pub selected_leading_color: Color,
    /// Trailing slot content color when selected.
    pub selected_trailing_color: Color,
}

impl ListItemColors {
    fn container_color(self, enabled: bool, selected: bool) -> Color {
        if !enabled {
            self.disabled_container_color
        } else if selected {
            self.selected_container_color
        } else {
            self.container_color
        }
    }

    fn headline_color(self, enabled: bool, selected: bool) -> Color {
        if !enabled {
            self.disabled_headline_color
        } else if selected {
            self.selected_headline_color
        } else {
            self.headline_color
        }
    }

    fn overline_color(self, enabled: bool, selected: bool) -> Color {
        if !enabled {
            self.disabled_overline_color
        } else if selected {
            self.selected_overline_color
        } else {
            self.overline_color
        }
    }

    fn supporting_color(self, enabled: bool, selected: bool) -> Color {
        if !enabled {
            self.disabled_supporting_color
        } else if selected {
            self.selected_supporting_color
        } else {
            self.supporting_color
        }
    }

    fn leading_color(self, enabled: bool, selected: bool) -> Color {
        if !enabled {
            self.disabled_leading_color
        } else if selected {
            self.selected_leading_color
        } else {
            self.leading_color
        }
    }

    fn trailing_color(self, enabled: bool, selected: bool) -> Color {
        if !enabled {
            self.disabled_trailing_color
        } else if selected {
            self.selected_trailing_color
        } else {
            self.trailing_color
        }
    }
}

/// Default tokens for list item components.
pub struct ListItemDefaults;

impl ListItemDefaults {
    /// Minimum height for a one-line list item container.
    pub const MIN_HEIGHT_ONE_LINE: Dp = Dp(56.0);
    /// Minimum height for a two-line list item container.
    pub const MIN_HEIGHT_TWO_LINE: Dp = Dp(72.0);
    /// Minimum height for a three-line list item container.
    pub const MIN_HEIGHT_THREE_LINE: Dp = Dp(88.0);
    /// Minimum size for leading content slots.
    pub const LEADING_MIN_SIZE: Dp = Dp(24.0);
    /// Minimum size for trailing content slots.
    pub const TRAILING_MIN_SIZE: Dp = Dp(24.0);
    /// Spacing between leading/trailing slots and the main text column.
    pub const INTERNAL_SPACING: Dp = Dp(12.0);
    /// Default content padding applied inside the list item container.
    pub const CONTENT_PADDING: Padding = Padding::new(Dp(16.0), Dp(10.0), Dp(16.0), Dp(10.0));
    /// Default tonal elevation for list items.
    pub const TONAL_ELEVATION: Dp = Dp(0.0);
    /// Default shadow elevation for list items.
    pub const SHADOW_ELEVATION: Dp = Dp(0.0);

    /// Default container shape for list items.
    pub fn shape() -> Shape {
        Shape::RECTANGLE
    }

    /// Default colors for list items derived from the current theme.
    pub fn colors() -> ListItemColors {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        let disabled = scheme
            .on_surface
            .with_alpha(MaterialAlpha::DISABLED_CONTENT);
        ListItemColors {
            container_color: scheme.surface,
            headline_color: scheme.on_surface,
            overline_color: scheme.on_surface_variant,
            supporting_color: scheme.on_surface_variant,
            leading_color: scheme.on_surface_variant,
            trailing_color: scheme.on_surface_variant,
            disabled_container_color: scheme.surface,
            disabled_headline_color: disabled,
            disabled_overline_color: disabled,
            disabled_supporting_color: disabled,
            disabled_leading_color: disabled,
            disabled_trailing_color: disabled,
            selected_container_color: scheme.secondary_container,
            selected_headline_color: scheme.on_secondary_container,
            selected_overline_color: scheme.on_secondary_container,
            selected_supporting_color: scheme.on_secondary_container,
            selected_leading_color: scheme.on_secondary_container,
            selected_trailing_color: scheme.on_secondary_container,
        }
    }

    /// Compute the minimum container height based on content lines.
    pub fn min_height(has_overline: bool, has_supporting: bool) -> Dp {
        if has_overline && has_supporting {
            Self::MIN_HEIGHT_THREE_LINE
        } else if has_overline || has_supporting {
            Self::MIN_HEIGHT_TWO_LINE
        } else {
            Self::MIN_HEIGHT_ONE_LINE
        }
    }
}

/// # list_item
///
/// Render a Material list row with optional leading/trailing slots and
/// supporting text.
///
/// ## Usage
///
/// Present list rows in settings, inboxes, or preference screens.
///
/// ## Parameters
///
/// - `modifier` — optional modifier chain applied to the list item container.
/// - `enabled` — optional enabled state; defaults to `true`.
/// - `selected` — whether the list item is selected.
/// - `headline` — primary headline text.
/// - `overline_text` — optional overline text.
/// - `supporting_text` — optional supporting text.
/// - `leading` — optional leading slot.
/// - `trailing` — optional trailing slot.
/// - `colors` — optional color palette override.
/// - `shape` — optional container shape override.
/// - `content_padding` — optional inner padding override.
/// - `tonal_elevation` — optional tonal elevation override.
/// - `shadow_elevation` — optional shadow elevation override.
/// - `min_height` — optional minimum height override.
/// - `on_click` — optional click callback.
/// - `interaction_state` — optional shared interaction state.
/// - `accessibility_label` — optional accessibility label.
/// - `accessibility_description` — optional accessibility description.
///
/// ## Examples
///
/// ```
/// use tessera_components::list_item::list_item;
/// use tessera_ui::tessera;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// #[tessera]
/// fn demo() {
///     material_theme()
///         .theme(|| MaterialTheme::default())
///         .child(|| {
///             list_item()
///                 .headline("Inbox")
///                 .supporting_text("3 new messages");
///         });
/// }
///
/// demo();
/// ```
#[tessera]
pub fn list_item(
    modifier: Option<Modifier>,
    enabled: Option<bool>,
    selected: bool,
    #[prop(into)] headline: String,
    #[prop(into)] overline_text: Option<String>,
    #[prop(into)] supporting_text: Option<String>,
    leading: Option<RenderSlot>,
    trailing: Option<RenderSlot>,
    colors: Option<ListItemColors>,
    shape: Option<Shape>,
    content_padding: Option<Padding>,
    tonal_elevation: Option<Dp>,
    shadow_elevation: Option<Dp>,
    min_height: Option<Dp>,
    on_click: Option<Callback>,
    interaction_state: Option<State<InteractionState>>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let typography = theme.typography;
    let modifier = modifier.unwrap_or_else(|| Modifier::new().fill_max_width());
    let enabled = enabled.unwrap_or(true);
    let colors = colors.unwrap_or_else(ListItemDefaults::colors);
    let shape = shape.unwrap_or_else(ListItemDefaults::shape);
    let content_padding = content_padding.unwrap_or(ListItemDefaults::CONTENT_PADDING);
    let tonal_elevation = tonal_elevation.unwrap_or(ListItemDefaults::TONAL_ELEVATION);
    let shadow_elevation = shadow_elevation.unwrap_or(ListItemDefaults::SHADOW_ELEVATION);

    let overline_text = overline_text.filter(|value| !value.is_empty());
    let supporting_text = supporting_text.filter(|value| !value.is_empty());
    let has_overline = overline_text.is_some();
    let has_supporting = supporting_text.is_some();
    let min_height =
        min_height.unwrap_or_else(|| ListItemDefaults::min_height(has_overline, has_supporting));
    let content_min_height =
        content_min_height(min_height, content_padding.top, content_padding.bottom);

    let container_color = colors.container_color(enabled, selected);
    let headline_color = colors.headline_color(enabled, selected);
    let overline_color = colors.overline_color(enabled, selected);
    let supporting_color = colors.supporting_color(enabled, selected);
    let leading_color = colors.leading_color(enabled, selected);
    let trailing_color = colors.trailing_color(enabled, selected);

    let accessibility_label =
        accessibility_label.or_else(|| (!headline.is_empty()).then(|| headline.clone()));
    let accessibility_description = accessibility_description
        .or_else(|| supporting_text.clone())
        .or_else(|| overline_text.clone());
    let internal_spacing = ListItemDefaults::INTERNAL_SPACING;
    list_item_surface(ListItemSurfaceArgs {
        modifier,
        container_color,
        shape,
        headline_color,
        enabled,
        tonal_elevation,
        shadow_elevation,
        interaction_state,
        on_click,
        accessibility_label,
        accessibility_description,
    })
    .with_child(move || {
        let headline = headline.clone();
        let leading = leading;
        let trailing = trailing;
        let overline_text = overline_text.clone();
        let supporting_text = supporting_text.clone();
        let row_modifier = Modifier::new()
            .constrain(
                None,
                Some(DimensionValue::Wrap {
                    min: Some(Px::from(content_min_height)),
                    max: None,
                }),
            )
            .padding(content_padding)
            .fill_max_width();

        row()
            .modifier(row_modifier)
            .main_axis_alignment(MainAxisAlignment::Start)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                if let Some(leading) = leading {
                    let color = leading_color;
                    {
                        render_slot()
                            .content_shared(leading)
                            .color(color)
                            .min_size(ListItemDefaults::LEADING_MIN_SIZE);
                    };
                    {
                        spacer().modifier(Modifier::new().width(internal_spacing));
                    };
                }

                let overline_text = overline_text.clone();
                let supporting_text = supporting_text.clone();
                let headline_text = headline.clone();
                column()
                    .modifier(Modifier::new().fill_max_width().weight(1.0))
                    .main_axis_alignment(MainAxisAlignment::Start)
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .children(move || {
                        if let Some(overline) = overline_text.clone() {
                            let color = overline_color;
                            render_text_line()
                                .text_value(overline.clone())
                                .style(typography.label_small)
                                .color(color);
                        }

                        let color = headline_color;
                        if headline_text.is_empty() {
                            spacer().modifier(Modifier::new());
                        } else {
                            render_text_line()
                                .text_value(headline_text.clone())
                                .style(typography.body_large)
                                .color(color);
                        }

                        if let Some(supporting) = supporting_text.clone() {
                            let color = supporting_color;
                            render_text_line()
                                .text_value(supporting.clone())
                                .style(typography.body_medium)
                                .color(color);
                        }
                    });

                if let Some(trailing) = trailing {
                    {
                        spacer().modifier(Modifier::new().width(internal_spacing));
                    };
                    let color = trailing_color;
                    {
                        render_slot()
                            .content_shared(trailing)
                            .color(color)
                            .min_size(ListItemDefaults::TRAILING_MIN_SIZE);
                    };
                }
            });
    });
}

#[tessera]
fn render_text_line(
    #[prop(into)] text_value: String,
    style: crate::theme::TextStyle,
    color: Color,
) {
    provide_text_style(style, move || {
        text().content(text_value.clone()).color(color);
    });
}

#[tessera]
fn render_slot(content: Option<RenderSlot>, color: Color, min_size: Dp) {
    let content = content.unwrap_or_else(RenderSlot::empty);
    provide_context(
        || ContentColor { current: color },
        move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(Modifier::new().size_in(Some(min_size), None, Some(min_size), None))
                .children(move || {
                    {
                        content.render();
                    };
                });
        },
    );
}

struct ListItemSurfaceArgs {
    modifier: Modifier,
    container_color: Color,
    shape: Shape,
    headline_color: Color,
    enabled: bool,
    tonal_elevation: Dp,
    shadow_elevation: Dp,
    interaction_state: Option<State<InteractionState>>,
    on_click: Option<Callback>,
    accessibility_label: Option<String>,
    accessibility_description: Option<String>,
}

fn list_item_surface(args: ListItemSurfaceArgs) -> crate::surface::SurfaceBuilder {
    let builder = surface()
        .modifier(args.modifier)
        .style(SurfaceStyle::Filled {
            color: args.container_color,
        })
        .shape(args.shape)
        .content_color(args.headline_color)
        .enabled(args.enabled)
        .ripple_color(args.headline_color)
        .tonal_elevation(args.tonal_elevation)
        .accessibility_role(Role::ListItem);
    let builder = if args.shadow_elevation.0 > 0.0 {
        builder.elevation(args.shadow_elevation)
    } else {
        builder
    };
    let builder = if let Some(interaction_state) = args.interaction_state {
        builder.interaction_state(interaction_state)
    } else {
        builder
    };
    let builder = if let Some(on_click) = args.on_click {
        builder
            .on_click_shared(on_click)
            .accessibility_focusable(true)
    } else {
        builder
    };
    let builder = if let Some(accessibility_label) = args.accessibility_label {
        builder.accessibility_label(accessibility_label)
    } else {
        builder
    };
    if let Some(accessibility_description) = args.accessibility_description {
        builder.accessibility_description(accessibility_description)
    } else {
        builder
    }
}

fn content_min_height(min_height: Dp, top_padding: Dp, bottom_padding: Dp) -> Dp {
    let total_padding = top_padding.0 + bottom_padding.0;
    let value = (min_height.0 - total_padding).max(0.0);
    Dp(value)
}
