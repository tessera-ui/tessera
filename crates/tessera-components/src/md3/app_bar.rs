//! Top app bars for app-level navigation and actions.
//!
//! ## Usage
//!
//! Show screen titles, navigation, and primary actions above page content.
use tessera_foundation::{
    gesture::DragAxis,
    modifier::{DraggableArgs, ModifierExt as _, Padding, SemanticsArgs},
};
use tessera_ui::{
    Asset, AxisConstraint, Callback, CallbackWith, Color, ComputedData, Constraint, Dp,
    LayoutResult, MeasurementError, Modifier, Px, PxPosition, RenderSlot, State,
    layout::MeasureScope, provide_context, remember, renderer::AppTitleBar, tessera, use_context,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    column::column,
    md3::{
        icon_button::icon_button,
        material_icons::filled,
        progress::cubic_bezier_easing,
        surface::surface,
        text::text,
        theme::{ContentColor, MaterialColorScheme, MaterialTheme},
    },
    res::I18N,
    row::row,
    spacer::spacer,
};

pub use scroll::{AppBarScrollBehavior, AppBarScrollController, app_bar_scroll_provider};

mod scroll;

/// Layout and typography of a top app bar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TopAppBarVariant {
    /// A single 64dp row with a title and optional subtitle.
    #[default]
    Small,
    /// A 112dp bar, or 136dp with a subtitle, with headline typography.
    MediumFlexible,
    /// A 120dp bar, or 152dp with a subtitle, with display typography.
    LargeFlexible,
}

/// Horizontal title placement within a top app bar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AppBarTitleAlignment {
    /// Align the title to the leading content inset.
    #[default]
    Start,
    /// Center relative to the complete bar, avoiding navigation and actions.
    Center,
}

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
    pub const TOP_APP_BAR_CONTENT_PADDING: Padding = Self::CONTENT_PADDING;

    /// Expanded medium height without a subtitle.
    pub const MEDIUM_FLEXIBLE_HEIGHT: Dp = Dp(112.0);
    /// Expanded medium height with a subtitle.
    pub const MEDIUM_FLEXIBLE_SUBTITLE_HEIGHT: Dp = Dp(136.0);
    /// Expanded large height without a subtitle.
    pub const LARGE_FLEXIBLE_HEIGHT: Dp = Dp(120.0);
    /// Expanded large height with a subtitle.
    pub const LARGE_FLEXIBLE_SUBTITLE_HEIGHT: Dp = Dp(152.0);

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
/// use tessera_components::md3::{app_bar::app_bar, text::text};
/// # use tessera_components::md3::theme::{MaterialTheme, material_theme};
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// app_bar().content(|| {
///     text("Inbox");
/// });
/// #     });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
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
    pub fn action<F, Output>(&self, action: F) -> Self
    where
        F: Fn() -> Output + 'static,
    {
        self.props.append("actions", move |props| {
            props
                .actions
                .get_or_insert_with(Vec::new)
                .push(RenderSlot::new(action))
        });
        Self { props: self.props }
    }

    /// Append a trailing action item using a shared callback.
    pub fn action_shared(&self, action: impl Into<RenderSlot> + 'static) -> Self {
        self.props.append("actions", move |props| {
            props
                .actions
                .get_or_insert_with(Vec::new)
                .push(action.into())
        });
        Self { props: self.props }
    }

    /// Replace the trailing actions with shared callbacks.
    pub fn actions_shared(&self, actions: Vec<RenderSlot>) -> Self {
        let value = Some(actions);
        self.props
            .configure("actions", move |props| props.actions = value);
        Self { props: self.props }
    }

    /// Appends a desktop minimize window control button.
    pub fn window_control_minimize(&self) -> Self {
        self.action(|| window_control(WindowControl::Minimize))
    }

    /// Appends a desktop maximize or restore window control button.
    pub fn window_control_toggle_maximize(&self) -> Self {
        self.action(|| window_control(WindowControl::Maximize))
    }

    /// Appends a desktop close window control button.
    pub fn window_control_close(&self) -> Self {
        self.action(|| window_control(WindowControl::Close))
    }
}

#[derive(Clone, Copy, PartialEq)]
enum WindowControl {
    Minimize,
    Maximize,
    Close,
}

