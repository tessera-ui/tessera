//! Material 3-style segmented buttons with single or multiple selection.
//! ## Usage: Group related actions into adjacent buttons that share MD3 styling and state feedback.
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use derive_builder::Builder;
use parking_lot::RwLock;
use tessera_ui::{Color, DimensionValue, Dp, accesskit::Role, tessera};

use crate::{
    RippleState,
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    button::{ButtonArgsBuilder, button},
    icon::{IconArgs, icon},
    row::{RowArgsBuilder, RowScope, row},
    shape_def::{RoundedCorner, Shape},
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgsBuilder, SurfaceStyle, surface},
    text::{TextArgsBuilder, text},
};

/// Selection behavior for the button group.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonGroupSelection {
    /// Only one segment can be selected at a time.
    Single,
    /// Multiple segments can be selected independently.
    Multiple,
}

/// Change notification emitted after a segment toggles.
#[derive(Clone, Debug)]
pub struct ButtonGroupChange {
    /// Index of the segment that changed.
    pub index: usize,
    /// Whether that segment is now selected.
    pub selected: bool,
    /// Snapshot of all selected indices (sorted).
    pub selection: Vec<usize>,
}

/// Visual and behavioral configuration for the `button_group` component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct ButtonGroupArgs {
    /// Width of the group container.
    #[builder(default = "DimensionValue::WRAP", setter(into))]
    pub width: DimensionValue,
    /// Height of the group container.
    #[builder(default = "DimensionValue::WRAP", setter(into))]
    pub height: DimensionValue,
    /// Fill color for the container behind the segments (MD3 uses surface-variant).
    #[builder(default = "Color::new(0.97, 0.97, 0.98, 1.0)")]
    pub container_color: Color,
    /// Outline color for the group perimeter.
    #[builder(default = "Color::new(0.39, 0.41, 0.44, 1.0)")]
    pub outline_color: Color,
    /// Outline width for the group perimeter.
    #[builder(default = "Dp(1.0)")]
    pub outline_width: Dp,
    /// Padding between the group outline and the first/last segments.
    #[builder(default = "Dp(2.0)")]
    pub container_padding: Dp,
    /// Background color for an unselected segment (usually matches `container_color`).
    #[builder(default = "Color::new(0.97, 0.97, 0.98, 1.0)")]
    pub segment_background: Color,
    /// Background color for a selected segment (MD3 primary-container default).
    #[builder(default = "Color::new(0.87, 0.91, 1.0, 1.0)")]
    pub selected_background: Color,
    /// Foreground color for unselected segment content.
    #[builder(default = "Color::new(0.14, 0.16, 0.20, 1.0)")]
    pub content_color: Color,
    /// Foreground color for selected segment content.
    #[builder(default = "Color::new(0.08, 0.22, 0.43, 1.0)")]
    pub selected_content_color: Color,
    /// Foreground color for disabled segment content.
    #[builder(default = "Color::new(0.14, 0.16, 0.20, 0.38)")]
    pub disabled_content_color: Color,
    /// Ripple tint for unselected segments.
    #[builder(default = "Color::new(0.08, 0.22, 0.43, 1.0)")]
    pub ripple_color: Color,
    /// Ripple tint for selected segments.
    #[builder(default = "Color::new(0.08, 0.22, 0.43, 1.0)")]
    pub selected_ripple_color: Color,
    /// Color used for hover/pressed state layers (applied as an overlay).
    #[builder(default = "Color::new(0.08, 0.22, 0.43, 1.0)")]
    pub state_layer_color: Color,
    /// Opacity applied to `state_layer_color` on hover.
    #[builder(default = "0.12")]
    pub hover_state_layer_opacity: f32,
    /// Whether clicking the active segment in single-selection mode will clear the selection.
    #[builder(default = "false")]
    pub allow_deselect_single: bool,
    /// Internal padding for each segment's content.
    #[builder(default = "Dp(12.0)")]
    pub segment_padding: Dp,
    /// Width of vertical dividers between segments.
    #[builder(default = "Dp(1.0)")]
    pub divider_width: Dp,
    /// Color of vertical dividers between segments.
    #[builder(default = "Color::new(0.82, 0.84, 0.86, 1.0)")]
    pub divider_color: Color,
    /// Whether to draw dividers between segments.
    #[builder(default = "true")]
    pub show_dividers: bool,
    /// Optional callback triggered after a selection change.
    #[builder(default, setter(strip_option))]
    pub on_change: Option<Arc<dyn Fn(ButtonGroupChange) + Send + Sync>>,
    /// Optional accessibility label for the group container.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_label: Option<String>,
}

