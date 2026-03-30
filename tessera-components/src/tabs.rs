//! A component for creating a tab-based layout.
//!
//! ## Usage
//!
//! Use to organize content into separate pages that can be switched between.
use tessera_ui::{
    CallbackWith, Color, ComputedData, Constraint, DimensionValue, Dp, FocusProperties,
    FocusRequester, FocusState, MeasurementError, Modifier, Px, PxPosition, RenderSlot, State,
    accesskit::Role,
    layout::{
        LayoutInput, LayoutOutput, LayoutPolicy, RenderInput, RenderPolicy, layout_primitive,
    },
    modifier::FocusModifierExt as _,
    receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::boxed,
    column::column,
    gesture_recognizer::{ScrollRecognizer, ScrollSettings},
    icon::{IconContent, icon as icon_component},
    modifier::{ModifierExt, SemanticsArgs, with_pointer_input},
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    text::text as text_component,
    theme::{ContentColor, MaterialAlpha, MaterialColorScheme, MaterialTheme},
};

const DEFAULT_SPATIAL_DAMPING_RATIO: f32 = 0.9;
const DEFAULT_SPATIAL_STIFFNESS: f32 = 700.0;

/// Visual variants supported by [`tabs`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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

#[derive(Clone, PartialEq, Copy, Debug)]
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

    fn is_animating(self) -> bool {
        (self.value - self.target).abs() >= 0.5 || self.velocity.abs() >= 0.5
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
#[derive(Clone, PartialEq)]
pub struct TabsController {
    active_tab: usize,
    indicator_x: Spring1D,
    indicator_width: Spring1D,
    content_scroll_offset: Spring1D,
    tab_row_scroll_offset: Spring1D,
    tab_row_scroll_max: Px,
    tab_row_scroll_user_overridden: bool,
    tab_bar_height: Px,
    last_frame_nanos: Option<u64>,
    indicator_initialized: bool,
    content_scroll_initialized: bool,
    tab_row_scroll_initialized: bool,
    pending_retarget_frame: bool,
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
            last_frame_nanos: None,
            indicator_initialized: false,
            content_scroll_initialized: false,
            tab_row_scroll_initialized: false,
            pending_retarget_frame: false,
        }
    }

    /// Set the active tab index.
    ///
    /// If the requested index equals the current active tab this is a no-op.
    pub fn set_active_tab(&mut self, index: usize) {
        if index != self.active_tab {
            self.active_tab = index;
            self.tab_row_scroll_user_overridden = false;
            self.pending_retarget_frame = true;
            self.last_frame_nanos = None;
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
        self.pending_retarget_frame = false;
        let target = target.max(Px(0)).min(self.tab_row_scroll_max);
        if !self.tab_row_scroll_initialized {
            self.tab_row_scroll_offset.snap_to(target.to_f32());
            self.tab_row_scroll_initialized = true;
        } else {
            self.last_frame_nanos = None;
            self.tab_row_scroll_offset.set_target(target.to_f32());
        }
    }

    fn tab_row_scroll_px(&self) -> Px {
        self.tab_row_scroll_offset.value_px()
    }

    fn set_content_scroll_target(&mut self, target: Px) {
        self.pending_retarget_frame = false;
        if !self.content_scroll_initialized {
            self.content_scroll_offset.snap_to(target.to_f32());
            self.content_scroll_initialized = true;
        } else {
            self.last_frame_nanos = None;
            self.content_scroll_offset.set_target(target.to_f32());
        }
    }

    fn content_scroll_px(&self) -> Px {
        self.content_scroll_offset.value_px()
    }

    fn set_indicator_targets(&mut self, width: Px, x: Px) {
        self.pending_retarget_frame = false;
        let width = width.max(Px(0)).to_f32();
        let x = x.to_f32();
        if !self.indicator_initialized {
            self.indicator_width.snap_to(width);
            self.indicator_x.snap_to(x);
            self.indicator_initialized = true;
        } else {
            self.last_frame_nanos = None;
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

    fn advance_from_frame_nanos(&mut self, frame_nanos: u64) {
        let dt = if let Some(last_frame_nanos) = self.last_frame_nanos {
            frame_nanos.saturating_sub(last_frame_nanos) as f32 / 1_000_000_000.0
        } else {
            1.0 / 60.0
        };
        self.last_frame_nanos = Some(frame_nanos);

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

    fn has_pending_animation_frame(&self) -> bool {
        self.pending_retarget_frame
            || self.indicator_x.is_animating()
            || self.indicator_width.is_animating()
            || self.content_scroll_offset.is_animating()
            || self.tab_row_scroll_offset.is_animating()
    }
}

impl Default for TabsController {
    fn default() -> Self {
        Self::new(0)
    }
}

/// A single tab item rendered by [`tabs`].
#[derive(Clone, PartialEq)]
pub struct TabItem {
    title: TabTitle,
    content: RenderSlot,
}

#[derive(Clone, PartialEq)]
enum TabTitle {
    Custom(RenderSlot),
    Themed(CallbackWith<Color>),
    Label {
        text: String,
        icon: Option<IconContent>,
    },
}

#[derive(Clone, PartialEq)]
struct TabsConfig {
    modifier: Modifier,
    variant: TabsVariant,
    initial_active_tab: usize,
    indicator_color: Color,
    container_color: Color,
    active_content_color: Color,
    inactive_content_color: Color,
    indicator_height: Dp,
    indicator_min_width: Dp,
    indicator_max_width: Option<Dp>,
    min_tab_height: Dp,
    tab_padding: Dp,
    enabled: bool,
    disabled_content_color: Color,
    divider_color: Color,
    scrollable: bool,
    edge_padding: Dp,
    min_scrollable_tab_width: Dp,
    items: Vec<TabItem>,
}

impl TabsBuilder {
    /// Adds a tab with its title and content builders.
    pub fn child<F1, F2>(mut self, title: F1, content: F2) -> Self
    where
        F1: Fn() + Send + Sync + 'static,
        F2: Fn() + Send + Sync + 'static,
    {
        self.props.items.push(TabItem {
            title: TabTitle::Custom(RenderSlot::new(title)),
            content: RenderSlot::new(content),
        });
        self
    }

    /// Adds a tab whose title closure receives the resolved content color.
    pub fn child_with_color<F1, F2>(mut self, title: F1, content: F2) -> Self
    where
        F1: Fn(Color) + Send + Sync + 'static,
        F2: Fn() + Send + Sync + 'static,
    {
        self.props.items.push(TabItem {
            title: TabTitle::Themed(CallbackWith::new(title)),
            content: RenderSlot::new(content),
        });
        self
    }

    /// Adds a tab whose title is rendered with the standard Material label
    /// layout.
    pub fn child_label<F>(mut self, text: impl Into<String>, content: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.props.items.push(TabItem {
            title: TabTitle::Label {
                text: text.into(),
                icon: None,
            },
            content: RenderSlot::new(content),
        });
        self
    }

    /// Adds a tab whose title is rendered with an icon and standard Material
    /// label layout.
    pub fn child_label_with_icon<F>(
        mut self,
        text: impl Into<String>,
        icon: impl Into<IconContent>,
        content: F,
    ) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.props.items.push(TabItem {
            title: TabTitle::Label {
                text: text.into(),
                icon: Some(icon.into()),
            },
            content: RenderSlot::new(content),
        });
        self
    }
}

impl TabLabelBuilder {
    /// Set the optional icon shown above the label text.
    pub fn icon(mut self, icon: impl Into<IconContent>) -> Self {
        self.props.icon = Some(icon.into());
        self
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
/// - `text` — text shown in the tab.
/// - `icon` — optional icon shown above the text.
/// - `horizontal_text_padding` — horizontal padding applied to the text area.
/// - `icon_size` — size of the icon when present.
///
/// ## Examples
///
/// ```
/// use tessera_components::tabs::tab_label;
/// use tessera_ui::tessera;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// #[tessera]
/// fn demo() {
/// #     material_theme()
/// #         .theme(|| MaterialTheme::default())
/// #         .child(|| {
///     tab_label().text("Home");
/// #         });
/// }
/// ```
#[tessera]
pub fn tab_label(
    #[prop(into)] text: String,
    #[prop(skip_setter)] icon: Option<IconContent>,
    horizontal_text_padding: Option<Dp>,
    icon_size: Option<Dp>,
) {
    let typography = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .typography;
    let style = typography.title_small;
    let content_color = use_context::<ContentColor>()
        .map(|c| c.get().current)
        .unwrap_or(ContentColor::default().current);

    let has_icon = icon.is_some();
    let has_text = !text.is_empty();
    let icon_content = icon.clone();
    let text_content = text.clone();
    let icon_size = icon_size.unwrap_or(Dp(24.0));
    let horizontal_padding = horizontal_text_padding.unwrap_or(TabsDefaults::TAB_PADDING);
    let font_size = style.font_size;
    let line_height = style.line_height.unwrap_or(Dp(style.font_size.0 * 1.2));

    // Determine container height based on content type
    let small_height = TabsDefaults::MIN_TAB_HEIGHT;
    let large_height = TabsDefaults::LARGE_TAB_HEIGHT;
    let container_height = if has_icon && has_text {
        large_height
    } else {
        small_height
    };

    // Use boxed to center the content within the tab
    let modifier = Modifier::new()
        .constrain(
            Some(DimensionValue::Wrap {
                min: None,
                max: None,
            }),
            Some(DimensionValue::Fixed(container_height.into())),
        )
        .padding_symmetric(horizontal_padding, Dp(0.0));

    boxed()
        .alignment(Alignment::Center)
        .modifier(modifier)
        .children(move || {
            {
                if has_icon && has_text {
                    let icon_content = icon_content.clone();
                    let text_content = text_content.clone();
                    column()
                        .main_axis_alignment(MainAxisAlignment::Center)
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .modifier(
                            Modifier::new()
                                .constrain(Some(DimensionValue::WRAP), Some(DimensionValue::WRAP)),
                        )
                        .children(move || {
                            let icon_content = icon_content.clone();
                            {
                                if let Some(ic) = icon_content.clone() {
                                    match ic {
                                        IconContent::Vector(data) => {
                                            icon_component()
                                                .vector(data)
                                                .size(icon_size)
                                                .tint(content_color);
                                        }
                                        IconContent::Raster(data) => {
                                            icon_component()
                                                .raster(data)
                                                .size(icon_size)
                                                .tint(content_color);
                                        }
                                    }
                                }
                            };
                            {
                                spacer().modifier(Modifier::new().constrain(
                                    Some(DimensionValue::Fixed(Px(0))),
                                    Some(DimensionValue::Fixed(Dp(2.0).into())),
                                ));
                            };
                            let text_content = text_content.clone();
                            {
                                text_component()
                                    .content(text_content.clone())
                                    .color(content_color)
                                    .size(font_size)
                                    .line_height(line_height);
                            };
                        });
                } else if has_icon {
                    if let Some(ic) = icon_content.clone() {
                        match ic {
                            IconContent::Vector(data) => {
                                icon_component()
                                    .vector(data)
                                    .size(icon_size)
                                    .tint(content_color);
                            }
                            IconContent::Raster(data) => {
                                icon_component()
                                    .raster(data)
                                    .size(icon_size)
                                    .tint(content_color);
                            }
                        }
                    }
                } else if has_text {
                    text_component()
                        .content(text_content.clone())
                        .color(content_color)
                        .size(font_size)
                        .line_height(line_height);
                }
            };
        });
}

#[derive(Clone)]
struct TabsContentLayout {
    scroll_offset: Px,
}

impl PartialEq for TabsContentLayout {
    fn eq(&self, other: &Self) -> bool {
        self.scroll_offset == other.scroll_offset
    }
}

impl LayoutPolicy for TabsContentLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let mut max_height = Px(0);
        let container_width = resolve_dimension(input.parent_constraint().width(), Px(0));

        for &child_id in input.children_ids().iter() {
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

        let mut current_x = self.scroll_offset;
        for &child_id in input.children_ids().iter() {
            output.place_child(child_id, PxPosition::new(current_x, Px(0)));
            current_x += container_width;
        }

        Ok(ComputedData {
            width: container_width,
            height: max_height,
        })
    }
}

impl RenderPolicy for TabsContentLayout {
    fn record(&self, input: &RenderInput<'_>) {
        input.metadata_mut().set_clips_children(true);
    }
}

fn render_tabs_content_container(scroll_offset: Px, children: Vec<RenderSlot>) {
    let policy = TabsContentLayout { scroll_offset };
    layout_primitive()
        .layout_policy(policy.clone())
        .render_policy(policy)
        .child(move || {
            for child in &children {
                child.render();
            }
        });
}

#[derive(Clone)]
struct TabsLayout {
    args: TabsConfig,
    num_tabs: usize,
    active_tab: usize,
    controller: State<TabsController>,
    tab_row_scroll_px: Px,
    indicator_x_px: Px,
    indicator_width_px: Px,
}

impl PartialEq for TabsLayout {
    fn eq(&self, other: &Self) -> bool {
        self.num_tabs == other.num_tabs
            && self.active_tab == other.active_tab
            && self.tab_row_scroll_px == other.tab_row_scroll_px
            && self.indicator_x_px == other.indicator_x_px
            && self.indicator_width_px == other.indicator_width_px
            && self.args.variant == other.args.variant
            && self.args.indicator_height == other.args.indicator_height
            && self.args.indicator_min_width == other.args.indicator_min_width
            && self.args.indicator_max_width == other.args.indicator_max_width
            && self.args.min_tab_height == other.args.min_tab_height
            && self.args.tab_padding == other.args.tab_padding
            && self.args.scrollable == other.args.scrollable
            && self.args.edge_padding == other.args.edge_padding
            && self.args.min_scrollable_tab_width == other.args.min_scrollable_tab_width
    }
}

impl LayoutPolicy for TabsLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let tabs_effective_constraint = Constraint::new(
            input.parent_constraint().width(),
            input.parent_constraint().height(),
        );

        let container_id = input.children_ids()[0];
        let divider_id = input.children_ids()[1];
        let indicator_id = input.children_ids()[2];
        let title_ids = &input.children_ids()[3..=self.num_tabs + 2];
        let content_container_id = input.children_ids()[self.num_tabs + 3];

        let horizontal_padding = self.args.tab_padding.to_px().to_f32() * 2.0;
        let indicator_min_width: Px = self.args.indicator_min_width.into();
        let available_width = match tabs_effective_constraint.width {
            DimensionValue::Fixed(v) => Some(v),
            DimensionValue::Wrap { max, .. } => max,
            DimensionValue::Fill { max, .. } => max,
        };

        let is_scrollable = self.args.scrollable || available_width.is_none();
        let match_content_size = matches!(self.args.variant, TabsVariant::Primary);

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
            let tab_width = if self.num_tabs == 0 {
                Px(0)
            } else {
                Px(final_width.0 / self.num_tabs as i32)
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
            let title_results = input.measure_children_untracked(measure_constraints)?;

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
            let intrinsic_results = input.measure_children_untracked(intrinsic_constraints)?;

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

            let tab_widths = vec![tab_width; self.num_tabs];
            let tab_lefts: Vec<Px> = (0..self.num_tabs)
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
            let min_tab_width: Px = self.args.min_scrollable_tab_width.into();
            let edge_padding: Px = self.args.edge_padding.into();

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
            let title_results = input.measure_children_untracked(measure_constraints)?;

            let mut tab_widths = Vec::with_capacity(self.num_tabs);
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
            let intrinsic_results = input.measure_children_untracked(intrinsic_constraints)?;

            let mut indicator_widths = Vec::with_capacity(self.num_tabs);
            for (idx, &title_id) in title_ids.iter().enumerate() {
                let tab_width = tab_widths.get(idx).copied().unwrap_or(Px(0));
                if match_content_size {
                    let intrinsic_width = intrinsic_results
                        .get(&title_id)
                        .map_or(Px(0), |s| s.width)
                        .min(tab_width);
                    let content_width = (intrinsic_width.to_f32() - horizontal_padding).max(0.0);
                    let indicator_width =
                        Px::saturating_from_f32(content_width).max(indicator_min_width);
                    indicator_widths.push(indicator_width);
                } else {
                    indicator_widths.push(tab_width);
                }
            }

            let mut tab_lefts = Vec::with_capacity(self.num_tabs);
            let mut left = edge_padding;
            for width in &tab_widths {
                tab_lefts.push(left);
                left += *width;
            }

            let strip_width_total = left + edge_padding;
            let final_width = available_width.unwrap_or(strip_width_total);

            let max_scroll = (strip_width_total - final_width).max(Px(0));
            let should_update_scroll_bounds = self.controller.with(|c| {
                if c.tab_row_scroll_max != max_scroll {
                    return true;
                }
                let clamped = c
                    .tab_row_scroll_offset
                    .value
                    .clamp(0.0, max_scroll.to_f32());
                (c.tab_row_scroll_offset.value - clamped).abs() > f32::EPSILON
            });
            if should_update_scroll_bounds {
                self.controller
                    .with_mut(|c| c.set_tab_row_scroll_bounds(max_scroll));
            }

            let selected_left = tab_lefts
                .get(self.active_tab)
                .copied()
                .unwrap_or(edge_padding);
            let selected_width = tab_widths.get(self.active_tab).copied().unwrap_or(Px(0));
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
        let target_offset = -Px(self.active_tab as i32 * page_width.0);
        let should_update_content_scroll = self.controller.with(|c| {
            !c.content_scroll_initialized
                || (c.content_scroll_offset.target - target_offset.to_f32()).abs() > f32::EPSILON
        });
        if should_update_content_scroll {
            self.controller
                .with_mut(|c| c.set_content_scroll_target(target_offset));
        }

        if is_scrollable {
            let should_update_tab_row_scroll_target = self.controller.with(|c| {
                !c.tab_row_scroll_user_overridden
                    && (!c.tab_row_scroll_initialized
                        || (c.tab_row_scroll_offset.target - scroll_target.to_f32()).abs()
                            > f32::EPSILON)
            });
            if should_update_tab_row_scroll_target {
                self.controller
                    .with_mut(|c| c.set_tab_row_scroll_target(scroll_target));
            }
        }

        let current_scroll_px = if is_scrollable {
            self.controller.with(|c| c.tab_row_scroll_px())
        } else {
            Px(0)
        };

        let (indicator_width, indicator_x) = {
            let desired_width = indicator_widths
                .get(self.active_tab)
                .copied()
                .unwrap_or(Px(0));
            let clamped_width = clamp_px(
                desired_width,
                self.args.indicator_min_width.into(),
                self.args.indicator_max_width.map(|v| v.into()),
            );

            let tab_left = tab_lefts.get(self.active_tab).copied().unwrap_or(Px(0));
            let tab_width = tab_widths.get(self.active_tab).copied().unwrap_or(Px(0));

            let centered_x = tab_left + Px((tab_width.0 - clamped_width.0) / 2);

            let should_update_indicator_targets = self.controller.with(|c| {
                !c.indicator_initialized
                    || (c.indicator_width.target - clamped_width.to_f32()).abs() > f32::EPSILON
                    || (c.indicator_x.target - centered_x.to_f32()).abs() > f32::EPSILON
            });
            if should_update_indicator_targets {
                self.controller
                    .with_mut(|c| c.set_indicator_targets(clamped_width, centered_x));
            }
            (
                self.controller.with(|c| c.indicator_width_px()),
                self.controller.with(|c| c.indicator_x_px()),
            )
        };

        let indicator_height: Px = self.args.indicator_height.into();
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

        let tab_bar_height = titles_max_height.max(self.args.min_tab_height.into());
        let should_update_tab_bar_height =
            self.controller.with(|c| c.tab_bar_height != tab_bar_height);
        if should_update_tab_bar_height {
            self.controller
                .with_mut(|c| c.set_tab_bar_height(tab_bar_height));
        }
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
            output.place_child(title_id, PxPosition::new(x, title_offset_y));
        }

        output.place_child(container_id, PxPosition::new(Px(0), Px(0)));
        output.place_child(
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
        output.place_child(
            indicator_id,
            PxPosition::new(
                indicator_x - current_scroll_px,
                tab_bar_height - indicator_height,
            ),
        );

        output.place_child(content_container_id, PxPosition::new(Px(0), tab_bar_height));

        Ok(ComputedData {
            width: final_width,
            height: final_height,
        })
    }
}

impl RenderPolicy for TabsLayout {
    fn record(&self, input: &RenderInput<'_>) {
        input.metadata_mut().set_clips_children(true);
    }
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
/// - `modifier` — modifier chain applied to the tabs subtree.
/// - `variant` — visual variant of the tab row.
/// - `initial_active_tab` — index of the initially selected tab.
/// - `indicator_color` — optional override for the active indicator color.
/// - `container_color` — optional override for the tab row background color.
/// - `active_content_color` — optional override for active tab content color.
/// - `inactive_content_color` — optional override for inactive tab content
///   color.
/// - `indicator_height` — optional override for the indicator height.
/// - `indicator_min_width` — optional override for the minimum indicator width.
/// - `indicator_max_width` — optional override for the maximum indicator width.
/// - `min_tab_height` — optional override for the minimum tab height.
/// - `tab_padding` — optional override for the per-tab horizontal padding.
/// - `enabled` — whether the tab row is interactive.
/// - `disabled_content_color` — optional override for disabled tab content
///   color.
/// - `divider_color` — optional override for the divider color.
/// - `scrollable` — whether the tab row should scroll horizontally.
/// - `edge_padding` — optional override for scrollable edge padding.
/// - `min_scrollable_tab_width` — optional override for minimum scrollable tab
///   width.
/// - `items` — tabs added through the builder methods.
///
/// ## Examples
///
/// ```
/// use tessera_components::{tabs::tabs, text::text};
/// use tessera_ui::{Dp, tessera};
///
/// #[tessera]
/// fn demo() {
///     tabs()
///         .initial_active_tab(1)
///         .child_with_color(
///             |color| {
///                 text().content("Flights").color(color).size(Dp(14.0));
///             },
///             || {
///                 text().content("Content for Flights");
///             },
///         )
///         .child_with_color(
///             |color| {
///                 text().content("Hotel").color(color).size(Dp(14.0));
///             },
///             || {
///                 text().content("Content for Hotel");
///             },
///         );
/// }
/// ```
#[tessera]
pub fn tabs(
    modifier: Modifier,
    variant: TabsVariant,
    initial_active_tab: usize,
    indicator_color: Option<Color>,
    container_color: Option<Color>,
    active_content_color: Option<Color>,
    inactive_content_color: Option<Color>,
    indicator_height: Option<Dp>,
    indicator_min_width: Option<Dp>,
    indicator_max_width: Option<Dp>,
    min_tab_height: Option<Dp>,
    tab_padding: Option<Dp>,
    enabled: Option<bool>,
    disabled_content_color: Option<Color>,
    divider_color: Option<Color>,
    scrollable: bool,
    edge_padding: Option<Dp>,
    min_scrollable_tab_width: Option<Dp>,
    #[prop(skip_setter)] items: Vec<TabItem>,
) {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let config = TabsConfig {
        modifier,
        variant,
        initial_active_tab,
        indicator_color: indicator_color.unwrap_or(scheme.primary),
        container_color: container_color.unwrap_or(scheme.surface),
        active_content_color: active_content_color.unwrap_or(scheme.primary),
        inactive_content_color: inactive_content_color.unwrap_or(scheme.on_surface_variant),
        indicator_height: indicator_height.unwrap_or(TabsDefaults::INDICATOR_HEIGHT),
        indicator_min_width: indicator_min_width.unwrap_or(TabsDefaults::INDICATOR_MIN_WIDTH),
        indicator_max_width,
        min_tab_height: min_tab_height.unwrap_or(TabsDefaults::MIN_TAB_HEIGHT),
        tab_padding: tab_padding.unwrap_or(TabsDefaults::TAB_PADDING),
        enabled: enabled.unwrap_or(true),
        disabled_content_color: disabled_content_color
            .unwrap_or_else(|| TabsDefaults::disabled_content_color(&scheme)),
        divider_color: divider_color.unwrap_or(scheme.surface_variant),
        scrollable,
        edge_padding: edge_padding.unwrap_or(TabsDefaults::SCROLLABLE_EDGE_PADDING),
        min_scrollable_tab_width: min_scrollable_tab_width
            .unwrap_or(TabsDefaults::SCROLLABLE_MIN_TAB_WIDTH),
        items: items.clone(),
    };
    let controller = remember(|| TabsController::new(config.initial_active_tab));
    render_tabs(config, controller);
}

fn render_tabs(args: TabsConfig, controller: State<TabsController>) {
    let tabs = args.items.clone();

    let num_tabs = tabs.len();
    if num_tabs == 0 {
        return;
    }
    let active_tab = controller
        .with(|c| c.active_tab())
        .min(num_tabs.saturating_sub(1));

    let (title_closures, content_closures): (Vec<_>, Vec<_>) =
        tabs.into_iter().map(|def| (def.title, def.content)).unzip();
    let tab_focus_requesters = remember(Vec::<FocusRequester>::new);
    let tab_focus_requesters = tab_focus_requesters.with_mut(|requesters| {
        requesters.resize_with(num_tabs, FocusRequester::new);
        requesters.truncate(num_tabs);
        requesters.clone()
    });

    let indicator_shape = match args.variant {
        TabsVariant::Primary => Shape::rounded_rectangle(Dp(3.0)),
        TabsVariant::Secondary => Shape::RECTANGLE,
    };

    let ripple_color = TabsDefaults::ripple_color(args.active_content_color);

    if controller.with(|c| c.has_pending_animation_frame()) {
        receive_frame_nanos(move |frame_nanos| {
            let has_pending_animation_frame = controller.with_mut(|controller| {
                controller.advance_from_frame_nanos(frame_nanos);
                controller.has_pending_animation_frame()
            });
            if has_pending_animation_frame {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }
    let scroll_offset = controller.with(|c| c.content_scroll_px());
    let tab_row_scroll_px = controller.with(|c| c.tab_row_scroll_px());
    let indicator_x_px = controller.with(|c| c.indicator_x_px());
    let indicator_width_px = controller.with(|c| c.indicator_width_px());

    let tab_row_scroll_recognizer =
        remember(|| ScrollRecognizer::new(ScrollSettings { consume: true }));
    let layout_args = args.clone();
    let modifier = with_pointer_input(
        Modifier::new().semantics(SemanticsArgs {
            role: Some(Role::TabList),
            ..Default::default()
        }),
        move |input| {
            let is_scrollable =
                args.scrollable || controller.with(|c| c.tab_row_scroll_max() > Px(0));
            if is_scrollable {
                let cursor_in_tab_bar = if let Some(pos) = input.cursor_position_rel {
                    let within_x = pos.x.0 >= 0 && pos.x.0 < input.computed_data.width.0;
                    let within_y =
                        pos.y.0 >= 0 && pos.y.0 < controller.with(|c| c.tab_bar_height()).0;
                    within_x && within_y
                } else {
                    false
                };

                if cursor_in_tab_bar {
                    let scroll_result = tab_row_scroll_recognizer.with_mut(|recognizer| {
                        recognizer.update(input.pass, input.pointer_changes.as_mut_slice())
                    });
                    let delta = if scroll_result.delta_x.abs() >= 0.01 {
                        scroll_result.delta_x
                    } else {
                        scroll_result.delta_y
                    };
                    if delta.abs() >= 0.01 {
                        controller.with_mut(|c| {
                            let current = c.tab_row_scroll_offset.target;
                            let max = c.tab_row_scroll_max().to_f32();
                            let next = (current - delta).clamp(0.0, max);
                            c.set_tab_row_scroll_immediate(Px::saturating_from_f32(next));
                        });
                    }
                }
            }
        },
    );

    let policy = TabsLayout {
        args: layout_args,
        num_tabs,
        active_tab,
        controller,
        tab_row_scroll_px,
        indicator_x_px,
        indicator_width_px,
    };
    layout_primitive()
        .modifier(modifier)
        .layout_policy(policy.clone())
        .render_policy(policy)
        .child(move || {
            let content_closures = content_closures.clone();
            surface()
                .style(args.container_color.into())
                .modifier(Modifier::new().fill_max_size())
                .shape(Shape::RECTANGLE)
                .with_child(|| {});

            surface()
                .style(args.divider_color.into())
                .modifier(Modifier::new().fill_max_size())
                .shape(Shape::RECTANGLE)
                .with_child(|| {});

            surface()
                .style(args.indicator_color.into())
                .modifier(Modifier::new().fill_max_size())
                .shape(indicator_shape)
                .with_child(|| {});

            for (index, child) in title_closures.iter().cloned().enumerate() {
                let label_color = if !args.enabled {
                    args.disabled_content_color
                } else if index == active_tab {
                    args.active_content_color
                } else {
                    args.inactive_content_color
                };

                let tab_height = match &child {
                    TabTitle::Label {
                        text,
                        icon: Some(_),
                    } if !text.is_empty() => TabsDefaults::LARGE_TAB_HEIGHT,
                    _ => args.min_tab_height,
                };
                let focus_requester = tab_focus_requesters[index];
                let previous_focus_requester =
                    tab_focus_requesters[(index + num_tabs.saturating_sub(1)) % num_tabs];
                let next_focus_requester = tab_focus_requesters[(index + 1) % num_tabs];

                tab_trigger(TabTriggerArgs {
                    controller,
                    title: child,
                    enabled: args.enabled,
                    index,
                    focus_requester,
                    previous_focus_requester,
                    next_focus_requester,
                    label_color,
                    ripple_color,
                    tab_height,
                    tab_padding: args.tab_padding,
                });
            }

            render_tabs_content_container(scroll_offset, content_closures);
        });
}

struct TabTriggerArgs {
    controller: State<TabsController>,
    title: TabTitle,
    enabled: bool,
    index: usize,
    focus_requester: FocusRequester,
    previous_focus_requester: FocusRequester,
    next_focus_requester: FocusRequester,
    label_color: Color,
    ripple_color: Color,
    tab_height: Dp,
    tab_padding: Dp,
}

fn tab_trigger(args: TabTriggerArgs) {
    let tab_modifier = Modifier::new()
        .constrain(None, Some(DimensionValue::Fixed(args.tab_height.into())))
        .focus_group()
        .on_focus_changed(move |focus_state: FocusState| {
            if !focus_state.has_focus() {
                return;
            }

            let should_select = args
                .controller
                .with(|state| state.active_tab() != args.index);
            if should_select {
                args.controller
                    .with_mut(|state| state.set_active_tab(args.index));
            }
        });

    let accessibility_label = match &args.title {
        TabTitle::Label { text, .. } => Some(text.clone()),
        _ => None,
    };

    match (args.enabled, accessibility_label) {
        (true, Some(label)) => {
            surface()
                .style(Color::TRANSPARENT.into())
                .content_alignment(Alignment::Center)
                .content_color(args.label_color)
                .modifier(tab_modifier)
                .ripple_color(args.ripple_color)
                .shape(Shape::RECTANGLE)
                .enabled(true)
                .focus_requester(args.focus_requester)
                .accessibility_role(tessera_ui::accesskit::Role::Tab)
                .accessibility_focusable(true)
                .accessibility_label(label)
                .focus_properties(
                    FocusProperties::new()
                        .left(args.previous_focus_requester)
                        .right(args.next_focus_requester),
                )
                .on_click(move || {
                    args.controller
                        .with_mut(|state| state.set_active_tab(args.index));
                })
                .with_child(move || {
                    render_tab_title(args.title.clone(), args.label_color, args.tab_padding);
                });
        }
        (true, None) => {
            surface()
                .style(Color::TRANSPARENT.into())
                .content_alignment(Alignment::Center)
                .content_color(args.label_color)
                .modifier(tab_modifier)
                .ripple_color(args.ripple_color)
                .shape(Shape::RECTANGLE)
                .enabled(true)
                .focus_requester(args.focus_requester)
                .accessibility_role(tessera_ui::accesskit::Role::Tab)
                .accessibility_focusable(true)
                .focus_properties(
                    FocusProperties::new()
                        .left(args.previous_focus_requester)
                        .right(args.next_focus_requester),
                )
                .on_click(move || {
                    args.controller
                        .with_mut(|state| state.set_active_tab(args.index));
                })
                .with_child(move || {
                    render_tab_title(args.title.clone(), args.label_color, args.tab_padding);
                });
        }
        (false, Some(label)) => {
            surface()
                .style(Color::TRANSPARENT.into())
                .content_alignment(Alignment::Center)
                .content_color(args.label_color)
                .modifier(tab_modifier)
                .ripple_color(args.ripple_color)
                .shape(Shape::RECTANGLE)
                .enabled(false)
                .focus_requester(args.focus_requester)
                .accessibility_role(tessera_ui::accesskit::Role::Tab)
                .accessibility_focusable(true)
                .accessibility_label(label)
                .with_child(move || {
                    render_tab_title(args.title.clone(), args.label_color, args.tab_padding);
                });
        }
        (false, None) => {
            surface()
                .style(Color::TRANSPARENT.into())
                .content_alignment(Alignment::Center)
                .content_color(args.label_color)
                .modifier(tab_modifier)
                .ripple_color(args.ripple_color)
                .shape(Shape::RECTANGLE)
                .enabled(false)
                .focus_requester(args.focus_requester)
                .accessibility_role(tessera_ui::accesskit::Role::Tab)
                .accessibility_focusable(true)
                .with_child(move || {
                    render_tab_title(args.title.clone(), args.label_color, args.tab_padding);
                });
        }
    }
}

fn render_tab_title(title: TabTitle, label_color: Color, tab_padding: Dp) {
    match title {
        TabTitle::Custom(render) => render.render(),
        TabTitle::Themed(render) => render.call(label_color),
        TabTitle::Label {
            text,
            icon: Some(icon),
        } => {
            tab_label()
                .text(text)
                .horizontal_text_padding(tab_padding)
                .icon(icon);
        }
        TabTitle::Label { text, icon: None } => {
            tab_label().text(text).horizontal_text_padding(tab_padding);
        }
    }
}
