//! A component for creating a tab-based layout.
//!
//! ## Usage
//!
//! Use to organize content into separate pages that can be switched between.
use std::time::Instant;

use derive_builder::Builder;
use tessera_ui::{
    Color, ComputedData, Constraint, CursorEventContent, DimensionValue, Dp, MeasurementError, Px,
    PxPosition, State, remember, tessera, use_context,
};

use crate::{
    alignment::Alignment,
    icon::{IconArgsBuilder, IconContent, icon},
    pipelines::text::{
        command::{TextCommand, TextConstraint},
        pipeline::TextData,
    },
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
    theme::{ContentColor, MaterialAlpha, MaterialColorScheme, MaterialTheme},
};

const DEFAULT_SPATIAL_DAMPING_RATIO: f32 = 0.9;
const DEFAULT_SPATIAL_STIFFNESS: f32 = 700.0;

/// Visual variants supported by [`tabs`].
#[derive(Clone, Copy, Debug, Default)]
pub enum TabsVariant {
    /// Primary tabs.
    #[default]
    Primary,
    /// Secondary tabs.
    Secondary,
}

/// Material Design 3 defaults for [`tabs`].
pub struct TabsDefaults;

impl TabsDefaults {
    /// Default indicator height.
    pub const INDICATOR_HEIGHT: Dp = Dp(3.0);
    /// Default minimum indicator width.
    pub const INDICATOR_MIN_WIDTH: Dp = Dp(24.0);
    /// Default maximum indicator width.
    pub const INDICATOR_MAX_WIDTH: Option<Dp> = None;
    /// Minimum height for a tab (Material spec uses 48dp).
    pub const MIN_TAB_HEIGHT: Dp = Dp(48.0);
    /// Default internal padding for each tab.
    pub const TAB_PADDING: Dp = Dp(16.0);
    /// Default hover alpha for state layers.
    pub const HOVER_STATE_LAYER_OPACITY: f32 = MaterialAlpha::HOVER;
    /// Default divider height for tab rows.
    pub const DIVIDER_HEIGHT: Dp = Dp(1.0);
    /// Default minimum width for a scrollable tab.
    pub const SCROLLABLE_MIN_TAB_WIDTH: Dp = Dp(90.0);
    /// Default edge padding applied to scrollable tab rows.
    pub const SCROLLABLE_EDGE_PADDING: Dp = Dp(52.0);
    /// Default height for a tab that shows both icon and text.
    pub const LARGE_TAB_HEIGHT: Dp = Dp(72.0);

    /// Default disabled content color.
    pub fn disabled_content_color(scheme: &MaterialColorScheme) -> Color {
        scheme
            .on_surface
            .with_alpha(MaterialAlpha::DISABLED_CONTENT)
    }

    /// Default ripple color derived from the selected content color.
    pub fn ripple_color(selected_content_color: Color) -> Color {
        selected_content_color
    }
}

#[derive(Clone, Copy, Debug)]
struct Spring1D {
    value: f32,
    velocity: f32,
    target: f32,
}

impl Spring1D {
    fn new(value: f32) -> Self {
        Self {
            value,
            velocity: 0.0,
            target: value,
        }
    }

    fn snap_to(&mut self, value: f32) {
        self.value = value;
        self.target = value;
        self.velocity = 0.0;
    }

    fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    fn update(&mut self, dt: f32, stiffness: f32, damping_ratio: f32) {
        let dt = dt.clamp(0.0, 0.05);
        let stiffness = stiffness.max(0.0);
        if stiffness == 0.0 {
            self.snap_to(self.target);
            return;
        }

        let damping_ratio = damping_ratio.max(0.0);
        let damping = 2.0 * damping_ratio * stiffness.sqrt();
        let displacement = self.value - self.target;
        let acceleration = -stiffness * displacement - damping * self.velocity;

        self.velocity += acceleration * dt;
        self.value += self.velocity * dt;

        if (self.value - self.target).abs() < 0.5 && self.velocity.abs() < 0.5 {
            self.snap_to(self.target);
        }
    }

    fn value_px(self) -> Px {
        Px::saturating_from_f32(self.value)
    }
}

fn clamp_wrap(min: Option<Px>, max: Option<Px>, measure: Px) -> Px {
    min.unwrap_or(Px(0))
        .max(measure)
        .min(max.unwrap_or(Px::MAX))
}

fn fill_value(min: Option<Px>, max: Option<Px>, measure: Px) -> Px {
    max.expect("Seems that you are trying to fill an infinite dimension, which is not allowed")
        .max(measure)
        .max(min.unwrap_or(Px(0)))
}

fn clamp_px(value: Px, min: Px, max: Option<Px>) -> Px {
    let clamped_max = max.unwrap_or(value);
    Px(value.0.max(min.0).min(clamped_max.0))
}

fn resolve_dimension(dim: DimensionValue, measure: Px) -> Px {
    match dim {
        DimensionValue::Fixed(v) => v,
        DimensionValue::Wrap { min, max } => clamp_wrap(min, max, measure),
        DimensionValue::Fill { min, max } => fill_value(min, max, measure),
    }
}

/// Controller for the `tabs` component.
///
/// Tracks the active tab index, previous index, animation progress and cached
/// values used to animate the indicator and content scrolling.
#[derive(Clone)]
pub struct TabsController {
    active_tab: usize,
    indicator_x: Spring1D,
    indicator_width: Spring1D,
    content_scroll_offset: Spring1D,
    tab_row_scroll_offset: Spring1D,
    tab_row_scroll_max: Px,
    tab_row_scroll_user_overridden: bool,
    tab_bar_height: Px,
    last_frame_time: Option<Instant>,
    indicator_initialized: bool,
    content_scroll_initialized: bool,
    tab_row_scroll_initialized: bool,
}