/// Content options for each segment.
pub enum ButtonGroupItemContent {
    /// Text-only segment.
    Text(String),
    /// Text with a leading icon, spaced by 8dp.
    TextWithIcon {
        /// The segment label.
        label: String,
        /// The icon to render before the label.
        icon: IconArgs,
        /// Gap between icon and text.
        spacing: Dp,
    },
    /// Fully custom content; the provided color should be applied to match the group's state.
    Custom(Box<dyn FnOnce(Color) + Send + Sync>),
}

/// Definition of a single segment inside the group.
pub struct ButtonGroupItem {
    content: ButtonGroupItemContent,
    on_select: Option<Arc<dyn Fn(bool) + Send + Sync>>,
    accessibility_label: Option<String>,
    enabled: bool,
}

impl ButtonGroupItem {
    /// Create a text-only segment.
    pub fn text(label: impl Into<String>) -> Self {
        Self {
            content: ButtonGroupItemContent::Text(label.into()),
            on_select: None,
            accessibility_label: None,
            enabled: true,
        }
    }

    /// Create a segment with a leading icon.
    pub fn text_with_icon(label: impl Into<String>, icon: impl Into<IconArgs>) -> Self {
        Self {
            content: ButtonGroupItemContent::TextWithIcon {
                label: label.into(),
                icon: icon.into(),
                spacing: Dp(8.0),
            },
            on_select: None,
            accessibility_label: None,
            enabled: true,
        }
    }

    /// Create a segment with fully custom content that receives the resolved content color.
    pub fn custom(content: impl FnOnce(Color) + Send + Sync + 'static) -> Self {
        Self {
            content: ButtonGroupItemContent::Custom(Box::new(content)),
            on_select: None,
            accessibility_label: None,
            enabled: true,
        }
    }

    /// Attach a callback that receives the new selected state after the segment toggles.
    pub fn with_on_select(mut self, callback: Arc<dyn Fn(bool) + Send + Sync>) -> Self {
        self.on_select = Some(callback);
        self
    }

    /// Override the accessibility label for this segment.
    pub fn with_accessibility_label(mut self, label: impl Into<String>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    /// Disable this segment (no interaction or ripple).
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Scope used to define the segments in a group.
pub struct ButtonGroupScope<'a> {
    items: &'a mut Vec<ButtonGroupItem>,
}

impl<'a> ButtonGroupScope<'a> {
    /// Register a segment item.
    pub fn item(&mut self, item: ButtonGroupItem) {
        self.items.push(item);
    }
}

struct ButtonGroupStateInner {
    selection_mode: ButtonGroupSelection,
    selected: HashSet<usize>,
    ripple_states: HashMap<usize, RippleState>,
    allow_deselect_single: bool,
}

impl ButtonGroupStateInner {
    fn new(
        selection_mode: ButtonGroupSelection,
        initial_selection: impl IntoIterator<Item = usize>,
        allow_deselect_single: bool,
    ) -> Self {
        let mut selected = HashSet::new();
        match selection_mode {
            ButtonGroupSelection::Single => {
                if let Some(first) = initial_selection.into_iter().next() {
                    selected.insert(first);
                }
            }
            ButtonGroupSelection::Multiple => {
                for idx in initial_selection {
                    selected.insert(idx);
                }
            }
        }
        Self {
            selection_mode,
            selected,
            ripple_states: HashMap::new(),
            allow_deselect_single,
        }
    }

    fn toggle(&mut self, index: usize) -> bool {
        match self.selection_mode {
            ButtonGroupSelection::Single => {
                if self.selected.contains(&index) {
                    if self.allow_deselect_single {
                        self.selected.clear();
                        return false;
                    }
                    true
                } else {
                    self.selected.clear();
                    self.selected.insert(index);
                    true
                }
            }
            ButtonGroupSelection::Multiple => {
                if self.selected.remove(&index) {
                    false
                } else {
                    self.selected.insert(index);
                    true
                }
            }
        }
    }

