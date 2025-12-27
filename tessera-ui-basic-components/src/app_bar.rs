//! Top app bars for app-level navigation and actions.
//!
//! ## Usage
//!
//! Use for screen titles, navigation affordances, and primary actions at the
//! top of a view.
use std::sync::Arc;

use derive_setters::Setters;
use tessera_ui::{Color, Dp, Modifier, provide_context, tessera, use_context};

use crate::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    modifier::{ModifierExt as _, Padding},
    row::{RowArgs, RowScope, row},
    spacer::spacer,
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
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

/// Configuration arguments for [`app_bar`].
#[derive(Clone, Setters)]
pub struct AppBarArgs {
    /// Modifier chain applied to the app bar container.
    pub modifier: Modifier,
    /// Container color for the app bar surface.
    pub container_color: Color,
    /// Default content color propagated to descendants.
    pub content_color: Color,
    /// Elevation applied to the app bar surface.
    pub elevation: Dp,
    /// Padding applied to the app bar content row.
    pub content_padding: Padding,
    /// Main axis alignment for the content row.
    pub main_axis_alignment: MainAxisAlignment,
    /// Cross axis alignment for the content row.
    pub cross_axis_alignment: CrossAxisAlignment,
}

impl Default for AppBarArgs {
    fn default() -> Self {
        let scheme = use_context::<MaterialTheme>().get().color_scheme;
        Self {
            modifier: Modifier::new()
                .fill_max_width()
                .height(AppBarDefaults::TOP_APP_BAR_HEIGHT),
            container_color: AppBarDefaults::container_color(&scheme),
            content_color: AppBarDefaults::title_color(&scheme),
            elevation: AppBarDefaults::TOP_APP_BAR_ELEVATION,
            content_padding: AppBarDefaults::CONTENT_PADDING,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Center,
        }
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
/// - `args` — configures container appearance and layout; see [`AppBarArgs`].
/// - `content` — closure that receives a [`RowScope`] to build the bar content.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_ui_basic_components::{
///     app_bar::{AppBarArgs, AppBarDefaults, app_bar},
///     text::{TextArgs, text},
/// };
///
/// let args = AppBarArgs::default();
/// assert_eq!(
///     args.content_padding.left,
///     AppBarDefaults::HORIZONTAL_PADDING
/// );
///
/// app_bar(args, |scope| {
///     scope.child(|| text(TextArgs::default().text("Inbox")));
/// });
/// # }
/// # component();
/// ```
#[tessera]
pub fn app_bar<F>(args: impl Into<AppBarArgs>, content: F)
where
    F: FnOnce(&mut RowScope) + Send + Sync + 'static,
{
    let args: AppBarArgs = args.into();

    surface(
        SurfaceArgs::default()
            .style(args.container_color.into())
            .content_color(args.content_color)
            .content_alignment(Alignment::CenterStart)
            .elevation(args.elevation)
            .tonal_elevation(args.elevation)
            .modifier(args.modifier),
        move || {
            row(
                RowArgs::default()
                    .modifier(
                        Modifier::new()
                            .fill_max_size()
                            .padding(args.content_padding),
                    )
                    .main_axis_alignment(args.main_axis_alignment)
                    .cross_axis_alignment(args.cross_axis_alignment),
                content,
            );
        },
    );
}

/// Configuration arguments for [`top_app_bar`].
#[derive(Clone, Setters)]
pub struct TopAppBarArgs {
    /// Base container configuration for the app bar; see [`AppBarArgs`].
    pub app_bar: AppBarArgs,
    /// Title text displayed in the bar.
    #[setters(into)]
    pub title: String,
    /// Modifier chain applied to the title text.
    pub title_modifier: Modifier,
    /// Optional navigation icon rendered at the leading edge.
    #[setters(skip)]
    pub navigation_icon: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Actions rendered at the trailing edge.
    #[setters(skip)]
    pub actions: Vec<Arc<dyn Fn() + Send + Sync>>,
    /// Color applied to the navigation icon slot.
    pub navigation_icon_color: Color,
    /// Color applied to the trailing action slot.
    pub action_icon_color: Color,
    /// Total start inset applied to the title when no navigation icon is
    /// present.
    pub title_inset: Dp,
    /// Horizontal spacing inserted between action items.
    pub actions_spacing: Dp,
}

impl Default for TopAppBarArgs {
    fn default() -> Self {
        let scheme = use_context::<MaterialTheme>().get().color_scheme;
        Self {
            app_bar: AppBarArgs::default(),
            title: String::new(),
            title_modifier: Modifier::new(),
            navigation_icon: None,
            actions: Vec::new(),
            navigation_icon_color: AppBarDefaults::navigation_icon_color(&scheme),
            action_icon_color: AppBarDefaults::action_icon_color(&scheme),
            title_inset: AppBarDefaults::TITLE_INSET,
            actions_spacing: AppBarDefaults::ACTIONS_SPACING,
        }
    }
}

impl TopAppBarArgs {
    /// Create args with the required title text.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Self::default()
        }
    }