impl TabsController {
    /// Create a new state with the specified initial active tab.
    pub fn new(initial_tab: usize) -> Self {
        Self {
            active_tab: initial_tab,
            indicator_x: Spring1D::new(0.0),
            indicator_width: Spring1D::new(0.0),
            content_scroll_offset: Spring1D::new(0.0),
            tab_row_scroll_offset: Spring1D::new(0.0),
            tab_row_scroll_max: Px(0),
            tab_row_scroll_user_overridden: false,
            tab_bar_height: Px(0),
            last_frame_time: None,
            indicator_initialized: false,
            content_scroll_initialized: false,
            tab_row_scroll_initialized: false,
        }
    }

    /// Set the active tab index.
    ///
    /// If the requested index equals the current active tab this is a no-op.
    pub fn set_active_tab(&mut self, index: usize) {
        if index != self.active_tab {
            self.active_tab = index;
            self.tab_row_scroll_user_overridden = false;
        }
    }

    /// Returns the currently active tab index.
    pub fn active_tab(&self) -> usize {
        self.active_tab
    }

    fn set_tab_row_scroll_bounds(&mut self, max: Px) {
        self.tab_row_scroll_max = max.max(Px(0));
        let clamped = self
            .tab_row_scroll_offset
            .value
            .clamp(0.0, self.tab_row_scroll_max.to_f32());
        self.tab_row_scroll_offset.snap_to(clamped);
    }

    fn tab_row_scroll_max(&self) -> Px {
        self.tab_row_scroll_max
    }

    fn set_tab_bar_height(&mut self, height: Px) {
        self.tab_bar_height = height.max(Px(0));
    }

    fn tab_bar_height(&self) -> Px {
        self.tab_bar_height
    }

    fn set_tab_row_scroll_immediate(&mut self, value: Px) {
        let value = value.max(Px(0)).min(self.tab_row_scroll_max);
        self.tab_row_scroll_offset.snap_to(value.to_f32());
        self.tab_row_scroll_initialized = true;
        self.tab_row_scroll_user_overridden = true;
    }

    fn set_tab_row_scroll_target(&mut self, target: Px) {
        let target = target.max(Px(0)).min(self.tab_row_scroll_max);
        if !self.tab_row_scroll_initialized {
            self.tab_row_scroll_offset.snap_to(target.to_f32());
            self.tab_row_scroll_initialized = true;
        } else {
            self.tab_row_scroll_offset.set_target(target.to_f32());
        }
    }

    fn tab_row_scroll_px(&self) -> Px {
        self.tab_row_scroll_offset.value_px()
    }

    fn set_content_scroll_target(&mut self, target: Px) {
        if !self.content_scroll_initialized {
            self.content_scroll_offset.snap_to(target.to_f32());
            self.content_scroll_initialized = true;
        } else {
            self.content_scroll_offset.set_target(target.to_f32());
        }
    }

    fn content_scroll_px(&self) -> Px {
        self.content_scroll_offset.value_px()
    }

    fn set_indicator_targets(&mut self, width: Px, x: Px) {
        let width = width.max(Px(0)).to_f32();
        let x = x.to_f32();
        if !self.indicator_initialized {
            self.indicator_width.snap_to(width);
            self.indicator_x.snap_to(x);
            self.indicator_initialized = true;
        } else {
            self.indicator_width.set_target(width);
            self.indicator_x.set_target(x);
        }
    }

    fn indicator_width_px(&self) -> Px {
        self.indicator_width.value_px().max(Px(0))
    }

    fn indicator_x_px(&self) -> Px {
        self.indicator_x.value_px()
    }

    fn tick(&mut self, now: Instant) {
        let dt = if let Some(last) = self.last_frame_time {
            now.saturating_duration_since(last).as_secs_f32()
        } else {
            1.0 / 60.0
        };
        self.last_frame_time = Some(now);

        self.indicator_x
            .update(dt, DEFAULT_SPATIAL_STIFFNESS, DEFAULT_SPATIAL_DAMPING_RATIO);
        self.indicator_width
            .update(dt, DEFAULT_SPATIAL_STIFFNESS, DEFAULT_SPATIAL_DAMPING_RATIO);
        self.content_scroll_offset.update(
            dt,
            DEFAULT_SPATIAL_STIFFNESS,
            DEFAULT_SPATIAL_DAMPING_RATIO,
        );
        self.tab_row_scroll_offset.update(
            dt,
            DEFAULT_SPATIAL_STIFFNESS,
            DEFAULT_SPATIAL_DAMPING_RATIO,
        );
    }
}

