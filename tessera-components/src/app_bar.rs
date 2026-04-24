//! Top app bars for app-level navigation and actions.
//!
//! ## Usage
//!
//! Use for screen titles, navigation affordances, and primary actions at the
//! top of a view.
use tessera_ui::{Color, Dp, Modifier, RenderSlot, provide_context, tessera, use_context};

use crate::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    icon_button::icon_button,
    material_icons::filled,
    modifier::{ModifierExt as _, Padding},
    row::row,
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{ContentColor, MaterialColorScheme, MaterialTheme, provide_text_style},
};

/// Default tokens for app bars.
pub struct AppBarDefaults;

impl AppBarDefaults {
    /// Default height for a top app bar.
    pub const TOP_APP_BAR_HEIGHT: Dp = Dp(64.0);
    /// Default elevation for a top app bar.
    pub const TOP_APP_BAR_ELEVATION: Dp = Dp(0.0);
    /// Default horizontal padding applied to app bar content.
    pub const HORIZONTAL_PADDING: Dp = Dp(4.0);
    /// Default total title inset when no navigation icon is present.
    pub const TITLE_INSET: Dp = Dp(16.0);
    /// Default spacing between trailing action items.
    pub const ACTIONS_SPACING: Dp = Dp(0.0);
    /// Default padding applied to app bar content.
    pub const CONTENT_PADDING: Padding = Padding::symmetric(Self::HORIZONTAL_PADDING, Dp(0.0));
    /// Default padding applied to standard top app bar content.
    pub const TOP_APP_BAR_CONTENT_PADDING: Padding = Padding::symmetric(Dp(16.0), Dp(0.0));

    /// Container color for app bars.
    pub fn container_color(scheme: &MaterialColorScheme) -> Color {
        scheme.surface
    }

    /// Title content color for app bars.
    pub fn title_color(scheme: &MaterialColorScheme) -> Color {
        scheme.on_surface
    }

    /// Navigation icon color for app bars.
    pub fn navigation_icon_color(scheme: &MaterialColorScheme) -> Color {
        scheme.on_surface
    }

    /// Action icon color for app bars.
    pub fn action_icon_color(scheme: &MaterialColorScheme) -> Color {
        scheme.on_surface_variant
    }
}

/// # app_bar
///
/// Render a container row for app-level navigation and actions.
///
/// ## Usage
///
/// Use for custom top bars with mixed content like search fields or tabs.
///
/// ## Parameters
///
/// - `modifier` — modifier chain applied to the app bar container.
/// - `container_color` — optional app bar surface color.
/// - `content_color` — optional default content color propagated to
///   descendants.
/// - `elevation` — optional surface elevation.
/// - `content_padding` — optional padding applied to the row content.
/// - `main_axis_alignment` — optional main-axis alignment for the row.
/// - `cross_axis_alignment` — optional cross-axis alignment for the row.
/// - `content` — optional row content slot.
///
/// ## Examples
/// ```rust
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::{app_bar::app_bar, text::text};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// app_bar().content(|| {
///     text().content("Inbox");
/// });
/// #     });
/// # }
/// # component();
/// ```
#[tessera]
pub fn app_bar(
    modifier: Option<Modifier>,
    container_color: Option<Color>,
    content_color: Option<Color>,
    elevation: Option<Dp>,
    content_padding: Option<Padding>,
    main_axis_alignment: Option<MainAxisAlignment>,
    cross_axis_alignment: Option<CrossAxisAlignment>,
    content: Option<RenderSlot>,
) {
    let modifier = modifier.unwrap_or_default();
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let container_color =
        container_color.unwrap_or_else(|| AppBarDefaults::container_color(&scheme));
    let content_color = content_color.unwrap_or_else(|| AppBarDefaults::title_color(&scheme));
    let elevation = elevation.unwrap_or(AppBarDefaults::TOP_APP_BAR_ELEVATION);
    let content_padding = content_padding.unwrap_or(AppBarDefaults::CONTENT_PADDING);
    let main_axis_alignment = main_axis_alignment.unwrap_or(MainAxisAlignment::Start);
    let cross_axis_alignment = cross_axis_alignment.unwrap_or(CrossAxisAlignment::Center);
    let content = content.unwrap_or_else(RenderSlot::empty);

    surface()
        .style(container_color.into())
        .content_color(content_color)
        .content_alignment(Alignment::CenterStart)
        .elevation(elevation)
        .tonal_elevation(elevation)
        .modifier(
            modifier
                .fill_max_width()
                .height(AppBarDefaults::TOP_APP_BAR_HEIGHT),
        )
        .child(move || {
            let content = content;
            row()
                .modifier(Modifier::new().fill_max_size().padding(content_padding))
                .main_axis_alignment(main_axis_alignment)
                .cross_axis_alignment(cross_axis_alignment)
                .children_shared(content);
        });
}

impl TopAppBarBuilder {
    /// Append a trailing action item.
    pub fn action<F>(mut self, action: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.props
            .actions
            .get_or_insert_with(Vec::new)
            .push(RenderSlot::new(action));
        self
    }

    /// Append a trailing action item using a shared callback.
    pub fn action_shared(mut self, action: impl Into<RenderSlot>) -> Self {
        self.props
            .actions
            .get_or_insert_with(Vec::new)
            .push(action.into());
        self
    }

    /// Replace the trailing actions with shared callbacks.
    pub fn actions_shared(mut self, actions: Vec<RenderSlot>) -> Self {
        self.props.actions = Some(actions);
        self
    }

    /// Appends a desktop minimize window control button.
    pub fn window_control_minimize(self) -> Self {
        self.action(|| {
            icon_button()
                .icon(filled::MINIMIZE_SVG)
                .on_click(tessera_platform::window::minimize);
        })
    }

