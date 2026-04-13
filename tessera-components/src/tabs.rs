//! A component for creating a tab-based layout.
//!
//! ## Usage
//!
//! Use to organize related destinations into a horizontal tab strip.
use std::sync::Arc;

use parking_lot::Mutex;
use tessera_foundation::gesture::{ScrollRecognizer, ScrollSettings};
use tessera_ui::{
    AxisConstraint, Color, ComputedData, Constraint, Dp, FocusRequester, FocusState, LayoutResult,
    MeasurementError, Modifier, Px, PxPosition, RenderSlot, State,
    accesskit::Role,
    layout::{LayoutPolicy, MeasureScope, RenderInput, RenderPolicy, layout},
    modifier::FocusModifierExt as _,
    provide_context, receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::boxed,
    column::column,
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

fn clamp_px(value: Px, min: Px, max: Option<Px>) -> Px {
    let clamped_max = max.unwrap_or(value);
    Px(value.0.max(min.0).min(clamped_max.0))
}

/// Controller for the `tabs` component.
///
/// Tracks the active tab index and cached values used to animate the indicator
/// and tab-row scrolling.
#[derive(Clone, PartialEq)]
pub struct TabsController {
    active_tab: usize,
    indicator_x: Spring1D,
    indicator_width: Spring1D,
    tab_row_scroll_offset: Spring1D,
    tab_row_scroll_max: Px,
    tab_row_scroll_user_overridden: bool,
    tab_bar_height: Px,
    last_frame_nanos: Option<u64>,
    indicator_initialized: bool,
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
            tab_row_scroll_offset: Spring1D::new(0.0),
            tab_row_scroll_max: Px(0),
            tab_row_scroll_user_overridden: false,
            tab_bar_height: Px(0),
            last_frame_nanos: None,
            indicator_initialized: false,
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
            || self.tab_row_scroll_offset.is_animating()
    }
}

impl Default for TabsController {
    fn default() -> Self {
        Self::new(0)
    }
}

#[derive(Clone, PartialEq)]
struct TabsConfig {
    modifier: Modifier,
    variant: TabsVariant,
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
}

#[derive(Clone)]
struct TabsCompositionContext {
    controller: State<TabsController>,
    active_tab: usize,
    enabled: bool,
    active_content_color: Color,
    inactive_content_color: Color,
    disabled_content_color: Color,
    ripple_color: Color,
    min_tab_height: Dp,
    tab_padding: Dp,
    next_index: Arc<Mutex<usize>>,
}

#[derive(Clone, PartialEq)]
enum TabTitle {
    Custom(RenderSlot),
    Label {
        text: String,
        icon: Option<IconContent>,
    },
}

impl TabBuilder {
    /// Set a custom title slot for the tab and clear built-in label state.
    pub fn title<F>(mut self, title: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.props.title_slot = Some(RenderSlot::new(title));
        self.props.label_text = None;
        self.props.label_icon = None;
        self
    }

    /// Set a shared custom title slot for the tab and clear built-in label
    /// state.
    pub fn title_shared(mut self, title: impl Into<RenderSlot>) -> Self {
        self.props.title_slot = Some(title.into());
        self.props.label_text = None;
        self.props.label_icon = None;
        self
    }

    /// Set the built-in Material text label for the tab and clear any custom
    /// title slot.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.props.title_slot = None;
        self.props.label_text = Some(label.into());
        self
    }

    /// Set the built-in Material icon for the tab and clear any custom title
    /// slot.
    pub fn icon(mut self, icon: impl Into<IconContent>) -> Self {
        self.props.title_slot = None;
        self.props.label_icon = Some(icon.into());
        self
    }

    /// Set the built-in Material label and icon for the tab.
    pub fn label_with_icon(self, label: impl Into<String>, icon: impl Into<IconContent>) -> Self {
        self.label(label).icon(icon)
    }
}