impl Default for TabsController {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Configuration arguments for the [`tabs`] component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct TabsArgs {
    /// Visual variant for this tab row.
    #[builder(default)]
    pub variant: TabsVariant,
    /// Initial active tab index (0-based). Ignored if a controller is provided
    /// with its own state.
    #[builder(default = "0")]
    pub initial_active_tab: usize,
    /// Color of the active tab indicator.
    #[builder(default = "use_context::<MaterialTheme>().get().color_scheme.primary")]
    // Material primary tone
    pub indicator_color: Color,
    /// Background color for the tab row container.
    #[builder(default = "use_context::<MaterialTheme>().get().color_scheme.surface")]
    pub container_color: Color,
    /// Color applied to active tab titles (Material on-surface).
    #[builder(default = "use_context::<MaterialTheme>().get().color_scheme.primary")]
    pub active_content_color: Color,
    /// Color applied to inactive tab titles (Material on-surface-variant).
    #[builder(default = "use_context::<MaterialTheme>().get().color_scheme.on_surface_variant")]
    pub inactive_content_color: Color,
    /// Height of the indicator bar in density-independent pixels.
    #[builder(default = "TabsDefaults::INDICATOR_HEIGHT")]
    pub indicator_height: Dp,
    /// Minimum width for the indicator bar.
    #[builder(default = "TabsDefaults::INDICATOR_MIN_WIDTH")]
    pub indicator_min_width: Dp,
    /// Optional maximum width for the indicator bar.
    #[builder(default = "TabsDefaults::INDICATOR_MAX_WIDTH")]
    pub indicator_max_width: Option<Dp>,
    /// Minimum height for a tab (Material spec uses 48dp).
    #[builder(default = "TabsDefaults::MIN_TAB_HEIGHT")]
    pub min_tab_height: Dp,
    /// Internal padding for each tab, applied symmetrically.
    #[builder(default = "TabsDefaults::TAB_PADDING")]
    pub tab_padding: Dp,
    /// Whether the tab row is enabled for user interaction.
    ///
    /// When `false`, tabs will not react to input and will use disabled
    /// content colors.
    #[builder(default = "true")]
    pub enabled: bool,
    /// Color used for hover/pressed state layers.
    #[builder(default = "use_context::<MaterialTheme>().get().color_scheme.on_surface")]
    pub state_layer_color: Color,
    /// Opacity applied to the state layer on hover.
    #[builder(default = "TabsDefaults::HOVER_STATE_LAYER_OPACITY")]
    pub hover_state_layer_opacity: f32,
    /// Content color used when `enabled=false`.
    #[builder(
        default = "TabsDefaults::disabled_content_color(&use_context::<MaterialTheme>().get().color_scheme)"
    )]
    pub disabled_content_color: Color,
    /// Divider color drawn at the bottom of the tab bar.
    #[builder(default = "use_context::<MaterialTheme>().get().color_scheme.surface_variant")]
    pub divider_color: Color,
    /// Whether the tab row is horizontally scrollable.
    ///
    /// When enabled, each tab is measured at its intrinsic width (subject to
    /// `min_scrollable_tab_width`) and the row will scroll to keep the selected
    /// tab visible.
    #[builder(default = "false")]
    pub scrollable: bool,
    /// Edge padding for scrollable tab rows.
    #[builder(default = "TabsDefaults::SCROLLABLE_EDGE_PADDING")]
    pub edge_padding: Dp,
    /// Minimum tab width for scrollable tab rows.
    #[builder(default = "TabsDefaults::SCROLLABLE_MIN_TAB_WIDTH")]
    pub min_scrollable_tab_width: Dp,
    /// Width behavior for the entire tabs container.
    #[builder(default = "DimensionValue::FILLED")]
    pub width: DimensionValue,
    /// Height behavior for the tabs container.
    #[builder(default = "DimensionValue::Wrap { min: None, max: None }")]
    pub height: DimensionValue,
}

impl Default for TabsArgs {
    fn default() -> Self {
        TabsArgsBuilder::default()
            .build()
            .expect("builder construction failed")
    }
}

struct TabDef {
    title: TabTitle,
    content: Box<dyn FnOnce() + Send + Sync>,
}

enum TabTitle {
    Custom(Box<dyn FnOnce() + Send + Sync>),
    Themed(Box<dyn FnOnce(Color) + Send + Sync>),
    Label {
        text: String,
        icon: Option<IconContent>,
    },
}

/// Scope passed to tab configuration closures.
pub struct TabsScope<'a> {
    tabs: &'a mut Vec<TabDef>,
}

impl<'a> TabsScope<'a> {
    /// Adds a tab with its title and content builders.
    pub fn child<F1, F2>(&mut self, title: F1, content: F2)
    where
        F1: FnOnce() + Send + Sync + 'static,
        F2: FnOnce() + Send + Sync + 'static,
    {
        self.tabs.push(TabDef {
            title: TabTitle::Custom(Box::new(title)),
            content: Box::new(content),
        });
    }

    /// Adds a tab whose title closure receives the resolved content color
    /// (active/inactive).
    pub fn child_with_color<F1, F2>(&mut self, title: F1, content: F2)
    where
        F1: FnOnce(Color) + Send + Sync + 'static,
        F2: FnOnce() + Send + Sync + 'static,
    {
        self.tabs.push(TabDef {
            title: TabTitle::Themed(Box::new(title)),
            content: Box::new(content),
        });
    }

    /// Adds a tab whose title is rendered with the standard Material label
    /// layout.
    pub fn child_label<F>(&mut self, text: impl Into<String>, content: F)
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        self.tabs.push(TabDef {
            title: TabTitle::Label {
                text: text.into(),
                icon: None,
            },
            content: Box::new(content),
        });
    }

    /// Adds a tab whose title is rendered with an icon and standard Material
    /// label layout.
    pub fn child_label_with_icon<F>(
        &mut self,
        text: impl Into<String>,
        icon: impl Into<IconContent>,
        content: F,
    ) where
        F: FnOnce() + Send + Sync + 'static,
    {
        self.tabs.push(TabDef {
            title: TabTitle::Label {
                text: text.into(),
                icon: Some(icon.into()),
            },
            content: Box::new(content),
        });
    }
}