    fn select(&mut self, index: usize) -> bool {
        match self.selection_mode {
            ButtonGroupSelection::Single => {
                let changed = !self.selected.contains(&index);
                self.selected.clear();
                self.selected.insert(index);
                changed
            }
            ButtonGroupSelection::Multiple => self.selected.insert(index),
        }
    }

    fn deselect(&mut self, index: usize) -> bool {
        self.selected.remove(&index)
    }

    fn selected_sorted(&self) -> Vec<usize> {
        let mut values: Vec<_> = self.selected.iter().copied().collect();
        values.sort_unstable();
        values
    }

    fn ripple_state(&mut self, index: usize) -> RippleState {
        self.ripple_states.entry(index).or_default().clone()
    }
}

/// Shared state for a button group, tracking selection and ripple per segment.
///
/// Clone this handle to share selection across multiple `button_group` renders.
#[derive(Clone)]
pub struct ButtonGroupState {
    inner: Arc<RwLock<ButtonGroupStateInner>>,
}

impl ButtonGroupState {
    /// Create a single-select group with no initial selection.
    pub fn single() -> Self {
        Self::new(ButtonGroupSelection::Single, [], false)
    }

    /// Create a single-select group with an optional initial selection and deselect behavior.
    pub fn single_with_initial(initial: Option<usize>, allow_deselect: bool) -> Self {
        let iter = initial.into_iter();
        Self::new(ButtonGroupSelection::Single, iter, allow_deselect)
    }

    /// Create a multi-select group with optional initial selections.
    pub fn multiple(initial: impl IntoIterator<Item = usize>) -> Self {
        Self::new(ButtonGroupSelection::Multiple, initial, false)
    }

    /// Returns the current selection mode.
    pub fn selection_mode(&self) -> ButtonGroupSelection {
        self.inner.read().selection_mode
    }

    /// Returns whether the given index is currently selected.
    pub fn is_selected(&self, index: usize) -> bool {
        self.inner.read().selected.contains(&index)
    }

    /// Returns all selected indices (sorted).
    pub fn selected_indices(&self) -> Vec<usize> {
        self.inner.read().selected_sorted()
    }

    /// Select a segment explicitly.
    pub fn select(&self, index: usize) -> bool {
        self.inner.write().select(index)
    }

    /// Deselect a specific segment (multi-select only).
    pub fn deselect(&self, index: usize) -> bool {
        self.inner.write().deselect(index)
    }

    /// Toggle a segment and return whether it is selected after the change.
    pub fn toggle(&self, index: usize) -> bool {
        self.inner.write().toggle(index)
    }

    /// Returns whether single-select groups are allowed to clear selection when the active chip is pressed again.
    pub fn allows_single_deselect(&self) -> bool {
        self.inner.read().allow_deselect_single
    }

    fn new(
        selection_mode: ButtonGroupSelection,
        initial_selection: impl IntoIterator<Item = usize>,
        allow_deselect_single: bool,
    ) -> Self {
        Self {
            inner: Arc::new(RwLock::new(ButtonGroupStateInner::new(
                selection_mode,
                initial_selection,
                allow_deselect_single,
            ))),
        }
    }
}

fn overlay(base: Color, layer: Color, opacity: f32) -> Color {
    let clamped = opacity.clamp(0.0, 1.0);
    let inv = 1.0 - clamped;
    Color::new(
        base.r * inv + layer.r * clamped,
        base.g * inv + layer.g * clamped,
        base.b * inv + layer.b * clamped,
        base.a,
    )
}

fn segment_shape(index: usize, count: usize) -> Shape {
    if count <= 1 {
        return Shape::capsule();
    }

    match index {
        0 => Shape::RoundedRectangle {
            top_left: RoundedCorner::Capsule,
            top_right: RoundedCorner::manual(Dp(0.0), 3.0),
            bottom_right: RoundedCorner::manual(Dp(0.0), 3.0),
            bottom_left: RoundedCorner::Capsule,
        },
        i if i + 1 == count => Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(Dp(0.0), 3.0),
            top_right: RoundedCorner::Capsule,
            bottom_right: RoundedCorner::Capsule,
            bottom_left: RoundedCorner::manual(Dp(0.0), 3.0),
        },
        _ => Shape::RECTANGLE,
    }
}