#[tessera]
fn window_control(control: WindowControl) {
    let (label, icon, action): (&str, Asset, fn()) = match control {
        WindowControl::Minimize => (
            "md3-minimize-window",
            filled::MINIMIZE_SVG,
            tessera_platform::window::minimize,
        ),
        WindowControl::Maximize => (
            "md3-toggle-maximize-window",
            filled::FULLSCREEN_SVG,
            tessera_platform::window::toggle_maximize,
        ),
        WindowControl::Close => (
            "md3-close-window",
            filled::CLOSE_SVG,
            tessera_platform::window::close,
        ),
    };
    icon_button()
        .standard()
        .accessibility_label(I18N.text(label))
        .icon(icon)
        .on_click(action);
}

/// # top_app_bar
///
/// Render a small or flexible top app bar for screen titles, navigation, and
/// actions.
///
/// ## Usage
///
/// Use as the primary top bar to show the current screen title and actions.
/// Flexible variants grow to accommodate multiline titles and subtitles.
///
/// ## Parameters
///
/// - `modifier` — modifier chain applied to the surface.
/// - `container_color` — optional app bar surface color.
/// - `content_color` — optional default title color.
/// - `elevation` — optional surface elevation.
/// - `content_padding` — optional padding applied to app bar content.
/// - `title` — title text displayed in the bar.
/// - `title_content` — optional image, logo or custom title content replacing
///   the title text while retaining the subtitle and alignment.
/// - `title_area_modifier` — modifier chain applied to the title and subtitle
///   area.
/// - `window_chrome` — use this bar as the title bar when the window is
///   configured with `WindowConfig::with_app_titlebar`.
/// - `navigation_icon` — optional leading icon slot.
/// - `actions` — trailing action slots.
/// - `navigation_icon_color` — optional color for the navigation icon slot.
/// - `action_icon_color` — optional color for trailing action slots.
/// - `title_inset` — optional total title inset when no navigation icon is
///   present.
/// - `actions_spacing` — optional spacing inserted between action items.
/// - `variant` — small, medium flexible, or large flexible layout.
/// - `subtitle` — optional secondary text below the title.
/// - `title_alignment` — leading or globally centered title placement.
/// - `scroll_controller` — shared height and overlap state also passed to the scroll
///   provider.
/// - `scrolled_container_color` — optional container color when content scrolls
///   under the bar.
///
/// ## Examples
///
/// ```rust
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::md3::{
///     app_bar::{TopAppBarVariant, top_app_bar},
///     button::button,
///     text::text,
/// };
/// use tessera_ui::{Callback, remember};
/// # use tessera_components::md3::theme::{MaterialTheme, material_theme};
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// let unread = remember(|| 3usize);
/// let mark_read = Callback::new(move || unread.set(0));
/// top_app_bar()
///     .variant(TopAppBarVariant::MediumFlexible)
///     .title("Inbox")
///     .subtitle(format!("{} unread messages", unread.get()))
///     .action(move || {
///         button().text().on_click_shared(mark_read).child(|| {
///             text("Mark read");
///         });
///     });
/// mark_read.call();
/// assert_eq!(unread.get(), 0);
/// #     });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn top_app_bar(
    modifier: Option<Modifier>,
    container_color: Option<Color>,
    content_color: Option<Color>,
    elevation: Option<Dp>,
    content_padding: Option<Padding>,
    #[prop(into)] title: Option<String>,
    title_content: Option<RenderSlot>,
    title_area_modifier: Option<Modifier>,
    window_chrome: Option<bool>,
    navigation_icon: Option<RenderSlot>,
    #[prop(skip_setter)] actions: Option<Vec<RenderSlot>>,
    navigation_icon_color: Option<Color>,
    action_icon_color: Option<Color>,
    title_inset: Option<Dp>,
    actions_spacing: Option<Dp>,
    variant: Option<TopAppBarVariant>,
    #[prop(into)] subtitle: Option<String>,
    title_alignment: Option<AppBarTitleAlignment>,
    scroll_controller: Option<State<AppBarScrollController>>,
    scrolled_container_color: Option<Color>,
) {
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let scheme = theme.color_scheme;
    let variant = variant.unwrap_or_default();
    let alignment = title_alignment.unwrap_or_default();
    let has_navigation = navigation_icon.is_some();
    let title_bar = if window_chrome.unwrap_or(false) {
        AppTitleBar::current()
    } else {
        AppTitleBar::None
    };
    let mut actions = actions.unwrap_or_default();
    if title_bar == AppTitleBar::Custom {
        for control in [
            WindowControl::Minimize,
            WindowControl::Maximize,
            WindowControl::Close,
        ] {
            actions.push(RenderSlot::new(move || window_control(control)));
        }
    }
    let has_actions = !actions.is_empty();
    let title = title.unwrap_or_default();
    let subtitle = subtitle.filter(|value| !value.is_empty());
    let has_subtitle = subtitle.is_some();
    let title_color = content_color.unwrap_or_else(|| AppBarDefaults::title_color(&scheme));
    let nav_color =
        navigation_icon_color.unwrap_or_else(|| AppBarDefaults::navigation_icon_color(&scheme));
    let action_color =
        action_icon_color.unwrap_or_else(|| AppBarDefaults::action_icon_color(&scheme));
    let (title_style, subtitle_style) = match variant {
        TopAppBarVariant::Small => (theme.typography.title_large, theme.typography.label_medium),
        TopAppBarVariant::MediumFlexible => (
            theme.typography.headline_medium,
            theme.typography.label_large,
        ),
        TopAppBarVariant::LargeFlexible => (
            theme.typography.display_small,
            theme.typography.title_medium,
        ),
    };
    let policy = TopAppBarLayout {
        scroll_controller,
        variant,
        alignment,
        has_navigation,
        has_actions,
        has_subtitle,
        padding: content_padding.unwrap_or(AppBarDefaults::TOP_APP_BAR_CONTENT_PADDING),
        title_inset: title_inset.unwrap_or(AppBarDefaults::TITLE_INSET),
    };
    let title_area_modifier = title_area_modifier.unwrap_or_default();
    let title_area_modifier = if title_bar == AppTitleBar::None {
        title_area_modifier
    } else {
        title_area_modifier.window_drag_region()
    };
    let actions_spacing = actions_spacing.unwrap_or(AppBarDefaults::ACTIONS_SPACING);
    let elevation = elevation.unwrap_or(AppBarDefaults::TOP_APP_BAR_ELEVATION);
    let fraction = scroll_controller
        .map(|state| state.with(AppBarScrollController::collapsed_fraction))
        .unwrap_or(0.0);
    let color_fraction = scroll_controller
        .map(|state| state.with(AppBarScrollController::color_fraction))
        .unwrap_or(0.0);
    let container = container_color
        .unwrap_or_else(|| AppBarDefaults::container_color(&scheme))
        .lerp(
            &scrolled_container_color.unwrap_or(scheme.surface_container),
            color_fraction,
        );
    let modifier = modifier.unwrap_or_default();
    let modifier = if let Some(state) = scroll_controller {
        let active = remember(|| state);
        if active.get() != state {
            active.set(state);
        }
        modifier.draggable(DraggableArgs {
            enabled: state.with(AppBarScrollController::drag_enabled),
            axis: Some(DragAxis::Vertical),
            on_drag_delta: CallbackWith::new(
                move |delta: tessera_foundation::modifier::DragDelta| {
                    active
                        .get()
                        .with_mut(|value| value.drag_by(delta.y.to_f32()))
                },
            ),
            on_drag_stopped: Some(Callback::new(move || {
                active.get().with_mut(AppBarScrollController::end_drag)
            })),
            ..Default::default()
        })
    } else {
        modifier
    };
    surface()
        .style(container.into())
        .content_color(title_color)
        .elevation(elevation)
        .tonal_elevation(elevation)
        .modifier(
            modifier
                .fill_max_width()
                .clip_to_bounds()
                .semantics(SemanticsArgs {
                    role: Some(tessera_ui::accesskit::Role::Toolbar),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .child(move || {
            let title = title.clone();
            let subtitle = subtitle.clone();
            let title_area_modifier = title_area_modifier.clone();
            let actions = actions.clone();
            tessera_ui::layout::layout(policy.clone()).child(move || {
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(move || {
                        if let Some(navigation_icon) = navigation_icon {
                            provide_context(ContentColor { current: nav_color }, move || {
                                navigation_icon.render();
                            });
                        }
                    });
                let title = title.clone();
                let subtitle = subtitle.clone();
                let title_area_modifier = if alignment == AppBarTitleAlignment::Start {
                    title_area_modifier.clone().fill_max_width()
                } else {
                    title_area_modifier.clone()
                };
                let compact_title = title.clone();
                let compact_subtitle = subtitle.clone();
                column()
                    .modifier(title_layer_modifier(
                        title_area_modifier.clone(),
                        if variant == TopAppBarVariant::Small {
                            1.0
                        } else {
                            cubic_bezier_easing(fraction, 0.8, 0.0, 0.8, 0.15)
                        },
                        variant == TopAppBarVariant::Small || fraction >= 0.5,
                    ))
                    .main_axis_alignment(MainAxisAlignment::Center)
                    .cross_axis_alignment(if alignment == AppBarTitleAlignment::Center {
                        CrossAxisAlignment::Center
                    } else {
                        CrossAxisAlignment::Start
                    })
                    .children(move || {
                        if let Some(content) = title_content {
                            content.render();
                        } else if !compact_title.is_empty() {
                            text(compact_title.clone())
                                .single_line(true)
                                .style(theme.typography.title_large)
                                .color(title_color);
                        }
                        if let Some(subtitle) = &compact_subtitle {
                            text(subtitle.clone())
                                .single_line(true)
                                .style(theme.typography.label_medium)
                                .color(scheme.on_surface_variant);
                        }
                    });
                let actions = actions.clone();
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(move || {
                        let actions = actions.clone();
                        provide_context(
                            ContentColor {
                                current: action_color,
                            },
                            move || {
                                for (index, action) in actions.iter().enumerate() {
                                    if index > 0 && actions_spacing.0 > 0.0 {
                                        spacer().modifier(Modifier::new().width(actions_spacing));
                                    }
                                    action.render();
                                }
                            },
                        );
                    });
                if variant != TopAppBarVariant::Small {
                    let title = title.clone();
                    let subtitle = subtitle.clone();
                    column()
                        .modifier(title_layer_modifier(
                            title_area_modifier.clone(),
                            1.0 - fraction,
                            fraction < 0.5,
                        ))
                        .cross_axis_alignment(if alignment == AppBarTitleAlignment::Center {
                            CrossAxisAlignment::Center
                        } else {
                            CrossAxisAlignment::Start
                        })
                        .children(move || {
                            if let Some(content) = title_content {
                                content.render();
                            } else if !title.is_empty() {
                                text(title.clone()).style(title_style).color(title_color);
                            }
                            if let Some(subtitle) = &subtitle {
                                text(subtitle.clone())
                                    .style(subtitle_style)
                                    .color(scheme.on_surface_variant);
                            }
                        });
                }
            });
        });
}

fn title_layer_modifier(modifier: Modifier, alpha: f32, semantics: bool) -> Modifier {
    let modifier = modifier.clip_to_bounds().alpha(alpha);
    if semantics {
        modifier
    } else {
        modifier.semantics(SemanticsArgs {
            hidden: true,
            merge_descendants: false,
            ..Default::default()
        })
    }
}

#[derive(Clone, PartialEq)]
struct TopAppBarLayout {
    scroll_controller: Option<State<AppBarScrollController>>,
    variant: TopAppBarVariant,
    alignment: AppBarTitleAlignment,
    has_navigation: bool,
    has_actions: bool,
    has_subtitle: bool,
    padding: Padding,
    title_inset: Dp,
}

impl tessera_ui::LayoutPolicy for TopAppBarLayout {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = *scope.parent_constraint().as_ref();
        let width = parent.width.max.unwrap_or(parent.width.min);
        let top = self.padding.top.to_px();
        let bottom = self.padding.bottom.to_px();
        let left = self.padding.left.to_px().min(width);
        let right = self.padding.right.to_px().min((width - left).max(Px::ZERO));
        let available = (width - left - right).max(Px::ZERO);
        let toolbar_height = (parent.height - top - bottom)
            .clamp(AppBarDefaults::TOP_APP_BAR_HEIGHT.to_px())
            .min(AppBarDefaults::TOP_APP_BAR_HEIGHT.to_px());
        let children = scope.children();
        let actions = children[2].measure(
            scope,
            &Constraint::new(
                AxisConstraint::at_most(available),
                AxisConstraint::at_most(toolbar_height),
            ),
        )?;
        let navigation = children[0].measure(
            scope,
            &Constraint::new(
                AxisConstraint::at_most((available - actions.width).max(Px::ZERO)),
                AxisConstraint::at_most(toolbar_height),
            ),
        )?;
        let expanded = self.variant != TopAppBarVariant::Small;
        let inset = self.title_inset.to_px().max(left).min(width);
        let title_start = if !self.has_navigation {
            inset
        } else {
            (left + navigation.width + Dp(4.0).to_px()).min(width)
        };
        let title_end = if !self.has_actions {
            (width - self.title_inset.to_px().max(right)).max(title_start)
        } else {
            (width - right - actions.width - Dp(4.0).to_px()).max(title_start)
        }
        .min(width);
        let title_width = (title_end - title_start).max(Px::ZERO);
        let title_height = if self.alignment == AppBarTitleAlignment::Start {
            AxisConstraint::exact(toolbar_height)
        } else {
            AxisConstraint::at_most(toolbar_height)
        };
        let title = children[1].measure(
            scope,
            &Constraint::new(AxisConstraint::at_most(title_width), title_height),
        )?;
        let preferred = match (self.variant, self.has_subtitle) {
            (TopAppBarVariant::Small, _) => AppBarDefaults::TOP_APP_BAR_HEIGHT,
            (TopAppBarVariant::MediumFlexible, false) => AppBarDefaults::MEDIUM_FLEXIBLE_HEIGHT,
            (TopAppBarVariant::MediumFlexible, true) => {
                AppBarDefaults::MEDIUM_FLEXIBLE_SUBTITLE_HEIGHT
            }
            (TopAppBarVariant::LargeFlexible, false) => AppBarDefaults::LARGE_FLEXIBLE_HEIGHT,
            (TopAppBarVariant::LargeFlexible, true) => {
                AppBarDefaults::LARGE_FLEXIBLE_SUBTITLE_HEIGHT
            }
        }
        .to_px();
        let title_bottom_padding = match (self.variant, self.has_subtitle) {
            (TopAppBarVariant::Small, _) => Dp::ZERO,
            (_, false) => Dp(12.0),
            (TopAppBarVariant::MediumFlexible, true) => Dp(16.0),
            (TopAppBarVariant::LargeFlexible, true) => Dp(20.0),
        }
        .to_px();
        let large_title = if expanded {
            Some(children[3].measure(
                scope,
                &Constraint::new(
                    AxisConstraint::at_most(
                        (width - inset - self.title_inset.to_px().max(right)).max(Px::ZERO),
                    ),
                    parent.height.without_min() - top - bottom - toolbar_height,
                ),
            )?)
        } else {
            None
        };
        let content_height = large_title
            .as_ref()
            .map(|title| preferred.max(toolbar_height + title.height + title_bottom_padding))
            .unwrap_or(preferred);
        let compact_height = if !expanded
            && self.scroll_controller.is_some_and(|state| {
                state.with(AppBarScrollController::behavior) == AppBarScrollBehavior::EnterAlways
            }) {
            Px::ZERO
        } else {
            toolbar_height
        };
        let offset = if let Some(state) = self.scroll_controller {
            let range =
                if state.with(AppBarScrollController::behavior) == AppBarScrollBehavior::Pinned {
                    Dp::ZERO
                } else {
                    Dp::from(content_height - compact_height)
                };
            if state.with(AppBarScrollController::height_range) != range {
                state.with_mut(|value| {
                    value.configure_height(Dp::from(content_height), Dp::from(compact_height))
                });
            }
            state.with(AppBarScrollController::height_offset).to_px()
        } else {
            Px::ZERO
        };
        let height = parent
            .height
            .clamp((content_height + offset + top + bottom).max(Px::ZERO));
        let toolbar_height = if !expanded {
            (height - top - bottom).max(Px::ZERO).min(toolbar_height)
        } else {
            toolbar_height
        };
        let mut result = LayoutResult::new(ComputedData { width, height });
        let toolbar_visible = toolbar_height > Px::ZERO && height > top + bottom;
        if toolbar_visible {
            result.place_child(
                children[0],
                PxPosition::new(left, top + (toolbar_height - navigation.height) / 2),
            );
        }
        let title_x = if self.alignment == AppBarTitleAlignment::Center {
            ((width - title.width) / 2)
                .max(title_start)
                .min((title_end - title.width).max(title_start))
        } else {
            title_start
        };
        let title_y = top + (toolbar_height - title.height) / 2;
        let fraction = self
            .scroll_controller
            .map(|state| state.with(AppBarScrollController::collapsed_fraction))
            .unwrap_or(0.0);
        if toolbar_visible
            && title_width > Px::ZERO
            && title_y < height
            && (!expanded || fraction > 0.0)
        {
            result.place_child(children[1], PxPosition::new(title_x, title_y));
        }
        if let Some(title) = large_title
            && height > top + toolbar_height
            && fraction < 1.0
        {
            let x = if self.alignment == AppBarTitleAlignment::Center {
                (width - title.width) / 2
            } else {
                inset
            };
            result.place_child(children[3], PxPosition::new(x, top + toolbar_height));
        }
        if toolbar_visible {
            result.place_child(
                children[2],
                PxPosition::new(
                    width - right - actions.width,
                    top + (toolbar_height - actions.height) / 2,
                ),
            );
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod scroll_tests;