/// Arguments for [`tab_label`].
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct TabLabelArgs {
    /// Text shown in the tab.
    #[builder(setter(into))]
    pub text: String,
    /// Optional icon shown above the text.
    #[builder(default, setter(strip_option))]
    pub icon: Option<IconContent>,
    /// Horizontal padding applied to the text area.
    #[builder(default = "TabsDefaults::TAB_PADDING")]
    pub horizontal_text_padding: Dp,
    /// Height reserved for the active indicator when positioning text and icon.
    #[builder(default = "TabsDefaults::INDICATOR_HEIGHT")]
    pub indicator_height: Dp,
    /// Size of the icon, when present.
    #[builder(default = "Dp(24.0)")]
    pub icon_size: Dp,
}

impl Default for TabLabelArgs {
    fn default() -> Self {
        TabLabelArgsBuilder::default()
            .text("")
            .build()
            .expect("builder construction failed")
    }
}

/// # tab_label
///
/// Renders a standard Material tab label with optional icon and text.
///
/// ## Usage
///
/// Use inside tab rows to match Material baseline and icon spacing.
///
/// ## Parameters
///
/// - `args` — configures the label text, icon and spacing; see
///   [`TabLabelArgs`].
///
/// ## Examples
///
/// ```
/// use tessera_ui::{Dp, tessera};
/// use tessera_ui_basic_components::{
///     tabs::{TabLabelArgsBuilder, tab_label},
///     theme::{MaterialTheme, material_theme},
/// };
///
/// #[tessera]
/// fn demo() {
///     material_theme(MaterialTheme::default(), || {
///         tab_label(
///             TabLabelArgsBuilder::default()
///                 .text("Home")
///                 .build()
///                 .expect("builder construction failed"),
///         );
///     });
/// }
/// ```
#[tessera]
pub fn tab_label(args: TabLabelArgs) {
    let typography = use_context::<MaterialTheme>().get().typography;
    let style = typography.title_small;
    let content_color = use_context::<ContentColor>().get().current;

    if let Some(icon_content) = args.icon.clone() {
        icon(
            IconArgsBuilder::default()
                .content(icon_content)
                .size(args.icon_size)
                .build()
                .expect("builder construction failed"),
        );
    }

    let args_for_measure = args.clone();
    measure(Box::new(
        move |input| -> Result<ComputedData, MeasurementError> {
            let padding_px: Px = args_for_measure.horizontal_text_padding.into();
            let indicator_height_px: Px = args_for_measure.indicator_height.into();

            let icon_id = input.children_ids.first().copied();
            let has_icon = icon_id.is_some();
            let has_text = !args_for_measure.text.is_empty();
            let icon_size = if let Some(icon_id) = icon_id {
                let icon_constraint = Constraint::new(
                    DimensionValue::Fixed(args_for_measure.icon_size.into()),
                    DimensionValue::Fixed(args_for_measure.icon_size.into()),
                );
                input.measure_child(icon_id, &icon_constraint)?
            } else {
                ComputedData::ZERO
            };

            let max_width_px = match input.parent_constraint.width() {
                DimensionValue::Fixed(w) => Some(w),
                DimensionValue::Wrap { max, .. } => max,
                DimensionValue::Fill { max, .. } => max,
            };
            let text_max_width = max_width_px
                .map(|w| (w - padding_px * 2).max(Px(0)))
                .map(|w| w.to_f32());

            let line_height = style
                .line_height
                .unwrap_or(Dp(style.font_size.0 * 1.2))
                .to_pixels_f32();
            let (text_data, text_width, text_height, first_baseline, last_baseline, line_count) =
                if has_text {
                    let text_data = TextData::new(
                        args_for_measure.text.clone(),
                        content_color,
                        style.font_size.to_pixels_f32(),
                        line_height,
                        TextConstraint {
                            max_width: text_max_width,
                            max_height: None,
                        },
                    );
                    let text_width: Px = text_data.size[0].into();
                    let text_height: Px = text_data.size[1].into();
                    let first_baseline = text_data.first_baseline;
                    let last_baseline = text_data.last_baseline;
                    let line_count = text_data.line_count;
                    (
                        Some(text_data),
                        text_width,
                        text_height,
                        first_baseline,
                        last_baseline,
                        line_count,
                    )
                } else {
                    (None, Px(0), Px(0), 0.0, 0.0, 0)
                };

            let text_container_width = if has_text {
                text_width + padding_px * 2
            } else {
                Px(0)
            };
            let content_width_measure = text_container_width.max(icon_size.width);
            let width = resolve_dimension(input.parent_constraint.width(), content_width_measure);

            let small_height: Px = Dp(48.0).into();
            let large_height: Px = Dp(72.0).into();
            let icon_distance_from_baseline: Px = Dp(20.0).into();
            let base_height = if has_icon && has_text {
                large_height
            } else {
                small_height
            };
            let combined_height = if has_icon && has_text {
                icon_size.height + text_height + icon_distance_from_baseline
            } else {
                icon_size.height.max(text_height)
            };
            let height = resolve_dimension(
                input.parent_constraint.height(),
                base_height.max(combined_height),
            );

            let text_area_width = (width - padding_px * 2).max(Px(0));
            let text_x = Px::saturating_from_f32(
                padding_px.to_f32()
                    + ((text_area_width.to_f32() - text_width.to_f32()).max(0.0) / 2.0),
            );

            let (text_y, icon_x, icon_y) = if has_icon && has_text {
                let is_single_line =
                    line_count <= 1 || (first_baseline - last_baseline).abs() < 0.5;
                let baseline_offset: Px = if is_single_line {
                    Dp(14.0).into()
                } else {
                    Dp(6.0).into()
                };
                let text_offset = baseline_offset + indicator_height_px;
                let text_y = height - Px::saturating_from_f32(last_baseline) - text_offset;

                let icon_offset = icon_size.height + icon_distance_from_baseline
                    - Px::saturating_from_f32(first_baseline);
                let icon_y = text_y - icon_offset;
                let icon_x =
                    Px::saturating_from_f32((width.to_f32() - icon_size.width.to_f32()) / 2.0);
                (text_y, icon_x, icon_y)
            } else {
                let text_y =
                    Px::saturating_from_f32((height.to_f32() - text_height.to_f32()) / 2.0);
                let icon_x =
                    Px::saturating_from_f32((width.to_f32() - icon_size.width.to_f32()) / 2.0);
                let icon_y =
                    Px::saturating_from_f32((height.to_f32() - icon_size.height.to_f32()) / 2.0);
                (text_y, icon_x, icon_y)
            };

            if let Some(icon_id) = icon_id {
                input.place_child(icon_id, PxPosition::new(icon_x, icon_y));
            }

            if let Some(text_data) = text_data {
                let drawable = TextCommand {
                    data: text_data,
                    offset: PxPosition::new(text_x, text_y),
                };
                input.metadata_mut().push_draw_command(drawable);
            }

            Ok(ComputedData { width, height })
        },
    ));
}