fn render_item_content(content: ButtonGroupItemContent, color: Color) {
    match content {
        ButtonGroupItemContent::Text(label) => {
            text(
                TextArgsBuilder::default()
                    .text(label)
                    .color(color)
                    .size(Dp(14.0))
                    .build()
                    .expect("builder construction failed"),
            );
        }
        ButtonGroupItemContent::TextWithIcon {
            label,
            icon: icon_args,
            spacing,
        } => {
            let label_for_text = label.clone();
            let icon_args_for_icon = icon_args.clone();
            let spacing_value = spacing;
            row(
                RowArgsBuilder::default()
                    .height(DimensionValue::Wrap {
                        min: None,
                        max: None,
                    })
                    .build()
                    .expect("builder construction failed"),
                move |scope: &mut RowScope| {
                    let icon_args_captured = icon_args_for_icon.clone();
                    scope.child(move || {
                        icon(IconArgs {
                            tint: color,
                            ..icon_args_captured
                        });
                    });
                    scope.child(move || spacer(SpacerArgs::from(spacing_value)));
                    scope.child(move || {
                        text(
                            TextArgsBuilder::default()
                                .text(label_for_text.clone())
                                .color(color)
                                .size(Dp(14.0))
                                .build()
                                .expect("builder construction failed"),
                        );
                    });
                },
            );
        }
        ButtonGroupItemContent::Custom(content) => content(color),
    }
}

fn accessibility_label_for_item(
    item: &ButtonGroupItem,
    fallback_content_label: Option<String>,
) -> Option<String> {
    if let Some(label) = item.accessibility_label.clone() {
        return Some(label);
    }
    fallback_content_label
}

