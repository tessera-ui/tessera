//! A component for creating a tab-based layout.
//!
//! ## Usage
//!
//! Use to organize related destinations into a horizontal tab strip.
use std::{any::TypeId, sync::Arc};

use tessera_foundation::gesture::{ScrollRecognizer, ScrollSettings};
use tessera_ui::{
    AxisConstraint, Callback, Color, ComputedData, Constraint, Dp, FocusManager, FocusProperties,
    FocusRequester, FocusState, LayoutResult, MeasurementError, Modifier, ParentDataMap,
    ParentDataModifierNode, Px, PxPosition, RenderSlot, State,
    accesskit::Role,
    layout::{MeasureScope, layout},
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    provide_context, receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    animation::Animatable,
    boxed::boxed,
    column::column,
    icon::icon as icon_component,
    interaction_state::InteractionState,
    modifier::{ModifierExt, SemanticsArgs, with_pointer_input},
    painter::Painter,
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    surface::surface,
    text::text as text_component,
    theme::{ContentColor, MaterialAlpha, MaterialColorScheme, MaterialTheme},
};

use navigation::{TabFocusData, TabListSemantics, TabNavigation};

const DEFAULT_SPATIAL_DAMPING_RATIO: f32 = 0.9;
const DEFAULT_SPATIAL_STIFFNESS: f32 = 700.0;

mod navigation;

#[cfg(test)]
mod accessibility_tests;

/// Visual variants supported by [`tabs`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabsVariant {
    /// Primary tabs.
    #[default]
    Primary,
    /// Secondary tabs.
    Secondary,
}

/// Keyboard activation behavior for [`tabs`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabActivationMode {
    /// Move focus with arrows and select with Enter or Space.
    #[default]
    Manual,
    /// Select the destination when an arrow key moves focus to it.
    Automatic,
}

/// Material Design 3 defaults for [`tabs`].
pub struct TabsDefaults;

impl TabsDefaults {
    /// Default indicator height.
    pub const INDICATOR_HEIGHT: Dp = Dp(3.0);
    /// Default secondary indicator height.
    pub const SECONDARY_INDICATOR_HEIGHT: Dp = Dp(2.0);
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
    pub const LARGE_TAB_HEIGHT: Dp = Dp(64.0);

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
        let stiffness = stiffness.max(0.0);
        if stiffness == 0.0 {
            self.snap_to(self.target);
            return;
        }

