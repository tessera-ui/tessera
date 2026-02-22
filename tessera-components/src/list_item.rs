//! Material Design list item component.
//!
//! ## Usage
//!
//! Present rows of content in settings, inboxes, or selection lists.

use derive_setters::Setters;
use tessera_ui::{
    Callback, Color, DimensionValue, Dp, Modifier, Px, RenderSlot, State, accesskit::Role,
    provide_context, tessera, use_context,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::{BoxedArgs, boxed},
    column::{ColumnArgs, column},
    modifier::{InteractionState, ModifierExt as _, Padding},
    row::{RowArgs, row},
    shape_def::Shape,
    spacer::spacer,
    surface::{SurfaceArgs, SurfaceStyle, surface},
    text::{TextArgs, text},
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

/// Arguments for the [`list_item`] component.
#[derive(PartialEq, Clone, Setters)]
pub struct ListItemArgs {
    /// Modifier chain applied to the list item container.
    pub modifier: Modifier,
    /// Whether the list item is enabled for interaction.
    pub enabled: bool,
    /// Whether the list item is selected.
    pub selected: bool,
    /// Headline text displayed as the primary label.
    #[setters(into)]
    pub headline: String,
    /// Optional overline text displayed above the headline.
    #[setters(strip_option, into)]
    pub overline_text: Option<String>,
    /// Optional supporting text displayed below the headline.
    #[setters(strip_option, into)]
    pub supporting_text: Option<String>,
    /// Optional leading slot content (icon/avatar/etc.).
    #[setters(skip)]
    pub leading: Option<RenderSlot>,
    /// Optional trailing slot content (icon/switch/etc.).
    #[setters(skip)]
    pub trailing: Option<RenderSlot>,
    /// Colors used to render the list item.
    pub colors: ListItemColors,
    /// Shape of the list item container.
    pub shape: Shape,
    /// Inner padding applied to the list item content.
    pub content_padding: Padding,
    /// Tonal elevation applied to the list item surface.
    pub tonal_elevation: Dp,
    /// Shadow elevation applied to the list item surface.
    pub shadow_elevation: Dp,
    /// Optional minimum height override for the container.
    #[setters(strip_option)]
    pub min_height: Option<Dp>,
    /// Optional click handler for the list item.
    #[setters(skip)]
    pub on_click: Option<Callback>,
    /// Optional shared interaction state for hover/press feedback.
    #[setters(strip_option)]
    pub interaction_state: Option<State<InteractionState>>,
    /// Optional accessibility label announced by assistive technologies.
    #[setters(strip_option, into)]
    pub accessibility_label: Option<String>,
    /// Optional accessibility description announced by assistive technologies.
    #[setters(strip_option, into)]
    pub accessibility_description: Option<String>,
}

impl ListItemArgs {
    /// Create args with the required headline text.
    pub fn new(headline: impl Into<String>) -> Self {
        Self::default().headline(headline)
    }

    /// Set the leading slot content.
    pub fn leading<F>(mut self, leading: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.leading = Some(RenderSlot::new(leading));
        self
    }

    /// Set the leading slot content using a shared callback.
    pub fn leading_shared(mut self, leading: impl Into<RenderSlot>) -> Self {
        self.leading = Some(leading.into());
        self
    }

    /// Set the trailing slot content.
    pub fn trailing<F>(mut self, trailing: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.trailing = Some(RenderSlot::new(trailing));
        self
    }

    /// Set the trailing slot content using a shared callback.
    pub fn trailing_shared(mut self, trailing: impl Into<RenderSlot>) -> Self {
        self.trailing = Some(trailing.into());
        self
    }

    /// Set the click handler.
    pub fn on_click<F>(mut self, on_click: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Callback::new(on_click));
        self
    }

    /// Set the click handler using a shared callback.
    pub fn on_click_shared(mut self, on_click: impl Into<Callback>) -> Self {
        self.on_click = Some(on_click.into());
        self
    }
}