/// # tab
///
/// Renders a single tab inside [`tabs`].
///
/// ## Usage
///
/// Use inside a `tabs().content(...)` slot to declare one tab-strip item.
///
/// ## Parameters
///
/// - `title` — custom title slot rendered in the tab row.
/// - `label` — built-in Material text label for the tab.
/// - `icon` — built-in Material icon for the tab.
///
/// ## Examples
///
/// ```
/// use tessera_components::tabs::{tab, tabs};
/// use tessera_components::text::text;
/// use tessera_ui::tessera;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// #[tessera]
/// fn demo() {
/// #     material_theme()
/// #         .theme(|| MaterialTheme::default())
/// #         .child(|| {
///     tabs().content(|| {
///         tab().label("Home");
///     });
/// #         });
/// }
/// ```
#[tessera]
pub fn tab(
    #[prop(skip_setter)] title_slot: Option<RenderSlot>,
    #[prop(skip_setter)] label_text: Option<String>,
    #[prop(skip_setter)] label_icon: Option<IconContent>,
) {
    let composition = use_context::<TabsCompositionContext>()
        .expect("tab must be used inside tabs")
        .get();
    let index = {
        let mut next_index = composition.next_index.lock();
        let index = *next_index;
        *next_index += 1;
        index
    };
    let title = if let Some(title_slot) = title_slot {
        TabTitle::Custom(title_slot)
    } else if label_text.is_some() || label_icon.is_some() {
        TabTitle::Label {
            text: label_text.unwrap_or_default(),
            icon: label_icon,
        }
    } else {
        panic!("tab requires title(), label(), icon(), or label_with_icon()");
    };
    let accessibility_label = match &title {
        TabTitle::Label { text, .. } => Some(text.clone()),
        _ => None,
    };
    let label_color = if !composition.enabled {
        composition.disabled_content_color
    } else if index == composition.active_tab {
        composition.active_content_color
    } else {
        composition.inactive_content_color
    };
    let tab_height = match &title {
        TabTitle::Label {
            text,
            icon: Some(_),
        } if !text.is_empty() => TabsDefaults::LARGE_TAB_HEIGHT,
        _ => composition.min_tab_height,
    };
    let focus_requester = remember(FocusRequester::new).get();

    tab_trigger(TabTriggerArgs {
        controller: composition.controller,
        title,
        enabled: composition.enabled,
        index,
        focus_requester,
        label_color,
        ripple_color: composition.ripple_color,
        tab_height,
        tab_padding: composition.tab_padding,
        accessibility_label,
    });
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
    #[prop(into)] icon: Option<IconContent>,
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
            Some(AxisConstraint::NONE),
            Some(AxisConstraint::exact(container_height.into())),
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
                        .modifier(Modifier::new())
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
                                    Some(AxisConstraint::exact(Px(0))),
                                    Some(AxisConstraint::exact(Dp(2.0).into())),
                                ));
                            };
                            let text_content = text_content.clone();
                            {
                                text_component()
                                    .content(text_content.clone())
                                    .color(content_color)
                                    .style(style);
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
                        .style(style);
                }
            };
        });
}

#[derive(Clone)]
struct TabsLayout {
    args: TabsConfig,
    controller: State<TabsController>,
    tab_row_scroll_px: Px,
    indicator_x_px: Px,
    indicator_width_px: Px,
}