        let mut motion = Animatable::spring(self.value, stiffness.sqrt(), damping_ratio)
            .with_velocity(self.velocity);
        motion.set_target(self.target);
        self.value = motion.tick(dt);
        self.velocity = motion.velocity();

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

/// Internal runtime state for the tab row.
///
/// Holds the animation springs plus a cached copy of the externally owned
/// selected index. The selected index itself lives in caller-owned `State`;
/// this struct only mirrors it to drive the indicator and scroll animation.
#[derive(Clone, PartialEq)]
struct TabsController {
    density: f64,
    targets: Vec<TabFocusData>,
    focused_requester: Option<FocusRequester>,
    pending_focus: Option<FocusRequester>,
    activation_mode: TabActivationMode,
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
    fn new(initial_tab: usize) -> Self {
        Self {
            density: tessera_ui::dp::scale_factor(),
            targets: Vec::new(),
            focused_requester: None,
            pending_focus: None,
            activation_mode: TabActivationMode::Manual,
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

    /// Mirror the caller-owned selected index into the animation state.
    ///
    /// Only subsequent changes retarget the indicator/scroll springs;
    /// identical values are no-ops.
    fn sync_selected(&mut self, index: usize) {
        if index != self.active_tab {
            self.active_tab = index;
            self.tab_row_scroll_user_overridden = false;
            self.pending_retarget_frame = true;
            self.last_frame_nanos = None;
        }
    }

    fn update_density(&mut self, density: f64) {
        let ratio = density / self.density;
        for spring in [
            &mut self.indicator_x,
            &mut self.indicator_width,
            &mut self.tab_row_scroll_offset,
        ] {
            spring.value = (f64::from(spring.value) * ratio) as f32;
            spring.target = (f64::from(spring.target) * ratio) as f32;
            spring.velocity = (f64::from(spring.velocity) * ratio) as f32;
        }
        self.tab_row_scroll_max = Px((f64::from(self.tab_row_scroll_max.0) * ratio) as i32);
        self.tab_bar_height = Px((f64::from(self.tab_bar_height.0) * ratio) as i32);
        self.density = density;
    }

    fn active_tab(&self) -> usize {
        self.active_tab
    }

    fn set_focused_requester(&mut self, requester: Option<FocusRequester>) {
        self.focused_requester = requester;
        self.tab_row_scroll_user_overridden = requester.is_none();
    }

    fn sync_targets(&mut self, targets: Vec<TabFocusData>) {
        if self.focused_requester.is_some_and(|requester| {
            !targets
                .iter()
                .any(|target| target.requester == requester && target.enabled)
        }) {
            let fallback = targets
                .iter()
                .find(|target| target.selected && target.enabled)
                .or_else(|| targets.iter().find(|target| target.enabled))
                .map(|target| target.requester);
            self.focused_requester = fallback;
            self.pending_focus = fallback;
        }
        self.targets = targets;
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
    selection: State<usize>,
    modifier: Modifier,
    variant: TabsVariant,
    indicator_color: Color,
    container_color: Color,
    active_content_color: Color,
    inactive_content_color: Color,
    inactive_interaction_color: Color,
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

#[derive(Clone, Copy, PartialEq)]
struct TabsCompositionContext {
    controller: State<TabsController>,
    selected: State<usize>,
    enabled: bool,
    active_content_color: Color,
    inactive_content_color: Color,
    inactive_interaction_color: Color,
    disabled_content_color: Color,
    ripple_color: Color,
    min_tab_height: Dp,
    tab_padding: Dp,
}

#[derive(Clone, PartialEq)]
enum TabTitle {
    Custom(RenderSlot),
    Label { text: String, icon: Option<Painter> },
}

#[derive(Clone, Copy, PartialEq)]
struct TabIndicatorPadding(Dp);

impl ParentDataModifierNode for TabIndicatorPadding {
    fn apply_parent_data(&self, data: &mut ParentDataMap) {
        data.insert(TypeId::of::<Self>(), Arc::new(*self));
    }
}

#[derive(Clone, Copy, PartialEq)]
struct TabActivation {
    selection: State<usize>,
    controlled: bool,
    on_click: Option<Callback>,
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
    pub fn icon(mut self, icon: impl Into<Painter>) -> Self {
        self.props.title_slot = None;
        self.props.label_icon = Some(icon.into());
        self
    }

    /// Set the built-in Material label and icon for the tab.
    pub fn label_with_icon(self, label: impl Into<String>, icon: impl Into<Painter>) -> Self {
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
/// - `selected` — optional controlled selection; otherwise follows the tab row.
/// - `on_click` — callback invoked when the tab is activated.
/// - `enabled` — whether this tab can be focused or activated.
/// - `title` — custom title slot; primary indicators match its intrinsic width.
///   The slot owns its content padding.
/// - `label` — built-in Material text label for the tab.
/// - `icon` — built-in Material icon for the tab.
/// - `accessibility_label` — accessible name override. Authors must provide a
///   nonempty name for icon-only tabs and non-text custom titles.
///
/// ## Examples
///
/// ```
/// use tessera_components::tabs::{tab, tabs};
/// use tessera_components::text::text;
/// use tessera_ui::{remember, tessera};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// #[tessera]
/// fn demo() {
/// #     material_theme()
/// #         .theme(|| MaterialTheme::default())
/// #         .child(|| {
///     let selected = remember(|| 0usize);
///     tabs().selected(selected).content(move || {
///         tab()
///             .label("Home")
///             .selected(selected.get() == 0)
///             .on_click(move || selected.set(0));
///     });
/// #         });
/// }
/// ```
#[tessera]
pub fn tab(
    selected: Option<bool>,
    enabled: Option<bool>,
    #[prop(skip_setter)] title_slot: Option<RenderSlot>,
    #[prop(skip_setter)] label_text: Option<String>,
    #[prop(skip_setter)] label_icon: Option<Painter>,
    on_click: Option<Callback>,
    #[prop(into)] accessibility_label: Option<String>,
) {
    let composition = use_context::<TabsCompositionContext>()
        .expect("tab must be used inside tabs")
        .get();
    let focus_requester = remember(FocusRequester::new).get();
    let controller = composition.controller;
    let selected_state = composition.selected;
    let controlled = selected.is_some();
    let activation = TabActivation {
        selection: selected_state,
        controlled,
        on_click,
    };
    let live_activation = remember(|| activation);
    if live_activation.with(|value| value != &activation) {
        live_activation.set(activation);
    }
    let selected = selected.unwrap_or_else(|| {
        controller.with(|controller| {
            controller
                .targets
                .iter()
                .position(|target| target.requester == focus_requester)
                == Some(controller.active_tab())
        })
    });
    let enabled = composition.enabled && enabled.unwrap_or(true);
    let interaction = remember(InteractionState::new);
    let interacting = interaction.with(|interaction| {
        interaction.is_hovered() || interaction.is_focused() || interaction.is_pressed()
    });
    let activate = Callback::new(move || {
        let index = controller.with(|controller| {
            controller
                .targets
                .iter()
                .position(|target| target.requester == focus_requester && target.enabled)
        });
        if let Some(index) = index {
            let activation = live_activation.get();
            if !activation.controlled && activation.selection.get() != index {
                activation.selection.set(index);
            }
            if let Some(on_click) = activation.on_click {
                on_click.call();
            }
            FocusManager::current().defer_keyboard_until_layout();
        }
    });
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
    let accessibility_label = accessibility_label.or_else(|| match &title {
        TabTitle::Label { text, .. } if !text.trim().is_empty() => Some(text.clone()),
        _ => None,
    });
    let label_color = if !enabled {
        composition.disabled_content_color
    } else if selected {
        composition.active_content_color
    } else if interacting {
        composition.inactive_interaction_color
    } else {
        composition.inactive_content_color
    };
    let tab_height = match &title {
        TabTitle::Label {
            text,
            icon: Some(_),
        } if !text.is_empty() => Dp(TabsDefaults::LARGE_TAB_HEIGHT
            .0
            .max(composition.min_tab_height.0)),
        _ => composition.min_tab_height,
    };
    tab_trigger().args(TabTriggerArgs {
        controller,
        interaction,
        title,
        enabled,
        selected,
        on_click: Some(activate),
        focus_requester,
        label_color,
        ripple_color: if selected {
            composition.ripple_color
        } else {
            composition.inactive_interaction_color
        },
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
    #[prop(into)] text: Option<String>,
    #[prop(into)] icon: Option<Painter>,
    horizontal_text_padding: Option<Dp>,
    icon_size: Option<Dp>,
) {
    let text = text.unwrap_or_default();
    let has_icon = icon.is_some();
    let has_text = !text.is_empty();
    let icon_size = icon_size.unwrap_or(Dp(24.0));
    let horizontal_padding = horizontal_text_padding.unwrap_or(TabsDefaults::TAB_PADDING);
    let current = TabLabelContent {
        text,
        icon,
        icon_size,
    };
    let live = remember(|| current.clone());
    if live.with(|value| value != &current) {
        live.set(current);
    }
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
            let current = live.get();
            let has_icon = current.icon.is_some();
            let has_text = !current.text.is_empty();
            let content_color = use_context::<ContentColor>()
                .map(|color| color.get().current)
                .unwrap_or_else(|| ContentColor::default().current);
            let style = use_context::<MaterialTheme>()
                .expect("MaterialTheme must be provided")
                .get()
                .typography
                .title_small;
            {
                if has_icon && has_text {
                    column()
                        .main_axis_alignment(MainAxisAlignment::Center)
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .modifier(Modifier::new())
                        .children(move || {
                            let current = live.get();
                            let content_color = use_context::<ContentColor>()
                                .map(|color| color.get().current)
                                .unwrap_or_else(|| ContentColor::default().current);
                            let style = use_context::<MaterialTheme>()
                                .expect("MaterialTheme must be provided")
                                .get()
                                .typography
                                .title_small;
                            {
                                if let Some(ic) = current.icon {
                                    icon_component()
                                        .painter(ic)
                                        .size(current.icon_size)
                                        .tint(content_color);
                                }
                            };
                            {
                                spacer().modifier(Modifier::new().constrain(
                                    Some(AxisConstraint::exact(Px(0))),
                                    Some(AxisConstraint::exact(Dp(2.0).into())),
                                ));
                            };
                            {
                                text_component()
                                    .content(current.text)
                                    .color(content_color)
                                    .style(style);
                            };
                        });
                } else if has_icon {
                    if let Some(ic) = current.icon {
                        icon_component()
                            .painter(ic)
                            .size(current.icon_size)
                            .tint(content_color);
                    }
                } else if has_text {
                    text_component()
                        .content(current.text)
                        .color(content_color)
                        .style(style);
                }
            };
        });
}

#[derive(Clone, PartialEq)]
struct TabLabelContent {
    text: String,
    icon: Option<Painter>,
    icon_size: Dp,
}

#[derive(Clone)]
struct TabsLayout {
    args: TabsConfig,
    controller: State<TabsController>,
    tab_row_scroll_px: Px,
    indicator_x_px: Px,
    indicator_width_px: Px,
    focused_requester: Option<FocusRequester>,
    active_tab: usize,
    scroll_user_overridden: bool,
}

impl PartialEq for TabsLayout {
    fn eq(&self, other: &Self) -> bool {
        self.tab_row_scroll_px == other.tab_row_scroll_px
            && self.focused_requester == other.focused_requester
            && self.active_tab == other.active_tab
            && self.scroll_user_overridden == other.scroll_user_overridden
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

impl tessera_ui::LayoutModifierNode for TabsLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        let parent_constraint = *input.parent_constraint().as_ref();

        let container = children[0];
        let divider = children[1];
        let indicator = children[2];
        let title_ids = &children[3..];
        let num_tabs = title_ids.len();
        let targets = title_ids
            .iter()
            .filter_map(|child| child.parent_data::<TabFocusData>(input))
            .collect::<Vec<_>>();
        if self
            .controller
            .with(|controller| controller.targets != targets)
        {
            self.controller
                .with_mut(|controller| controller.sync_targets(targets));
        }
        let active_tab = self
            .controller
            .with(|controller| controller.active_tab())
            .min(num_tabs.saturating_sub(1));
        let reveal_tab = self
            .controller
            .with(|controller| {
                controller.focused_requester.and_then(|requester| {
                    controller
                        .targets
                        .iter()
                        .position(|target| target.requester == requester)
                })
            })
            .unwrap_or(active_tab);

        let title_padding = title_ids
            .iter()
            .map(|title| {
                title
                    .parent_data::<TabIndicatorPadding>(input)
                    .map_or(0.0, |padding| padding.0.to_px().to_f32() * 2.0)
            })
            .collect::<Vec<_>>();
        let indicator_min_width: Px = self.args.indicator_min_width.into();
        let available_width = parent_constraint.width.resolve_max();

        let is_scrollable = self.args.scrollable
            || available_width.is_none()
            || available_width.is_some_and(|width| {
                width.0
                    < TabsDefaults::MIN_TAB_HEIGHT
                        .to_px()
                        .0
                        .saturating_mul(num_tabs as i32)
            });
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
            let remainder = if num_tabs == 0 {
                0
            } else {
                final_width.0 % num_tabs as i32
            };
            let tab_widths = (0..num_tabs)
                .map(|index| tab_width + Px(i32::from((index as i32) < remainder)))
                .collect::<Vec<_>>();

            let mut titles_max_height = Px(0);
            for (index, &title_id) in title_ids.iter().enumerate() {
                let result = title_id.probe(
                    input,
                    &Constraint::new(
                        AxisConstraint::exact(tab_widths[index]),
                        AxisConstraint::NONE,
                    ),
                )?;
                titles_max_height = titles_max_height.max(result.height);
            }

            let intrinsic_results: Vec<_> = title_ids
                .iter()
                .enumerate()
                .map(|(index, &id)| {
                    id.probe_intrinsic(
                        input,
                        &Constraint::new(
                            AxisConstraint::at_most(tab_widths[index]),
                            AxisConstraint::exact(titles_max_height),
                        ),
                    )
                })
                .collect::<Result<_, _>>()?;

            let indicator_widths: Vec<Px> = title_ids
                .iter()
                .enumerate()
                .map(|(idx, _id)| {
                    if match_content_size {
                        let intrinsic_width = intrinsic_results[idx].width.min(tab_widths[idx]);
                        let content_width =
                            (intrinsic_width.to_f32() - title_padding[idx]).max(0.0);
                        Px::saturating_from_f32(content_width).max(indicator_min_width)
                    } else {
                        tab_widths[idx]
                    }
                })
                .collect();

            let mut left = Px(0);
            let tab_lefts = tab_widths
                .iter()
                .map(|width| {
                    let current = left;
                    left += *width;
                    current
                })
                .collect::<Vec<_>>();

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
                let result = title_id.probe(
                    input,
                    &Constraint::new(
                        AxisConstraint::at_least(min_tab_width),
                        AxisConstraint::NONE,
                    ),
                )?;
                tab_widths.push(result.width);
                titles_max_height = titles_max_height.max(result.height);
            }

            let intrinsic_results: Vec<_> = title_ids
                .iter()
                .map(|&id| {
                    id.probe_intrinsic(
                        input,
                        &Constraint::new(
                            AxisConstraint::NONE,
                            AxisConstraint::exact(titles_max_height),
                        ),
                    )
                })
                .collect::<Result<_, _>>()?;

            let mut indicator_widths = Vec::with_capacity(num_tabs);
            for (idx, &_title_id) in title_ids.iter().enumerate() {
                let tab_width = tab_widths.get(idx).copied().unwrap_or(Px(0));
                if match_content_size {
                    let intrinsic_width = intrinsic_results[idx].width.min(tab_width);
                    let content_width = (intrinsic_width.to_f32() - title_padding[idx]).max(0.0);
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

            let selected_left = tab_lefts.get(reveal_tab).copied().unwrap_or(edge_padding);
            let selected_width = tab_widths.get(reveal_tab).copied().unwrap_or(Px(0));
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
            if !is_scrollable
                && self
                    .controller
                    .with(|controller| controller.tab_row_scroll_max != Px(0))
            {
                self.controller
                    .with_mut(|controller| controller.set_tab_row_scroll_bounds(Px(0)));
            }

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
        let _ = indicator.measure(input, &indicator_constraint)?;

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
        let _ = divider.measure(input, &divider_constraint)?;

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
            let _ = title_id.measure(
                input,
                &Constraint::new(
                    AxisConstraint::exact(tab_widths.get(idx).copied().unwrap_or(Px(0))),
                    AxisConstraint::exact(tab_bar_height),
                ),
            )?;
        }

        let container_constraint = Constraint::new(
            AxisConstraint::exact(final_width),
            AxisConstraint::exact(tab_bar_height),
        );
        let _ = container.measure(input, &container_constraint)?;

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

impl tessera_ui::DrawModifierNode for TabsLayout {
    fn draw(
        &self,
        ctx: &mut tessera_ui::DrawModifierContext<'_, '_>,
        content: &mut dyn tessera_ui::DrawModifierContent,
    ) {
        let input = &mut *ctx.render_input;
        input.metadata_mut().set_clips_children(true);
        content.draw(input);
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
/// - `activation_mode` — manual activation by default, or selection with
///   arrows.
/// - `selected` — caller-owned state holding the active tab index. When
///   omitted, the row keeps an internal selection slot and drive is not
///   observable to the caller.
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
/// - `scrollable` — use content widths and horizontal scrolling; narrow fixed
///   rows also scroll when equal tabs cannot retain 48dp targets.
/// - `edge_padding` — optional override for scrollable edge padding.
/// - `min_scrollable_tab_width` — optional override for minimum scrollable tab
///   width.
/// - `content` — tab declarations rendered inside the tab row.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     tabs::{tab, tabs},
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
///     let selected = remember(|| 1usize);
///     let active_tab = selected.get();
///
///     tabs().selected(selected).content(move || {
///         tab()
///             .label("Flights")
///             .selected(selected.get() == 0)
///             .on_click(move || selected.set(0));
///         tab()
///             .label("Hotel")
///             .selected(selected.get() == 1)
///             .on_click(move || selected.set(1));
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
    modifier: Option<Modifier>,
    variant: Option<TabsVariant>,
    selected: Option<State<usize>>,
    activation_mode: Option<TabActivationMode>,
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
    scrollable: Option<bool>,
    edge_padding: Option<Dp>,
    min_scrollable_tab_width: Option<Dp>,
    content: Option<RenderSlot>,
) {
    let selected = if let Some(selected) = selected {
        selected
    } else {
        remember(|| 0usize)
    };
    let modifier = modifier.unwrap_or_default();
    let variant = variant.unwrap_or_default();
    let scrollable = scrollable.unwrap_or(false);
    let content = content.unwrap_or_else(RenderSlot::empty);
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let config = TabsConfig {
        selection: selected,
        modifier,
        variant,
        indicator_color: indicator_color.unwrap_or(scheme.primary),
        container_color: container_color.unwrap_or(scheme.surface),
        active_content_color: active_content_color.unwrap_or(match variant {
            TabsVariant::Primary => scheme.primary,
            TabsVariant::Secondary => scheme.on_surface,
        }),
        inactive_content_color: inactive_content_color.unwrap_or(scheme.on_surface_variant),
        inactive_interaction_color: scheme.on_surface,
        indicator_height: indicator_height.unwrap_or(match variant {
            TabsVariant::Primary => TabsDefaults::INDICATOR_HEIGHT,
            TabsVariant::Secondary => TabsDefaults::SECONDARY_INDICATOR_HEIGHT,
        }),
        indicator_min_width: indicator_min_width.unwrap_or(TabsDefaults::INDICATOR_MIN_WIDTH),
        indicator_max_width,
        min_tab_height: min_tab_height.unwrap_or(TabsDefaults::MIN_TAB_HEIGHT),
        tab_padding: tab_padding.unwrap_or(TabsDefaults::TAB_PADDING),
        enabled: enabled.unwrap_or(true),
        disabled_content_color: disabled_content_color
            .unwrap_or_else(|| TabsDefaults::disabled_content_color(&scheme)),
        divider_color: divider_color.unwrap_or(scheme.outline_variant),
        scrollable,
        edge_padding: edge_padding.unwrap_or(TabsDefaults::SCROLLABLE_EDGE_PADDING),
        min_scrollable_tab_width: min_scrollable_tab_width
            .unwrap_or(TabsDefaults::SCROLLABLE_MIN_TAB_WIDTH),
    };
    // The selected index is owned by the caller; the tab row only mirrors it to
    // drive the indicator and scroll animation. Callers that do not care about
    // reading the selection get an internal slot so the API stays zero-config.
    let controller = remember(|| TabsController::new(0));
    let density = tessera_ui::dp::scale_factor();
    if controller.with(|controller| controller.density != density) {
        controller.with_mut(|controller| controller.update_density(density));
    }
    let activation_mode = activation_mode.unwrap_or_default();
    if controller.with(|controller| controller.activation_mode != activation_mode) {
        controller.with_mut(|controller| controller.activation_mode = activation_mode);
    }
    if controller.with(|controller| controller.pending_focus.is_some()) {
        let requester = controller.with_mut(|controller| controller.pending_focus.take());
        if let Some(requester) = requester {
            requester.request_focus();
            FocusManager::current().defer_keyboard_until_layout();
        }
    }
    let tab_row_scroll_recognizer =
        remember(|| ScrollRecognizer::new(ScrollSettings { consume: true }));
    let selected_index = selected.get();
    if controller.with(|controller| controller.active_tab()) != selected_index {
        controller.with_mut(|controller| controller.sync_selected(selected_index));
    }
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
    tabs_content()
        .args(config)
        .controller(controller)
        .content_shared(content)
        .tab_row_scroll_recognizer(tab_row_scroll_recognizer);
}

#[tessera]
fn tabs_content(
    args: Option<TabsConfig>,
    controller: Option<State<TabsController>>,
    content: Option<RenderSlot>,
    tab_row_scroll_recognizer: Option<State<ScrollRecognizer>>,
) {
    let args = args.expect("tabs_content requires args to be set");
    let live = remember(|| args.clone());
    if live.with(|value| value != &args) {
        live.set(args.clone());
    }
    let controller = controller.expect("tabs_content requires controller to be set");
    let content = content.expect("tabs_content requires content to be set");
    let tab_row_scroll_recognizer =
        tab_row_scroll_recognizer.expect("tabs_content requires scroll recognizer to be set");
    let tab_row_scroll_px = controller.with(|c| c.tab_row_scroll_px());
    let indicator_x_px = controller.with(|c| c.indicator_x_px());
    let indicator_width_px = controller.with(|c| c.indicator_width_px());

    let layout_args = args.clone();
    let modifier = with_pointer_input(
        args.modifier
            .clone()
            .push_semantics(TabListSemantics)
            .focus_group()
            .on_focus_changed(move |state: FocusState| {
                if !state.has_focus()
                    && controller.with(|controller| controller.focused_requester.is_some())
                {
                    controller.with_mut(|controller| controller.set_focused_requester(None));
                    FocusManager::current().defer_keyboard_until_layout();
                }
            }),
        move |input| {
            let is_scrollable = controller.with(|c| c.tab_row_scroll_max() > Px(0));
            if is_scrollable {
                let cursor_in_tab_bar = if let Some(pos) = input.cursor_position_rel {
                    let within_x = pos.x >= 0.0 && pos.x < input.computed_data.width.0 as f32;
                    let within_y =
                        pos.y >= 0.0 && pos.y < controller.with(|c| c.tab_bar_height()).0 as f32;
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
        focused_requester: controller.with(|controller| controller.focused_requester),
        active_tab: controller.with(|controller| controller.active_tab),
        scroll_user_overridden: controller
            .with(|controller| controller.tab_row_scroll_user_overridden),
    };
    layout()
        .modifier(modifier)
        .policy(policy.clone())
        .draw(policy)
        .child(move || {
            let args = live.get();
            let ripple_color = TabsDefaults::ripple_color(args.active_content_color);
            let indicator_shape = match args.variant {
                TabsVariant::Primary => Shape::RoundedRectangle {
                    top_left: RoundedCorner::manual(Dp(3.0), 3.0),
                    top_right: RoundedCorner::manual(Dp(3.0), 3.0),
                    bottom_right: RoundedCorner::manual(Dp(0.0), 3.0),
                    bottom_left: RoundedCorner::manual(Dp(0.0), 3.0),
                },
                TabsVariant::Secondary => Shape::RECTANGLE,
            };
            surface()
                .style(args.container_color.into())
                .modifier(Modifier::new().fill_max_size())
                .shape(Shape::RECTANGLE)
                .child(|| {});

            surface()
                .style(args.divider_color.into())
                .modifier(Modifier::new().fill_max_size())
                .shape(Shape::RECTANGLE)
                .child(|| {});

            surface()
                .style(args.indicator_color.into())
                .modifier(Modifier::new().fill_max_size())
                .shape(indicator_shape)
                .child(|| {});

            let composition = TabsCompositionContext {
                controller,
                selected: args.selection,
                enabled: args.enabled,
                active_content_color: args.active_content_color,
                inactive_content_color: args.inactive_content_color,
                inactive_interaction_color: args.inactive_interaction_color,
                disabled_content_color: args.disabled_content_color,
                ripple_color,
                min_tab_height: args.min_tab_height,
                tab_padding: args.tab_padding,
            };
            let children = RenderSlot::new(move || {
                let context = use_context::<TabsCompositionContext>()
                    .expect("Tabs composition context must be provided");
                if context.with(|current| current != &composition) {
                    context.set(composition);
                }
                content.render();
            });
            provide_context(|| composition, move || children.render());
        });
}

#[derive(Clone, PartialEq)]
struct TabTriggerArgs {
    controller: State<TabsController>,
    interaction: State<InteractionState>,
    title: TabTitle,
    enabled: bool,
    selected: bool,
    on_click: Option<Callback>,
    focus_requester: FocusRequester,
    label_color: Color,
    ripple_color: Color,
    tab_height: Dp,
    tab_padding: Dp,
    accessibility_label: Option<String>,
}

#[tessera]
fn tab_trigger(args: Option<TabTriggerArgs>) {
    let args = args.expect("tab_trigger requires args to be set");
    let live = remember(|| args.clone());
    if live.with(|value| value != &args) {
        live.set(args.clone());
    }
    let controller = args.controller;
    let requester = args.focus_requester;
    let target = TabFocusData {
        requester,
        selected: args.selected,
        enabled: args.enabled,
        activate: args.on_click.expect("tab activation"),
    };
    if controller.with(|controller| {
        controller
            .targets
            .iter()
            .any(|current| current.requester == requester && current != &target)
    }) {
        controller.with_mut(|controller| {
            let mut targets = controller.targets.clone();
            if let Some(current) = targets
                .iter_mut()
                .find(|current| current.requester == requester)
            {
                *current = target;
            }
            controller.sync_targets(targets);
        });
    }
    let tab_stop = controller.with(|controller| {
        if let Some(focused) = controller.focused_requester {
            focused == requester
        } else {
            args.selected
                || (!controller
                    .targets
                    .iter()
                    .any(|target| target.enabled && target.selected)
                    && controller
                        .targets
                        .iter()
                        .find(|target| target.enabled)
                        .is_some_and(|target| target.requester == requester))
        }
    });
    let mut tab_modifier = Modifier::new()
        .semantics(SemanticsArgs {
            selected: Some(args.selected),
            ..Default::default()
        })
        .constrain(None, Some(AxisConstraint::exact(args.tab_height.into())))
        .focus_group()
        .focus_properties(
            FocusProperties::new()
                .skip_traversal(!tab_stop)
                .can_focus(args.enabled),
        )
        .push_parent_data(target)
        .push_parent_data(TabIndicatorPadding(match &args.title {
            TabTitle::Custom(_) => Dp::ZERO,
            TabTitle::Label { .. } => args.tab_padding,
        }))
        .push_keyboard_input(TabNavigation {
            controller,
            requester,
        });

    if args.enabled {
        tab_modifier = tab_modifier.on_focus_event(move |focus_state: FocusState| {
            if focus_state.has_focus()
                && controller.with(|controller| controller.focused_requester != Some(requester))
            {
                controller.with_mut(|controller| controller.set_focused_requester(Some(requester)));
                FocusManager::current().defer_keyboard_until_layout();
            }
        });
    }

    let accessibility_label = args.accessibility_label.clone();
    let label_color = args.label_color;
    let ripple_color = args.ripple_color;
    let focus_requester = args.focus_requester;
    let on_click = args.on_click;

    surface()
        .style(Color::TRANSPARENT.into())
        .content_alignment(Alignment::Center)
        .content_color(label_color)
        .modifier(tab_modifier)
        .ripple_color(ripple_color)
        .shape(Shape::RECTANGLE)
        .enabled(args.enabled)
        .interaction_state(args.interaction)
        .focus_requester(focus_requester)
        .accessibility_role(Role::Tab)
        .accessibility_focusable(args.enabled)
        .accessibility_label_optional(accessibility_label)
        .on_click_optional(on_click)
        .child(move || {
            let args = live.get();
            tab_title_content()
                .title(args.title)
                .tab_padding(args.tab_padding);
        });
}

#[tessera]
fn tab_title_content(title: Option<TabTitle>, tab_padding: Option<Dp>) {
    let title = title.expect("tab_title_content requires title to be set");
    let tab_padding = tab_padding.unwrap_or(TabsDefaults::TAB_PADDING);
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

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use tessera_ui::{
        CursorEventContent, Dp, GestureState, Modifier, PointerChange, PressKeyEventType, Px,
        PxPosition, remember, tessera,
    };

    use crate::{
        column::column,
        lazy_list::lazy_column,
        modifier::{ModifierExt as _, SemanticsArgs},
        spacer::spacer,
        theme::{MaterialTheme, material_theme},
    };

    use super::{Spring1D, TabsController, tab, tabs};

    #[test]
    fn density_changes_preserve_in_flight_indicator_and_scroll_motion() {
        let mut original = TabsController::new(0);
        original.indicator_x = Spring1D {
            value: 10.0,
            velocity: 20.0,
            target: 100.0,
        };
        original.indicator_width = Spring1D {
            value: 24.0,
            velocity: 3.0,
            target: 60.0,
        };
        original.tab_row_scroll_offset = Spring1D {
            value: 40.0,
            velocity: 10.0,
            target: 200.0,
        };
        original.tab_row_scroll_max = Px(1000);
        original.tab_bar_height = Px(48);
        original.last_frame_nanos = Some(0);
        let mut scaled = original.clone();
        scaled.update_density(original.density * 1.5);
        original.advance_from_frame_nanos(16_666_667);
        scaled.advance_from_frame_nanos(16_666_667);
        for (before, after) in [
            (original.indicator_x, scaled.indicator_x),
            (original.indicator_width, scaled.indicator_width),
            (original.tab_row_scroll_offset, scaled.tab_row_scroll_offset),
        ] {
            assert!((after.value - before.value * 1.5).abs() < 0.0001);
            assert!((after.velocity - before.velocity * 1.5).abs() < 0.001);
            assert_eq!(after.target, before.target * 1.5);
        }
        assert_eq!(scaled.tab_row_scroll_max, Px(1500));
        assert_eq!(scaled.tab_bar_height, Px(72));
        scaled.update_density(original.density);
        assert!((scaled.indicator_x.value - original.indicator_x.value).abs() < 0.0001);
        assert!(scaled.has_pending_animation_frame());
    }

    thread_local! {
        static IDLE_BUILDS: Cell<usize> = const { Cell::new(0) };
    }

    #[tessera]
    fn idle_tabs_probe() {
        material_theme().theme(MaterialTheme::default).child(|| {
            IDLE_BUILDS.with(|builds| builds.set(builds.get() + 1));
            tabs_row();
        });
    }

    #[test]
    fn tabs_stop_rebuilding_after_the_indicator_settles() {
        IDLE_BUILDS.with(|builds| builds.set(0));
        let mut session = tessera_ui::testing::start_layout_test_session(
            tessera_ui::testing::layout_test(|| {
                idle_tabs_probe();
            })
            .viewport_px(1000, 900),
        );
        for frame in 0..90 {
            let _ = tessera_ui::testing::pump_layout_test_session(&mut session, frame * 16_666_667);
        }
        let settled_builds = IDLE_BUILDS.with(Cell::get);
        for frame in 90..98 {
            let _ = tessera_ui::testing::pump_layout_test_session(&mut session, frame * 16_666_667);
        }
        assert_eq!(IDLE_BUILDS.with(Cell::get), settled_builds);
    }

    #[tessera]
    fn tabs_row() {
        let selected = remember(|| 0usize);
        column().children(move || {
            tabs().selected(selected).content(move || {
                tab()
                    .label("Flights")
                    .selected(selected.get() == 0)
                    .on_click(move || selected.set(0));
                tab()
                    .label("Hotels")
                    .selected(selected.get() == 1)
                    .on_click(move || selected.set(1));
                tab()
                    .label("Trains")
                    .selected(selected.get() == 2)
                    .on_click(move || selected.set(2));
            });
            if selected.get() == 1 {
                spacer().modifier(Modifier::new().width(Dp(10.0)).height(Dp(10.0)).semantics(
                    SemanticsArgs {
                        test_tag: Some("hotels-active".to_string()),
                        ..Default::default()
                    },
                ));
            }
        });
    }

    #[tessera]
    fn tabs_click_probe() {
        material_theme()
            .theme(MaterialTheme::default)
            .child(move || {
                lazy_column().item(move || {
                    tabs_row();
                });
            });
    }

    #[test]
    fn clicking_middle_tab_selects_middle_tab() {
        let mut session = tessera_ui::testing::start_layout_test_session(
            tessera_ui::testing::layout_test(|| {
                tabs_click_probe();
            })
            .viewport_px(300, 200),
        );
        let _ = tessera_ui::testing::pump_layout_test_session(&mut session, 0);

        // Three equal tabs span the 300px viewport, so the middle tab covers
        // x in [100, 200] and its center is at x = 150.
        let now = tessera_ui::time::Instant::now();
        let cursor = Some(PxPosition::new(Px::new(150), Px::new(24)));

        // Move the pointer over the middle tab first. This rebuilds that tab
        // in isolation inside `lazy_column`; a per-instance self-assigned index
        // must still resolve to the middle tab after the click.
        let moved = PointerChange::new(
            now,
            1,
            CursorEventContent::Moved(PxPosition::new(Px::new(150), Px::new(24))),
            GestureState::TapCandidate,
        );
        let _ = tessera_ui::testing::pump_layout_test_session_with_input(
            &mut session,
            16_666_667,
            vec![moved],
            cursor,
        );

        let press = PointerChange::new(
            now,
            1,
            CursorEventContent::Pressed(PressKeyEventType::Left),
            GestureState::TapCandidate,
        );
        let _ = tessera_ui::testing::pump_layout_test_session_with_input(
            &mut session,
            33_333_334,
            vec![press],
            cursor,
        );
        let release = PointerChange::new(
            now,
            1,
            CursorEventContent::Released(PressKeyEventType::Left),
            GestureState::TapCandidate,
        );
        let mut snapshot = tessera_ui::testing::pump_layout_test_session_with_input(
            &mut session,
            50_000_001,
            vec![release],
            cursor,
        );
        for frame in 3..40u64 {
            snapshot = tessera_ui::testing::pump_layout_test_session(
                &mut session,
                50_000_001 + 16_666_667 * (frame - 2),
            );
        }

        // With the regression the shared counter keeps growing, so the click
        // selects the last tab and this probe never appears.
        snapshot.node("hotels-active").exists();
    }
}

#[cfg(test)]
mod custom_title_tests {
    use std::cell::Cell;

    use tessera_ui::{Dp, Modifier, Px, State, remember, tessera, use_context};

    use crate::{modifier::ModifierExt as _, spacer::spacer, theme::material_theme};

    use super::{TabsCompositionContext, TabsController, tab, tabs};

    thread_local! {
        static CONTROLLER: Cell<Option<State<TabsController>>> = const { Cell::new(None) };
        static SELECTED: Cell<Option<State<usize>>> = const { Cell::new(None) };
        static SCROLLABLE: Cell<Option<State<bool>>> = const { Cell::new(None) };
    }

    #[tessera]
    fn fixture() {
        let selected = remember(|| 0usize);
        let scrollable = remember(|| false);
        SELECTED.set(Some(selected));
        SCROLLABLE.set(Some(scrollable));
        material_theme().child(move || {
            tabs()
                .selected(selected)
                .scrollable(scrollable.get())
                .modifier(Modifier::new().width(Dp(300.0)))
                .content(|| {
                    CONTROLLER.set(Some(
                        use_context::<TabsCompositionContext>()
                            .unwrap()
                            .get()
                            .controller,
                    ));
                    tab().accessibility_label("Eighty").title(|| {
                        spacer().modifier(Modifier::new().width(Dp(80.0)).height(Dp(24.0)));
                    });
                    tab().accessibility_label("One twenty").title(|| {
                        spacer().modifier(Modifier::new().width(Dp(120.0)).height(Dp(24.0)));
                    });
                });
        });
    }

    #[test]
    fn primary_custom_title_indicator_uses_unpadded_slot_width_in_both_row_modes() {
        let mut session = tessera_ui::testing::start_layout_test_session(
            tessera_ui::testing::layout_test(fixture).viewport_px(300, 100),
        );
        let mut frame = 0_u64;
        tessera_ui::testing::pump_layout_test_session(&mut session, frame);
        let selected = SELECTED.get().unwrap();
        let scrollable = SCROLLABLE.get().unwrap();
        let controller = CONTROLLER.get().unwrap();
        for scrolling in [false, true] {
            scrollable.set(scrolling);
            for (index, width) in [(0, 80), (1, 120)] {
                selected.set(index);
                for _ in 0..90 {
                    frame += 16_666_667;
                    tessera_ui::testing::pump_layout_test_session(&mut session, frame);
                }
                assert_eq!(controller.with(|c| c.indicator_width.value_px()), Px(width));
            }
        }
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod small_custom_title_tests {
    use std::{cell::Cell, time::Duration};

    use tessera_ui::{
        Color, Dp, HeadlessConfig, HeadlessRenderer, Modifier, Px, State,
        accesskit::{Action, ActionRequest, Node, NodeId, Role, TreeId},
        remember, tessera, use_context,
    };

    use crate::{
        modifier::ModifierExt as _,
        spacer::spacer,
        theme::{MaterialTheme, material_theme},
    };

    use super::{TabsCompositionContext, TabsController, tab, tabs};

    thread_local! {
        static CONTROLLER: Cell<Option<State<TabsController>>> = const { Cell::new(None) };
        static SELECTED: Cell<Option<State<usize>>> = const { Cell::new(None) };
        static SCROLLABLE: Cell<Option<State<bool>>> = const { Cell::new(None) };
    }

    #[tessera]
    fn fixture() {
        let selected = remember(|| 0usize);
        let scrollable = remember(|| false);
        SELECTED.set(Some(selected));
        SCROLLABLE.set(Some(scrollable));
        material_theme()
            .theme(|| {
                let mut theme = MaterialTheme::default();
                theme.color_scheme.primary = Color::RED;
                theme.color_scheme.surface = Color::WHITE;
                theme
            })
            .child(move || {
                tabs()
                    .selected(selected)
                    .scrollable(scrollable.get())
                    .min_scrollable_tab_width(Dp::ZERO)
                    .edge_padding(Dp::ZERO)
                    .modifier(Modifier::new().width(Dp(300.0)))
                    .content(|| {
                        CONTROLLER.set(Some(
                            use_context::<TabsCompositionContext>()
                                .unwrap()
                                .get()
                                .controller,
                        ));
                        tab().accessibility_label("Small title").title(|| {
                            spacer().modifier(
                                Modifier::new()
                                    .width(Dp(32.0))
                                    .height(Dp(16.0))
                                    .background(Color::BLUE),
                            );
                        });
                        tab().accessibility_label("Minimum title").title(|| {
                            spacer().modifier(
                                Modifier::new()
                                    .width(Dp(16.0))
                                    .height(Dp(16.0))
                                    .background(Color::BLUE),
                            );
                        });
                    });
            });
    }

    fn node(renderer: &HeadlessRenderer, label: &str, role: Role) -> (NodeId, Node) {
        renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, node)| node.role() == role && node.label() == Some(label))
            .unwrap_or_else(|| panic!("missing {role:?} {label}"))
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn gpu_small_custom_title_indicators_exclude_minimum_interactive_padding() {
        crate::test_utils::run_isolated_test(
            "tabs::small_custom_title_tests::gpu_small_custom_title_indicators_exclude_minimum_interactive_padding",
            || {
                let mut renderer = HeadlessRenderer::new(
                    Box::new(fixture),
                    vec![Box::new(crate::TesseraComponents)],
                    HeadlessConfig {
                        width: 300,
                        height: 100,
                        sample_count: 1,
                        frame_time: Duration::from_millis(20),
                        ..Default::default()
                    },
                )
                .unwrap();
                renderer.render_frames(12);
                let selected = SELECTED.get().unwrap();
                let scrollable = SCROLLABLE.get().unwrap();
                let controller = CONTROLLER.get().unwrap();
                for scrolling in [false, true] {
                    scrollable.set(scrolling);
                    renderer.render_frames(90);
                    for (index, label, visual_width, indicator_width) in
                        [(0, "Small title", 32, 32), (1, "Minimum title", 16, 24)]
                    {
                        let target_node = node(&renderer, label, Role::Tab).0;
                        assert!(renderer.dispatch_accessibility_action(ActionRequest {
                            action: Action::Click,
                            target_tree: TreeId::ROOT,
                            target_node,
                            data: None,
                        }));
                        renderer.render_frames(90);
                        assert_eq!(selected.get(), index);
                        assert_eq!(
                            renderer.accessibility_snapshot().unwrap().focus,
                            target_node
                        );
                        tessera_ui::FocusManager::current().clear_focus(true);
                        renderer.render_frames(12);
                        assert_ne!(
                            renderer.accessibility_snapshot().unwrap().focus,
                            target_node
                        );
                        assert_eq!(selected.get(), index);
                        let target = node(&renderer, label, Role::Tab).1.bounds().unwrap();
                        assert_eq!(target.x1 - target.x0, if scrolling { 48.0 } else { 150.0 });
                        assert_eq!(target.y1 - target.y0, 48.0);
                        let center = (target.x0 + target.x1) * 0.5;
                        assert_eq!(
                            controller.with(|c| c.indicator_width_px()),
                            Px(indicator_width)
                        );
                        let pixels = renderer.read_pixels().unwrap();
                        let width = renderer.physical_size().width;
                        let mut visual = Vec::new();
                        for y in target.y0 as u32..target.y1 as u32 {
                            for x in target.x0 as u32..target.x1 as u32 {
                                let offset = ((y * width + x) * 4) as usize;
                                if pixels[offset] < 30
                                    && pixels[offset + 1] < 30
                                    && pixels[offset + 2] > 240
                                {
                                    visual.push((x, y));
                                }
                            }
                        }
                        assert!(
                            !visual.is_empty(),
                            "no blue title paint within {target:?} after clearing focus"
                        );
                        let left = visual.iter().map(|(x, _)| *x).min().unwrap();
                        let right = visual.iter().map(|(x, _)| *x).max().unwrap() + 1;
                        let top = visual.iter().map(|(_, y)| *y).min().unwrap();
                        let bottom = visual.iter().map(|(_, y)| *y).max().unwrap() + 1;
                        assert_eq!(
                            (right - left, bottom - top),
                            (visual_width as u32, 16),
                            "title paint bbox={:?}",
                            (left, top, right, bottom)
                        );
                        assert_eq!(visual.len(), visual_width * 16);
                        assert_eq!((f64::from(left) + f64::from(right)) * 0.5, center);
                        assert_eq!(
                            (f64::from(top) + f64::from(bottom)) * 0.5,
                            (target.y0 + target.y1) * 0.5,
                            "the full natural title paint must remain centered within the interaction target",
                        );
                        let y = target.y1 as u32 - 1;
                        let painted = (0..width)
                            .filter(|x| {
                                let offset = ((y * width + x) * 4) as usize;
                                pixels[offset] > 240
                                    && pixels[offset + 1] < 30
                                    && pixels[offset + 2] < 30
                            })
                            .collect::<Vec<_>>();
                        assert_eq!(painted.len(), indicator_width as usize);
                        assert_eq!(
                            (f64::from(*painted.first().unwrap())
                                + f64::from(*painted.last().unwrap())
                                + 1.0)
                                * 0.5,
                            center,
                            "the visual indicator must remain centered within the full interaction target",
                        );
                    }
                }
            },
        );
    }
}