impl Default for ListItemArgs {
    fn default() -> Self {
        Self {
            modifier: Modifier::new().fill_max_width(),
            enabled: true,
            selected: false,
            headline: String::new(),
            overline_text: None,
            supporting_text: None,
            leading: None,
            trailing: None,
            colors: ListItemDefaults::colors(),
            shape: ListItemDefaults::shape(),
            content_padding: ListItemDefaults::CONTENT_PADDING,
            tonal_elevation: ListItemDefaults::TONAL_ELEVATION,
            shadow_elevation: ListItemDefaults::SHADOW_ELEVATION,
            min_height: None,
            on_click: None,
            interaction_state: None,
            accessibility_label: None,
            accessibility_description: None,
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
/// - `args` — configures list item content, colors, and interaction; see
///   [`ListItemArgs`].
///
/// ## Examples
///
/// ```
/// use tessera_components::list_item::{ListItemArgs, list_item};
/// use tessera_ui::tessera;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// #[tessera]
/// fn demo() {
/// #     let args = tessera_components::theme::MaterialThemeProviderArgs::new(
/// #         || MaterialTheme::default(),
/// #         || {
///     let args = ListItemArgs::new("Inbox").supporting_text("3 new messages");
///     assert_eq!(args.headline, "Inbox");
///     list_item(&args);
/// #         },
/// #     );
/// #     material_theme(&args);
/// }
///
/// demo();
/// ```
#[tessera]
pub fn list_item(args: &ListItemArgs) {
    let args = args.clone();
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let typography = theme.typography;

    let overline_text = args.overline_text.clone().filter(|value| !value.is_empty());
    let supporting_text = args
        .supporting_text
        .clone()
        .filter(|value| !value.is_empty());
    let has_overline = overline_text.is_some();
    let has_supporting = supporting_text.is_some();
    let min_height = args
        .min_height
        .unwrap_or_else(|| ListItemDefaults::min_height(has_overline, has_supporting));
    let content_min_height = content_min_height(
        min_height,
        args.content_padding.top,
        args.content_padding.bottom,
    );

    let enabled = args.enabled;
    let selected = args.selected;
    let colors = args.colors;
    let container_color = colors.container_color(enabled, selected);
    let headline_color = colors.headline_color(enabled, selected);
    let overline_color = colors.overline_color(enabled, selected);
    let supporting_color = colors.supporting_color(enabled, selected);
    let leading_color = colors.leading_color(enabled, selected);
    let trailing_color = colors.trailing_color(enabled, selected);

    let accessibility_label = args
        .accessibility_label
        .clone()
        .or_else(|| (!args.headline.is_empty()).then(|| args.headline.clone()));
    let accessibility_description = args
        .accessibility_description
        .clone()
        .or_else(|| supporting_text.clone())
        .or_else(|| overline_text.clone());

    let mut surface_args = SurfaceArgs::default()
        .modifier(args.modifier)
        .style(SurfaceStyle::Filled {
            color: container_color,
        })
        .shape(args.shape)
        .content_color(headline_color)
        .enabled(enabled)
        .ripple_color(headline_color)
        .tonal_elevation(args.tonal_elevation)
        .accessibility_role(Role::ListItem);

    if args.shadow_elevation.0 > 0.0 {
        surface_args = surface_args.elevation(args.shadow_elevation);
    }

    if let Some(state) = args.interaction_state {
        surface_args = surface_args.interaction_state(state);
    }

    if let Some(on_click) = args.on_click {
        surface_args = surface_args
            .on_click_shared(on_click)
            .accessibility_focusable(true);
    }

    if let Some(label) = accessibility_label {
        surface_args = surface_args.accessibility_label(label);
    }
    if let Some(description) = accessibility_description {
        surface_args = surface_args.accessibility_description(description);
    }

    let headline = args.headline;
    let leading = args.leading;
    let trailing = args.trailing;
    let internal_spacing = ListItemDefaults::INTERNAL_SPACING;
    let content_padding = args.content_padding;

    surface(&crate::surface::SurfaceArgs::with_child(
        surface_args,
        move || {
            let headline = headline.clone();
            let leading = leading.clone();
            let trailing = trailing.clone();
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

            row(
                RowArgs::default()
                    .modifier(row_modifier)
                    .main_axis_alignment(MainAxisAlignment::Start)
                    .cross_axis_alignment(CrossAxisAlignment::Center),
                move |row_scope| {
                    if let Some(leading) = leading.clone() {
                        let color = leading_color;
                        row_scope.child(move || {
                            render_slot(leading.clone(), color, ListItemDefaults::LEADING_MIN_SIZE);
                        });
                        row_scope.child(move || {
                            spacer(&crate::spacer::SpacerArgs::new(
                                Modifier::new().width(internal_spacing),
                            ));
                        });
                    }

                    let overline_text = overline_text.clone();
                    let supporting_text = supporting_text.clone();
                    let headline_text = headline.clone();
                    row_scope.child_weighted(
                        move || {
                            let overline_text = overline_text.clone();
                            let supporting_text = supporting_text.clone();
                            let headline_text = headline_text.clone();
                            column(
                                ColumnArgs::default()
                                    .modifier(Modifier::new().fill_max_width())
                                    .main_axis_alignment(MainAxisAlignment::Start)
                                    .cross_axis_alignment(CrossAxisAlignment::Start),
                                move |column_scope| {
                                    if let Some(overline) = overline_text.clone() {
                                        let color = overline_color;
                                        column_scope.child(move || {
                                            render_text_line(
                                                overline.clone(),
                                                typography.label_small,
                                                color,
                                            );
                                        });
                                    }

                                    let color = headline_color;
                                    column_scope.child(move || {
                                        if headline_text.is_empty() {
                                            spacer(
                                                &crate::spacer::SpacerArgs::new(Modifier::new()),
                                            );
                                        } else {
                                            render_text_line(
                                                headline_text.clone(),
                                                typography.body_large,
                                                color,
                                            );
                                        }
                                    });

                                    if let Some(supporting) = supporting_text.clone() {
                                        let color = supporting_color;
                                        column_scope.child(move || {
                                            render_text_line(
                                                supporting.clone(),
                                                typography.body_medium,
                                                color,
                                            );
                                        });
                                    }
                                },
                            );
                        },
                        1.0,
                    );

                    if let Some(trailing) = trailing.clone() {
                        row_scope.child(move || {
                            spacer(&crate::spacer::SpacerArgs::new(
                                Modifier::new().width(internal_spacing),
                            ));
                        });
                        let color = trailing_color;
                        row_scope.child(move || {
                            render_slot(
                                trailing.clone(),
                                color,
                                ListItemDefaults::TRAILING_MIN_SIZE,
                            );
                        });
                    }
                },
            );
        },
    ));
}

fn render_text_line(text_value: String, style: crate::theme::TextStyle, color: Color) {
    provide_text_style(style, move || {
        text(&crate::text::TextArgs::from(
            &TextArgs::default().text(&text_value).color(color),
        ));
    });
}

fn render_slot(content: RenderSlot, color: Color, min_size: Dp) {
    provide_context(
        || ContentColor { current: color },
        move || {
            boxed(
                BoxedArgs::default()
                    .alignment(Alignment::Center)
                    .modifier(Modifier::new().size_in(Some(min_size), None, Some(min_size), None)),
                move |scope| {
                    scope.child(move || {
                        content.render();
                    });
                },
            );
        },
    );
}

fn content_min_height(min_height: Dp, top_padding: Dp, bottom_padding: Dp) -> Dp {
    let total_padding = top_padding.0 + bottom_padding.0;
    let value = (min_height.0 - total_padding).max(0.0);
    Dp(value)
}