    /// Appends a desktop maximize or restore window control button.
    pub fn window_control_toggle_maximize(self) -> Self {
        self.action(|| {
            icon_button()
                .icon(filled::FULLSCREEN_SVG)
                .on_click(tessera_platform::window::toggle_maximize);
        })
    }

    /// Appends a desktop close window control button.
    pub fn window_control_close(self) -> Self {
        self.action(|| {
            icon_button()
                .icon(filled::CLOSE_SVG)
                .on_click(tessera_platform::window::close);
        })
    }
}

/// # top_app_bar
///
/// Render a standard top app bar with title, navigation icon, and actions.
///
/// ## Usage
///
/// Use as the primary top bar to show the current screen title and actions.
///
/// ## Parameters
///
/// - `modifier` — modifier chain applied to the surface.
/// - `container_color` — optional app bar surface color.
/// - `content_color` — optional default title color.
/// - `elevation` — optional surface elevation.
/// - `content_padding` — optional padding applied to app bar content.
/// - `title` — title text displayed in the bar.
/// - `title_area_modifier` — modifier chain applied to the weighted title area.
/// - `navigation_icon` — optional leading icon slot.
/// - `actions` — trailing action slots.
/// - `navigation_icon_color` — optional color for the navigation icon slot.
/// - `action_icon_color` — optional color for trailing action slots.
/// - `title_inset` — optional total title inset when no navigation icon is
///   present.
/// - `actions_spacing` — optional spacing inserted between action items.
///
/// ## Examples
/// ```rust
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::{app_bar::top_app_bar, text::text};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// top_app_bar().title("Inbox").action(|| {
///     text().content("Edit");
/// });
/// #     });
/// # }
/// # component();
/// ```
#[tessera]
pub fn top_app_bar(
    modifier: Option<Modifier>,
    container_color: Option<Color>,
    content_color: Option<Color>,
    elevation: Option<Dp>,
    content_padding: Option<Padding>,
    #[prop(into)] title: Option<String>,
    title_area_modifier: Option<Modifier>,
    navigation_icon: Option<RenderSlot>,
    #[prop(skip_setter)] actions: Option<Vec<RenderSlot>>,
    navigation_icon_color: Option<Color>,
    action_icon_color: Option<Color>,
    title_inset: Option<Dp>,
    actions_spacing: Option<Dp>,
) {
    let modifier = modifier.unwrap_or_default();
    let title = title.unwrap_or_default();
    let title_area_modifier = title_area_modifier.unwrap_or_default();
    let actions = actions.unwrap_or_default();
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let scheme = theme.color_scheme;
    let typography = theme.typography;

    let container_color =
        container_color.unwrap_or_else(|| AppBarDefaults::container_color(&scheme));
    let content_color = content_color.unwrap_or_else(|| AppBarDefaults::title_color(&scheme));
    let content_padding = content_padding.unwrap_or(AppBarDefaults::TOP_APP_BAR_CONTENT_PADDING);
    let navigation_icon_color =
        navigation_icon_color.unwrap_or_else(|| AppBarDefaults::navigation_icon_color(&scheme));
    let action_icon_color =
        action_icon_color.unwrap_or_else(|| AppBarDefaults::action_icon_color(&scheme));
    let title_inset = title_inset.unwrap_or(AppBarDefaults::TITLE_INSET);
    let actions_spacing = actions_spacing.unwrap_or(AppBarDefaults::ACTIONS_SPACING);

    let start_padding = content_padding.left;
    let extra_inset = Dp((title_inset.0 - start_padding.0).max(0.0));
    let title_style = typography.title_large;

    app_bar()
        .modifier(modifier)
        .container_color(container_color)
        .content_color(content_color)
        .elevation(elevation.unwrap_or(AppBarDefaults::TOP_APP_BAR_ELEVATION))
        .content_padding(content_padding)
        .content(move || {
            let navigation_icon = navigation_icon;
            let actions = actions.clone();
            let title_text = title.clone();
            let title_mod = title_area_modifier.clone();
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    let navigation_icon = navigation_icon;
                    let actions = actions.clone();
                    if let Some(navigation_icon) = navigation_icon {
                        let nav_color = navigation_icon_color;
                        provide_context(
                            || ContentColor { current: nav_color },
                            || {
                                navigation_icon.render();
                            },
                        );
                    } else if extra_inset.0 > 0.0 {
                        let spacer_width = extra_inset;
                        spacer().modifier(Modifier::new().width(spacer_width));
                    }

                    let title_text = title_text.clone();
                    let title_mod = title_mod.clone();
                    row()
                        .modifier(title_mod.fill_max_size().weight(1.0))
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children(move || {
                            if title_text.is_empty() {
                                spacer().modifier(Modifier::new().fill_max_width());
                            } else {
                                let text_value = title_text.clone();
                                provide_text_style(title_style, move || {
                                    text().content(text_value.clone());
                                });
                            }
                        });

                    if !actions.is_empty() {
                        let actions_len = actions.len();
                        let spacing = actions_spacing;
                        let action_color = action_icon_color;
                        let actions = actions.clone();
                        provide_context(
                            || ContentColor {
                                current: action_color,
                            },
                            || {
                                row()
                                    .cross_axis_alignment(CrossAxisAlignment::Center)
                                    .children(move || {
                                        for (index, action) in actions.iter().cloned().enumerate() {
                                            action.render();
                                            if spacing.0 > 0.0 && index + 1 < actions_len {
                                                let spacer_width = spacing;
                                                spacer()
                                                    .modifier(Modifier::new().width(spacer_width));
                                            }
                                        }
                                    });
                            },
                        );
                    }
                });
        });
}