/// # button_group
///
/// Render a Material 3 segmented button group with single or multiple selection.
///
/// ## Usage
///
/// Present adjacent actions that share equal emphasis while indicating selection state.
///
/// ## Parameters
///
/// - `args` — controls MD3 styling, spacing, and callbacks; see [`ButtonGroupArgs`].
/// - `state` — a clonable [`ButtonGroupState`] to drive single/multi selection.
/// - `scope_config` — closure used to register each segment via [`ButtonGroupScope`].
///
/// ## Examples
///
/// ```
/// use std::sync::Arc;
/// use tessera_ui_basic_components::{
///     button_group::{
///         button_group, ButtonGroupArgsBuilder, ButtonGroupItem, ButtonGroupSelection,
///         ButtonGroupState,
///     },
/// };
///
/// let state = ButtonGroupState::single_with_initial(Some(0), false);
/// let args = ButtonGroupArgsBuilder::default()
///     .allow_deselect_single(false)
///     .build()
///     .expect("builder construction failed");
///
/// button_group(args, state.clone(), |scope| {
///     scope.item(ButtonGroupItem::text("Day"));
///     scope.item(ButtonGroupItem::text("Week"));
///     scope.item(ButtonGroupItem::text("Month"));
/// });
///
/// // State utility can be tested directly.
/// assert_eq!(state.selection_mode(), ButtonGroupSelection::Single);
/// assert!(state.is_selected(0));
/// state.toggle(1);
/// assert!(state.is_selected(1));
/// assert!(!state.is_selected(0));
/// ```
#[tessera]
pub fn button_group<F>(args: ButtonGroupArgs, state: ButtonGroupState, scope_config: F)
where
    F: FnOnce(&mut ButtonGroupScope),
{
    let mut items = Vec::new();
    {
        let mut scope = ButtonGroupScope { items: &mut items };
        scope_config(&mut scope);
    }

    let item_count = items.len();
    let selection_mode = state.selection_mode();

    let mut container_builder = SurfaceArgsBuilder::default()
        .style(SurfaceStyle::FilledOutlined {
            fill_color: args.container_color,
            border_color: args.outline_color,
            border_width: args.outline_width,
        })
        .shape(Shape::capsule())
        .padding(args.container_padding)
        .width(args.width)
        .height(args.height)
        .accessibility_role(Role::Group);

    if let Some(label) = args.accessibility_label.clone() {
        container_builder = container_builder.accessibility_label(label);
    }

    surface(
        container_builder
            .build()
            .expect("builder construction failed"),
        None,
        || {
            if item_count == 0 {
                return;
            }

            let allow_deselect = args.allow_deselect_single || state.allows_single_deselect();

            row(
                RowArgsBuilder::default()
                    .width(DimensionValue::FILLED)
                    .height(DimensionValue::Wrap {
                        min: None,
                        max: None,
                    })
                    .main_axis_alignment(MainAxisAlignment::Start)
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .build()
                    .expect("builder construction failed"),
                |row_scope| {
                    for (idx, item) in items.into_iter().enumerate() {
                        let state_for_item = state.clone();
                        let is_selected = state_for_item.is_selected(idx);
                        let content_color = if item.enabled {
                            if is_selected {
                                args.selected_content_color
                            } else {
                                args.content_color
                            }
                        } else {
                            args.disabled_content_color
                        };
                        let background = if is_selected {
                            args.selected_background
                        } else {
                            args.segment_background
                        };
                        let hover_color = if item.enabled {
                            Some(overlay(
                                background,
                                args.state_layer_color,
                                args.hover_state_layer_opacity,
                            ))
                        } else {
                            None
                        };

                        let item_content_label = match &item.content {
                            ButtonGroupItemContent::Text(label) => Some(label.clone()),
                            ButtonGroupItemContent::TextWithIcon { label, .. } => {
                                Some(label.clone())
                            }
                            ButtonGroupItemContent::Custom(_) => None,
                        };
                        let accessibility_label =
                            accessibility_label_for_item(&item, item_content_label);

                        let ripple_state = state_for_item.inner.write().ripple_state(idx);
                        let on_change = args.on_change.clone();

                        row_scope.child_weighted(
                            move || {
                                let state_clone = state_for_item.clone();
                                let on_select = item.on_select.clone();
                                let button_shape = segment_shape(idx, item_count);
                                let ripple = if is_selected {
                                    args.selected_ripple_color
                                } else {
                                    args.ripple_color
                                };

                                let click_handler = if item.enabled {
                                    Some(Arc::new(move || {
                                        let will_deselect = selection_mode
                                            == ButtonGroupSelection::Single
                                            && allow_deselect
                                            && state_clone.is_selected(idx);
                                        let selected_now = if will_deselect {
                                            state_clone.deselect(idx);
                                            false
                                        } else {
                                            state_clone.toggle(idx)
                                        };

                                        if let Some(cb) = on_select.as_ref() {
                                            cb(selected_now);
                                        }
                                        if let Some(change) = on_change.as_ref() {
                                            change(ButtonGroupChange {
                                                index: idx,
                                                selected: selected_now,
                                                selection: state_clone.selected_indices(),
                                            });
                                        }
                                    })
                                        as Arc<dyn Fn() + Send + Sync>)
                                } else {
                                    None
                                };

                                let mut builder = ButtonArgsBuilder::default()
                                    .color(background)
                                    .hover_color(hover_color)
                                    .padding(args.segment_padding)
                                    .shape(button_shape)
                                    .width(DimensionValue::FILLED)
                                    .ripple_color(ripple);

                                if let Some(handler) = click_handler {
                                    builder = builder.on_click(handler);
                                }
                                if let Some(label) = accessibility_label {
                                    builder = builder.accessibility_label(label);
                                }

                                button(
                                    builder.build().expect("builder construction failed"),
                                    ripple_state,
                                    move || render_item_content(item.content, content_color),
                                );
                            },
                            1.0,
                        );

                        if args.show_dividers && idx + 1 < item_count {
                            let divider_color = args.divider_color;
                            let divider_width = args.divider_width;
                            row_scope.child(move || {
                                surface(
                                    SurfaceArgsBuilder::default()
                                        .style(SurfaceStyle::Filled {
                                            color: divider_color,
                                        })
                                        .width(divider_width)
                                        .height(DimensionValue::WRAP)
                                        .build()
                                        .expect("builder construction failed"),
                                    None,
                                    || {},
                                );
                            });
                        }
                    }
                },
            );
        },
    );
}