#[tessera]
fn tabs_content_container(scroll_offset: Px, children: Vec<Box<dyn FnOnce() + Send + Sync>>) {
    for child in children {
        child();
    }

    measure(Box::new(
        move |input| -> Result<ComputedData, MeasurementError> {
            input.enable_clipping();

            let mut max_height = Px(0);
            let container_width = resolve_dimension(input.parent_constraint.width(), Px(0));

            for &child_id in input.children_ids.iter() {
                let child_constraint = Constraint::new(
                    DimensionValue::Fixed(container_width),
                    DimensionValue::Wrap {
                        min: None,
                        max: None,
                    },
                );
                let child_size = input.measure_child(child_id, &child_constraint)?;
                max_height = max_height.max(child_size.height);
            }

            let mut current_x = scroll_offset;
            for &child_id in input.children_ids.iter() {
                input.place_child(child_id, PxPosition::new(current_x, Px(0)));
                current_x += container_width;
            }

            Ok(ComputedData {
                width: container_width,
                height: max_height,
            })
        },
    ));
}

/// # tabs
///
/// Renders a set of tabs with corresponding content pages.
///
/// ## Usage
///
/// Display a row of tab titles and a content area that switches between
/// different views.
///
/// ## Parameters
///
/// - `args` — configures the tabs' layout, initial active tab, and indicator
///   color; see [`TabsArgs`].
/// - `scope_config` — a closure that receives a [`TabsScope`] for defining each
///   tab's title and content. Use [`TabsScope::child_with_color`] to let the
///   component supply Material-compliant active/inactive colors.
///
/// ## Examples
///
/// ```
/// use tessera_ui::{Dp, tessera};
/// use tessera_ui_basic_components::{
///     tabs::{TabsArgsBuilder, tabs},
///     text::{TextArgsBuilder, text},
/// };
///
/// #[tessera]
/// fn demo() {
///     tabs(
///         TabsArgsBuilder::default()
///             .initial_active_tab(1)
///             .build()
///             .expect("builder construction failed"),
///         |scope| {
///             scope.child_with_color(
///                 |color| {
///                     text(
///                         TextArgsBuilder::default()
///                             .text("Flights".to_string())
///                             .color(color)
///                             .size(Dp(14.0))
///                             .build()
///                             .expect("builder construction failed"),
///                     )
///                 },
///                 || {
///                     text(
///                         TextArgsBuilder::default()
///                             .text("Content for Flights")
///                             .build()
///                             .expect("builder construction failed"),
///                     )
///                 },
///             );
///             scope.child_with_color(
///                 |color| {
///                     text(
///                         TextArgsBuilder::default()
///                             .text("Hotel".to_string())
///                             .color(color)
///                             .size(Dp(14.0))
///                             .build()
///                             .expect("builder construction failed"),
///                     )
///                 },
///                 || {
///                     text(
///                         TextArgsBuilder::default()
///                             .text("Content for Hotel")
///                             .build()
///                             .expect("builder construction failed"),
///                     )
///                 },
///             );
///         },
///     );
/// }
/// ```
#[tessera]
pub fn tabs<F>(args: TabsArgs, scope_config: F)
where
    F: FnOnce(&mut TabsScope),
{
    let controller = remember(|| TabsController::new(args.initial_active_tab));
    tabs_with_controller(args, controller, scope_config);
}