impl PartialEq for TabsLayout {
    fn eq(&self, other: &Self) -> bool {
        self.tab_row_scroll_px == other.tab_row_scroll_px
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
    fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        let parent_constraint = *input.parent_constraint().as_ref();

        let container = children[0];
        let divider = children[1];
        let indicator = children[2];
        let title_ids = &children[3..];
        let num_tabs = title_ids.len();
        let active_tab = self
            .controller
            .with(|controller| controller.active_tab())
            .min(num_tabs.saturating_sub(1));

        let horizontal_padding = self.args.tab_padding.to_px().to_f32() * 2.0;
        let indicator_min_width: Px = self.args.indicator_min_width.into();
        let available_width = parent_constraint.width.resolve_max();

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
        ): (Px, Px, Vec<Px>, Vec<Px>, Vec<Px>, Px, Px) = if !is_scrollable {
            let final_width = available_width.unwrap_or(Px(0));
            let tab_width = if num_tabs == 0 {
                Px(0)
            } else {
                Px(final_width.0 / num_tabs as i32)
            };

            let mut titles_max_height = Px(0);
            for &title_id in title_ids {
                let result = title_id.measure_untracked(&Constraint::new(
                    AxisConstraint::exact(tab_width),
                    AxisConstraint::NONE,
                ))?;
                titles_max_height = titles_max_height.max(result.height);
            }

            let intrinsic_results: Vec<_> = title_ids
                .iter()
                .map(|&id| {
                    id.measure_untracked(&Constraint::new(
                        AxisConstraint::at_most(tab_width),
                        AxisConstraint::exact(titles_max_height),
                    ))
                })
                .collect::<Result<_, _>>()?;

            let indicator_widths: Vec<Px> = title_ids
                .iter()
                .enumerate()
                .map(|(idx, _id)| {
                    if match_content_size {
                        let intrinsic_width = intrinsic_results[idx].width.min(tab_width);
                        let content_width =
                            (intrinsic_width.to_f32() - horizontal_padding).max(0.0);
                        Px::saturating_from_f32(content_width).max(indicator_min_width)
                    } else {
                        tab_width
                    }
                })
                .collect();

            let tab_widths: Vec<Px> = vec![tab_width; num_tabs];
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
            let min_tab_width: Px = self.args.min_scrollable_tab_width.into();
            let edge_padding: Px = self.args.edge_padding.into();

            let mut tab_widths = Vec::with_capacity(num_tabs);
            let mut titles_max_height = Px(0);
            for &title_id in title_ids {
                let result = title_id.measure_untracked(&Constraint::new(
                    AxisConstraint::at_least(min_tab_width),
                    AxisConstraint::NONE,
                ))?;
                tab_widths.push(result.width);
                titles_max_height = titles_max_height.max(result.height);
            }

            let intrinsic_results: Vec<_> = title_ids
                .iter()
                .map(|&id| {
                    id.measure_untracked(&Constraint::new(
                        AxisConstraint::NONE,
                        AxisConstraint::exact(titles_max_height),
                    ))
                })
                .collect::<Result<_, _>>()?;

            let mut indicator_widths = Vec::with_capacity(num_tabs);
            for (idx, &_title_id) in title_ids.iter().enumerate() {
                let tab_width = tab_widths.get(idx).copied().unwrap_or(Px(0));
                if match_content_size {
                    let intrinsic_width = intrinsic_results[idx].width.min(tab_width);
                    let content_width = (intrinsic_width.to_f32() - horizontal_padding).max(0.0);
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
            let desired_width = indicator_widths.get(active_tab).copied().unwrap_or(Px(0));
            let clamped_width = clamp_px(
                desired_width,
                self.args.indicator_min_width.into(),
                self.args.indicator_max_width.map(|v| v.into()),
            );

            let tab_left = tab_lefts.get(active_tab).copied().unwrap_or(Px(0));
            let tab_width = tab_widths.get(active_tab).copied().unwrap_or(Px(0));

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
            AxisConstraint::exact(indicator_width),
            AxisConstraint::exact(indicator_height),
        );
        let _ = indicator.measure(&indicator_constraint)?;

        let divider_height: Px = TabsDefaults::DIVIDER_HEIGHT.into();
        let divider_width = if is_scrollable {
            strip_width_total
        } else {
            final_width
        };
        let divider_constraint = Constraint::new(
            AxisConstraint::exact(divider_width),
            AxisConstraint::exact(divider_height),
        );
        let _ = divider.measure(&divider_constraint)?;

        let tab_bar_height = titles_max_height.max(self.args.min_tab_height.into());
        let should_update_tab_bar_height =
            self.controller.with(|c| c.tab_bar_height != tab_bar_height);
        if should_update_tab_bar_height {
            self.controller
                .with_mut(|c| c.set_tab_bar_height(tab_bar_height));
        }
        let final_height = tab_bar_height;
        let title_offset_y = Px((tab_bar_height.0 - titles_max_height.0) / 2).max(Px(0));

        for (idx, &title_id) in title_ids.iter().enumerate() {
            let _ = title_id.measure(&Constraint::new(
                AxisConstraint::exact(tab_widths.get(idx).copied().unwrap_or(Px(0))),
                AxisConstraint::exact(tab_bar_height),
            ))?;
        }

        let container_constraint = Constraint::new(
            AxisConstraint::exact(final_width),
            AxisConstraint::exact(tab_bar_height),
        );
        let _ = container.measure(&container_constraint)?;

        for (i, &title_id) in title_ids.iter().enumerate() {
            let x = tab_lefts.get(i).copied().unwrap_or(Px(0)) - current_scroll_px;
            result.place_child(title_id, PxPosition::new(x, title_offset_y));
        }

        result.place_child(container, PxPosition::new(Px(0), Px(0)));
        result.place_child(
            divider,
            PxPosition::new(
                if is_scrollable {
                    -current_scroll_px
                } else {
                    Px(0)
                },
                tab_bar_height - divider_height,
            ),
        );
        result.place_child(
            indicator,
            PxPosition::new(
                indicator_x - current_scroll_px,
                tab_bar_height - indicator_height,
            ),
        );

        Ok(result.with_size(ComputedData {
            width: final_width,
            height: final_height,
        }))
    }
}

impl RenderPolicy for TabsLayout {
    fn record(&self, input: &mut RenderInput<'_>) {
        input.metadata_mut().set_clips_children(true);
    }
}

/// # tabs
///
/// Renders a Material tab row.
///
/// ## Usage
///
/// Show a row of related destinations and render the selected page outside the
/// tab row.
///
/// ## Parameters
///
/// - `modifier` — modifier chain applied to the tabs subtree.
/// - `variant` — visual variant of the tab row.
/// - `controller` — optional external controller used to read or drive the
///   active tab.
/// - `initial_active_tab` — index of the initially selected tab when no
///   external controller is provided.
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
/// - `content` — tab declarations rendered inside the tab row.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     tabs::{TabsController, tab, tabs},
///     text::text,
/// };
/// use tessera_ui::{LayoutResult, remember, tessera};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// #[tessera]
/// fn demo() {
/// #     material_theme()
/// #         .theme(|| MaterialTheme::default())
/// #         .child(|| {
///     let controller = remember(|| TabsController::new(1));
///     let active_tab = controller.with(|controller| controller.active_tab());
///
///     tabs().controller(controller).content(|| {
///         tab().label("Flights");
///         tab().label("Hotel");
///     });
///
///     if active_tab == 0 {
///         text().content("Content for Flights");
///     } else {
///         text().content("Content for Hotel");
///     }
/// #         });
/// }
/// ```
#[tessera]
pub fn tabs(
    modifier: Modifier,
    variant: TabsVariant,
    controller: Option<State<TabsController>>,
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
    content: RenderSlot,
) {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let config = TabsConfig {
        modifier,
        variant,
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
    };
    let controller =
        controller.unwrap_or_else(|| remember(|| TabsController::new(initial_active_tab)));
    render_tabs(config, controller, content);
}

fn render_tabs(args: TabsConfig, controller: State<TabsController>, content: RenderSlot) {
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
        controller,
        tab_row_scroll_px,
        indicator_x_px,
        indicator_width_px,
    };
    layout()
        .modifier(modifier)
        .layout_policy(policy.clone())
        .render_policy(policy)
        .child(move || {
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

            provide_context(
                || TabsCompositionContext {
                    controller,
                    active_tab: controller.with(|controller| controller.active_tab()),
                    enabled: args.enabled,
                    active_content_color: args.active_content_color,
                    inactive_content_color: args.inactive_content_color,
                    disabled_content_color: args.disabled_content_color,
                    ripple_color,
                    min_tab_height: args.min_tab_height,
                    tab_padding: args.tab_padding,
                    next_index: Arc::new(Mutex::new(0)),
                },
                move || {
                    content.render();
                },
            );
        });
}