    /// Set the navigation icon slot content.
    pub fn navigation_icon<F>(mut self, navigation_icon: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.navigation_icon = Some(Arc::new(navigation_icon));
        self
    }

    /// Set the navigation icon slot content using a shared callback.
    pub fn navigation_icon_shared(mut self, navigation_icon: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.navigation_icon = Some(navigation_icon);
        self
    }

    /// Append a trailing action item.
    pub fn action<F>(mut self, action: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.actions.push(Arc::new(action));
        self
    }

    /// Append a trailing action item using a shared callback.
    pub fn action_shared(mut self, action: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.actions.push(action);
        self
    }

    /// Replace the trailing actions with shared callbacks.
    pub fn actions_shared(mut self, actions: Vec<Arc<dyn Fn() + Send + Sync>>) -> Self {
        self.actions = actions;
        self
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
/// - `args` — configures content, colors, and layout; see [`TopAppBarArgs`].
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_ui_basic_components::{
///     app_bar::{TopAppBarArgs, top_app_bar},
///     text::{TextArgs, text},
/// };
///
/// let args = TopAppBarArgs::new("Inbox").action(|| {
///     text(TextArgs::default().text("Edit"));
/// });
/// assert_eq!(args.title, "Inbox");
///
/// top_app_bar(args);
/// # }
/// # component();
/// ```
#[tessera]
pub fn top_app_bar(args: impl Into<TopAppBarArgs>) {
    let args: TopAppBarArgs = args.into();
    let typography = use_context::<MaterialTheme>().get().typography;

    let TopAppBarArgs {
        app_bar: app_bar_args,
        title,
        title_modifier,
        navigation_icon,
        actions,
        navigation_icon_color,
        action_icon_color,
        title_inset,
        actions_spacing,
    } = args;

    let start_padding = app_bar_args.content_padding.left;
    let extra_inset = Dp((title_inset.0 - start_padding.0).max(0.0));
    let title_style = typography.title_large;

    app_bar(app_bar_args, move |scope| {
        if let Some(navigation_icon) = navigation_icon {
            let nav_color = navigation_icon_color;
            scope.child(move || {
                provide_context(ContentColor { current: nav_color }, || {
                    navigation_icon();
                });
            });
        } else if extra_inset.0 > 0.0 {
            let spacer_width = extra_inset;
            scope.child(move || {
                spacer(Modifier::new().width(spacer_width));
            });
        }

        let title_text = title;
        scope.child_weighted(
            move || {
                if title_text.is_empty() {
                    spacer(Modifier::new().fill_max_width());
                } else {
                    provide_text_style(title_style, move || {
                        text(
                            TextArgs::default()
                                .text(title_text)
                                .modifier(title_modifier),
                        );
                    });
                }
            },
            1.0,
        );

        if !actions.is_empty() {
            let actions_len = actions.len();
            let spacing = actions_spacing;
            let action_color = action_icon_color;
            scope.child(move || {
                provide_context(
                    ContentColor {
                        current: action_color,
                    },
                    || {
                        row(
                            RowArgs::default().cross_axis_alignment(CrossAxisAlignment::Center),
                            move |row_scope| {
                                for (index, action) in actions.into_iter().enumerate() {
                                    row_scope.child(move || {
                                        action();
                                    });
                                    if spacing.0 > 0.0 && index + 1 < actions_len {
                                        let spacer_width = spacing;
                                        row_scope.child(move || {
                                            spacer(Modifier::new().width(spacer_width));
                                        });
                                    }
                                }
                            },
                        );
                    },
                );
            });
        }
    });
}