/// # tabs_with_controller
///
/// Controlled variant that accepts an explicit controller.
///
/// ## Usage
///
/// Use when you need to synchronize active tab selection across components or
/// restore selection after remounts.
///
/// ## Parameters
///
/// - `args` — configures the tabs' layout and indicator color; see
///   [`TabsArgs`].
/// - `controller` — a [`TabsController`] storing the active tab index and
///   animation progress.
/// - `scope_config` — a closure that receives a [`TabsScope`] for defining each
///   tab's title and content.
///
/// ## Examples
///
/// ```
/// use tessera_ui::{remember, tessera};
/// use tessera_ui_basic_components::{
///     tabs::{TabsArgsBuilder, TabsController, tabs_with_controller},
///     text::{TextArgsBuilder, text},
/// };
///
/// #[tessera]
/// fn demo() {
///     let controller = remember(|| TabsController::new(0));
///     tabs_with_controller(
///         TabsArgsBuilder::default()
///             .build()
///             .expect("builder construction failed"),
///         controller,
///         |scope| {
///             scope.child_with_color(
///                 |color| {
///                     text(
///                         TextArgsBuilder::default()
///                             .text("A".to_string())
///                             .color(color)
///                             .build()
///                             .expect("builder construction failed"),
///                     )
///                 },
///                 || {
///                     text(
///                         TextArgsBuilder::default()
///                             .text("Tab A")
///                             .build()
///                             .expect("builder construction failed"),
///                     )
///                 },
///             );
///             scope.child_with_color(
///                 |color| {
///                     text(
///                         TextArgsBuilder::default()
///                             .text("B".to_string())
///                             .color(color)
///                             .build()
///                             .expect("builder construction failed"),
///                     )
///                 },
///                 || {
///                     text(
///                         TextArgsBuilder::default()
///                             .text("Tab B")
///                             .build()
///                             .expect("builder construction failed"),
///                     )
///                 },
///             );
///         },
///     );
/// }
/// ```
#[tessera]
pub fn tabs_with_controller<F>(args: TabsArgs, controller: State<TabsController>, scope_config: F)
where
    F: FnOnce(&mut TabsScope),
{
    let mut tabs = Vec::new();
    let mut scope = TabsScope { tabs: &mut tabs };
    scope_config(&mut scope);

    let num_tabs = tabs.len();
    if num_tabs == 0 {
        return;
    }
    let active_tab = controller
        .with(|c| c.active_tab())
        .min(num_tabs.saturating_sub(1));

    let (title_closures, content_closures): (Vec<_>, Vec<_>) =
        tabs.into_iter().map(|def| (def.title, def.content)).unzip();

    surface(
        SurfaceArgsBuilder::default()
            .style(args.container_color.into())
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .shape(Shape::RECTANGLE)
            .build()
            .expect("builder construction failed"),
        || {},
    );

    surface(
        SurfaceArgsBuilder::default()
            .style(args.divider_color.into())
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .shape(Shape::RECTANGLE)
            .build()
            .expect("builder construction failed"),
        || {},
    );

    let indicator_shape = match args.variant {
        TabsVariant::Primary => Shape::rounded_rectangle(Dp(3.0)),
        TabsVariant::Secondary => Shape::RECTANGLE,
    };

    surface(
        SurfaceArgsBuilder::default()
            .style(args.indicator_color.into())
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .shape(indicator_shape)
            .build()
            .expect("builder construction failed"),
        || {},
    );

    let ripple_color = TabsDefaults::ripple_color(args.active_content_color);

    for (index, child) in title_closures.into_iter().enumerate() {
        let label_color = if !args.enabled {
            args.disabled_content_color
        } else if index == active_tab {
            args.active_content_color
        } else {
            args.inactive_content_color
        };

        let tab_width = if args.scrollable {
            DimensionValue::Wrap {
                min: None,
                max: None,
            }
        } else {
            DimensionValue::FILLED
        };

        let tab_height = match &child {
            TabTitle::Label {
                text,
                icon: Some(_),
            } if !text.is_empty() => TabsDefaults::LARGE_TAB_HEIGHT,
            _ => args.min_tab_height,
        };

        let mut tab_surface = SurfaceArgsBuilder::default()
            .style(Color::TRANSPARENT.into())
            .content_alignment(Alignment::Center)
            .content_color(label_color)
            .width(tab_width)
            .height(DimensionValue::Fixed(tab_height.into()))
            .ripple_color(ripple_color)
            .shape(Shape::RECTANGLE)
            .enabled(args.enabled)
            .accessibility_role(tessera_ui::accesskit::Role::Tab)
            .accessibility_focusable(true);

        if let TabTitle::Label { text, .. } = &child {
            tab_surface = tab_surface.accessibility_label(text.clone());
        }

        if args.enabled {
            tab_surface = tab_surface.on_click(move || {
                controller.with_mut(|c| c.set_active_tab(index));
            });
        }

        surface(
            tab_surface.build().expect("builder construction failed"),
            move || match child {
                TabTitle::Custom(render) => render(),
                TabTitle::Themed(render) => render(label_color),
                TabTitle::Label { text, icon } => {
                    let mut builder = TabLabelArgsBuilder::default();
                    builder = builder.text(text);
                    if let Some(icon) = icon {
                        builder = builder.icon(icon);
                    }
                    builder = builder.horizontal_text_padding(args.tab_padding);
                    builder = builder.indicator_height(args.indicator_height);
                    tab_label(builder.build().expect("builder construction failed"));
                }
            },
        );
    }

    let scroll_offset = controller.with(|c| c.content_scroll_px());

    tabs_content_container(scroll_offset, content_closures);

    input_handler(Box::new(move |input| {
        input
            .accessibility()
            .role(tessera_ui::accesskit::Role::TabList)
            .commit();
        controller.with_mut(|c| c.tick(Instant::now()));

        if args.scrollable
            && let Some(pos) = input.cursor_position_rel
            && pos.y < controller.with(|c| c.tab_bar_height())
        {
            let mut consumed_scroll = false;
            for event in input
                .cursor_events
                .iter()
                .filter_map(|event| match &event.content {
                    CursorEventContent::Scroll(event) => Some(event),
                    _ => None,
                })
            {
                let delta = if event.delta_x.abs() >= 0.01 {
                    event.delta_x
                } else {
                    event.delta_y
                };
                if delta.abs() < 0.01 {
                    continue;
                }

                controller.with_mut(|c| {
                    let current = c.tab_row_scroll_offset.target;
                    let max = c.tab_row_scroll_max().to_f32();
                    let next = (current - delta).clamp(0.0, max);
                    c.set_tab_row_scroll_immediate(Px::saturating_from_f32(next));
                });
                consumed_scroll = true;
            }

            if consumed_scroll {
                input
                    .cursor_events
                    .retain(|event| !matches!(event.content, CursorEventContent::Scroll(_)));
            }
        }
    }));

    let tabs_args = args.clone();

    measure(Box::new(
        move |input| -> Result<ComputedData, MeasurementError> {
            let tabs_intrinsic_constraint = Constraint::new(tabs_args.width, tabs_args.height);
            let tabs_effective_constraint =
                tabs_intrinsic_constraint.merge(input.parent_constraint);

            let container_id = input.children_ids[0];
            let divider_id = input.children_ids[1];
            let indicator_id = input.children_ids[2];
            let title_ids = &input.children_ids[3..=num_tabs + 2];
            let content_container_id = input.children_ids[num_tabs + 3];

            let horizontal_padding = tabs_args.tab_padding.to_px().to_f32() * 2.0;
            let indicator_min_width: Px = tabs_args.indicator_min_width.into();
            let available_width = match tabs_effective_constraint.width {
                DimensionValue::Fixed(v) => Some(v),
                DimensionValue::Wrap { max, .. } => max,
                DimensionValue::Fill { max, .. } => max,
            };

            let is_scrollable = tabs_args.scrollable || available_width.is_none();
            let match_content_size = matches!(tabs_args.variant, TabsVariant::Primary);

            if is_scrollable {
                input.enable_clipping();
            }

            let (
                final_width,
                strip_width_total,
                tab_widths,
                tab_lefts,
                indicator_widths,
                titles_max_height,
                scroll_target,
            ) = if !is_scrollable {
                let final_width = available_width.unwrap_or(Px(0));
                let tab_width = if num_tabs == 0 {
                    Px(0)
                } else {
                    Px(final_width.0 / num_tabs as i32)
                };

                let measure_constraints: Vec<_> = title_ids
                    .iter()
                    .map(|&id| {
                        (
                            id,
                            Constraint::new(
                                DimensionValue::Fixed(tab_width),
                                DimensionValue::Wrap {
                                    min: None,
                                    max: None,
                                },
                            ),
                        )
                    })
                    .collect();
                let title_results = input.measure_children(measure_constraints)?;

                let mut titles_max_height = Px(0);
                for &title_id in title_ids {
                    if let Some(result) = title_results.get(&title_id) {
                        titles_max_height = titles_max_height.max(result.height);
                    }
                }

                let intrinsic_constraints: Vec<_> = title_ids
                    .iter()
                    .map(|&id| {
                        (
                            id,
                            Constraint::new(
                                DimensionValue::Wrap {
                                    min: None,
                                    max: Some(tab_width),
                                },
                                DimensionValue::Fixed(titles_max_height),
                            ),
                        )
                    })
                    .collect();
                let intrinsic_results = input.measure_children(intrinsic_constraints)?;

                let indicator_widths: Vec<Px> = title_ids
                    .iter()
                    .map(|id| {
                        if match_content_size {
                            let intrinsic_width = intrinsic_results
                                .get(id)
                                .map_or(Px(0), |s| s.width)
                                .min(tab_width);
                            let content_width =
                                (intrinsic_width.to_f32() - horizontal_padding).max(0.0);
                            Px::saturating_from_f32(content_width).max(indicator_min_width)
                        } else {
                            tab_width
                        }
                    })
                    .collect();

                let tab_widths = vec![tab_width; num_tabs];
                let tab_lefts: Vec<Px> = (0..num_tabs)
                    .map(|index| Px(index as i32 * tab_width.0))
                    .collect();

                (
                    final_width,
                    final_width,
                    tab_widths,
                    tab_lefts,
                    indicator_widths,
                    titles_max_height,
                    Px(0),
                )
            } else {
                let min_tab_width: Px = tabs_args.min_scrollable_tab_width.into();
                let edge_padding: Px = tabs_args.edge_padding.into();

                let measure_constraints: Vec<_> = title_ids
                    .iter()
                    .map(|&id| {
                        (
                            id,
                            Constraint::new(
                                DimensionValue::Wrap {
                                    min: Some(min_tab_width),
                                    max: None,
                                },
                                DimensionValue::Wrap {
                                    min: None,
                                    max: None,
                                },
                            ),
                        )
                    })
                    .collect();
                let title_results = input.measure_children(measure_constraints)?;

                let mut tab_widths = Vec::with_capacity(num_tabs);
                let mut titles_max_height = Px(0);
                for &title_id in title_ids {
                    if let Some(result) = title_results.get(&title_id) {
                        tab_widths.push(result.width);
                        titles_max_height = titles_max_height.max(result.height);
                    }
                }

                let intrinsic_constraints: Vec<_> = title_ids
                    .iter()
                    .map(|&id| {
                        (
                            id,
                            Constraint::new(
                                DimensionValue::Wrap {
                                    min: None,
                                    max: None,
                                },
                                DimensionValue::Fixed(titles_max_height),
                            ),
                        )
                    })
                    .collect();
                let intrinsic_results = input.measure_children(intrinsic_constraints)?;

                let mut indicator_widths = Vec::with_capacity(num_tabs);
                for (idx, &title_id) in title_ids.iter().enumerate() {
                    let tab_width = tab_widths.get(idx).copied().unwrap_or(Px(0));
                    if match_content_size {
                        let intrinsic_width = intrinsic_results
                            .get(&title_id)
                            .map_or(Px(0), |s| s.width)
                            .min(tab_width);
                        let content_width =
                            (intrinsic_width.to_f32() - horizontal_padding).max(0.0);
                        let indicator_width =
                            Px::saturating_from_f32(content_width).max(indicator_min_width);
                        indicator_widths.push(indicator_width);
                    } else {
                        indicator_widths.push(tab_width);
                    }
                }

                let mut tab_lefts = Vec::with_capacity(num_tabs);
                let mut left = edge_padding;
                for width in &tab_widths {
                    tab_lefts.push(left);
                    left += *width;
                }

                let strip_width_total = left + edge_padding;
                let final_width = available_width.unwrap_or(strip_width_total);

                let max_scroll = (strip_width_total - final_width).max(Px(0));
                controller.with_mut(|c| c.set_tab_row_scroll_bounds(max_scroll));

                let selected_left = tab_lefts.get(active_tab).copied().unwrap_or(edge_padding);
                let selected_width = tab_widths.get(active_tab).copied().unwrap_or(Px(0));
                let selected_center = selected_left + Px(selected_width.0.saturating_div(2));
                let target_scroll_f = (selected_center.to_f32() - final_width.to_f32() / 2.0)
                    .clamp(0.0, max_scroll.to_f32());
                let scroll_target = Px::saturating_from_f32(target_scroll_f);

                (
                    final_width,
                    strip_width_total,
                    tab_widths,
                    tab_lefts,
                    indicator_widths,
                    titles_max_height,
                    scroll_target,
                )
            };

            let content_container_constraint = Constraint::new(
                DimensionValue::Fixed(final_width),
                DimensionValue::Wrap {
                    min: None,
                    max: None,
                },
            );
            let content_container_size =
                input.measure_child(content_container_id, &content_container_constraint)?;

            let page_width = content_container_size.width;
            let target_offset = -Px(active_tab as i32 * page_width.0);
            controller.with_mut(|c| c.set_content_scroll_target(target_offset));

            if is_scrollable {
                controller.with_mut(|c| {
                    if !c.tab_row_scroll_user_overridden {
                        c.set_tab_row_scroll_target(scroll_target);
                    }
                });
            }

            let current_scroll_px = if is_scrollable {
                controller.with(|c| c.tab_row_scroll_px())
            } else {
                Px(0)
            };

            let (indicator_width, indicator_x) = {
                let desired_width = indicator_widths.get(active_tab).copied().unwrap_or(Px(0));
                let clamped_width = clamp_px(
                    desired_width,
                    tabs_args.indicator_min_width.into(),
                    tabs_args.indicator_max_width.map(|v| v.into()),
                );

                let tab_left = tab_lefts.get(active_tab).copied().unwrap_or(Px(0));
                let tab_width = tab_widths.get(active_tab).copied().unwrap_or(Px(0));

                let centered_x = tab_left + Px((tab_width.0 - clamped_width.0) / 2);

                controller.with_mut(|c| c.set_indicator_targets(clamped_width, centered_x));
                (
                    controller.with(|c| c.indicator_width_px()),
                    controller.with(|c| c.indicator_x_px()),
                )
            };

            let indicator_height: Px = tabs_args.indicator_height.into();
            let indicator_constraint = Constraint::new(
                DimensionValue::Fixed(indicator_width),
                DimensionValue::Fixed(indicator_height),
            );
            let _ = input.measure_child(indicator_id, &indicator_constraint)?;

            let divider_height: Px = TabsDefaults::DIVIDER_HEIGHT.into();
            let divider_width = if is_scrollable {
                strip_width_total
            } else {
                final_width
            };
            let divider_constraint = Constraint::new(
                DimensionValue::Fixed(divider_width),
                DimensionValue::Fixed(divider_height),
            );
            let _ = input.measure_child(divider_id, &divider_constraint)?;

            let tab_bar_height = titles_max_height.max(tabs_args.min_tab_height.into());
            controller.with_mut(|c| c.set_tab_bar_height(tab_bar_height));
            let final_height = tab_bar_height + content_container_size.height;
            let title_offset_y = Px((tab_bar_height.0 - titles_max_height.0) / 2).max(Px(0));

            let title_constraints: Vec<_> = title_ids
                .iter()
                .enumerate()
                .map(|(idx, &id)| {
                    (
                        id,
                        Constraint::new(
                            DimensionValue::Fixed(tab_widths.get(idx).copied().unwrap_or(Px(0))),
                            DimensionValue::Fixed(tab_bar_height),
                        ),
                    )
                })
                .collect();
            let _ = input.measure_children(title_constraints)?;

            let container_constraint = Constraint::new(
                DimensionValue::Fixed(final_width),
                DimensionValue::Fixed(tab_bar_height),
            );
            let _ = input.measure_child(container_id, &container_constraint)?;

            for (i, &title_id) in title_ids.iter().enumerate() {
                let x = tab_lefts.get(i).copied().unwrap_or(Px(0)) - current_scroll_px;
                input.place_child(title_id, PxPosition::new(x, title_offset_y));
            }

            input.place_child(container_id, PxPosition::new(Px(0), Px(0)));
            input.place_child(
                divider_id,
                PxPosition::new(
                    if is_scrollable {
                        -current_scroll_px
                    } else {
                        Px(0)
                    },
                    tab_bar_height - divider_height,
                ),
            );
            input.place_child(
                indicator_id,
                PxPosition::new(
                    indicator_x - current_scroll_px,
                    tab_bar_height - indicator_height,
                ),
            );

            input.place_child(content_container_id, PxPosition::new(Px(0), tab_bar_height));

            Ok(ComputedData {
                width: final_width,
                height: final_height,
            })
        },
    ));
}