struct TabTriggerArgs {
    controller: State<TabsController>,
    title: TabTitle,
    enabled: bool,
    index: usize,
    focus_requester: FocusRequester,
    label_color: Color,
    ripple_color: Color,
    tab_height: Dp,
    tab_padding: Dp,
    accessibility_label: Option<String>,
}

fn tab_trigger(args: TabTriggerArgs) {
    let tab_modifier = Modifier::new()
        .constrain(None, Some(AxisConstraint::exact(args.tab_height.into())))
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

    match (args.enabled, args.accessibility_label.clone()) {
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
                .on_click(move || {
                    args.controller
                        .with_mut(|state| state.set_active_tab(args.index));
                })
                .with_child(move || {
                    render_tab_title(args.title.clone(), args.tab_padding);
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
                .on_click(move || {
                    args.controller
                        .with_mut(|state| state.set_active_tab(args.index));
                })
                .with_child(move || {
                    render_tab_title(args.title.clone(), args.tab_padding);
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
                    render_tab_title(args.title.clone(), args.tab_padding);
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
                    render_tab_title(args.title.clone(), args.tab_padding);
                });
        }
    }
}

fn render_tab_title(title: TabTitle, tab_padding: Dp) {
    match title {
        TabTitle::Custom(render) => render.render(),
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
