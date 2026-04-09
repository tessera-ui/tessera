//! Material Design segmented buttons for compact selections.
//!
//! ## Usage
//!
//! Switch between views or filters with a connected control.

use tessera_ui::{
    AxisConstraint, Callback, Color, ComputedData, Constraint, Dp, FocusState,
    FocusTraversalPolicy, LayoutPolicy, LayoutResult, MeasurementError, Modifier, Px, PxPosition,
    RenderSlot,
    accesskit::Role,
    layout::{MeasureScope, layout},
    modifier::FocusModifierExt as _,
    provide_context, tessera, use_context,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment},
    icon::{IconContent, icon as icon_component},
    modifier::{ModifierExt as _, Padding},
    row::row,
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    surface::{SurfaceStyle, surface},
    text::text,
    theme::{MaterialAlpha, MaterialTheme, provide_text_style},
};

const SEGMENTED_ICON_SPACING: Dp = Dp(8.0);

#[derive(Clone, Copy, Debug)]
struct SegmentedButtonRowContext {
    select_on_focus: bool,
}

/// Color values for segmented buttons in different states.
#[derive(Clone, PartialEq, Copy, Debug)]
pub struct SegmentedButtonColors {
    /// Container color when enabled and active.
    pub active_container_color: Color,
    /// Content color when enabled and active.
    pub active_content_color: Color,
    /// Border color when enabled and active.
    pub active_border_color: Color,
    /// Container color when enabled and inactive.
    pub inactive_container_color: Color,
    /// Content color when enabled and inactive.
    pub inactive_content_color: Color,
    /// Border color when enabled and inactive.
    pub inactive_border_color: Color,
    /// Container color when disabled and active.
    pub disabled_active_container_color: Color,
    /// Content color when disabled and active.
    pub disabled_active_content_color: Color,
    /// Border color when disabled and active.
    pub disabled_active_border_color: Color,
    /// Container color when disabled and inactive.
    pub disabled_inactive_container_color: Color,
    /// Content color when disabled and inactive.
    pub disabled_inactive_content_color: Color,
    /// Border color when disabled and inactive.
    pub disabled_inactive_border_color: Color,
}

impl SegmentedButtonColors {
    fn container_color(self, enabled: bool, active: bool) -> Color {
        match (enabled, active) {
            (true, true) => self.active_container_color,
            (true, false) => self.inactive_container_color,
            (false, true) => self.disabled_active_container_color,
            (false, false) => self.disabled_inactive_container_color,
        }
    }

    fn content_color(self, enabled: bool, active: bool) -> Color {
        match (enabled, active) {
            (true, true) => self.active_content_color,
            (true, false) => self.inactive_content_color,
            (false, true) => self.disabled_active_content_color,
            (false, false) => self.disabled_inactive_content_color,
        }
    }

    fn border_color(self, enabled: bool, active: bool) -> Color {
        match (enabled, active) {
            (true, true) => self.active_border_color,
            (true, false) => self.inactive_border_color,
            (false, true) => self.disabled_active_border_color,
            (false, false) => self.disabled_inactive_border_color,
        }
    }
}

/// Defaults for segmented buttons.
pub struct SegmentedButtonDefaults;

impl SegmentedButtonDefaults {
    /// Minimum height for segmented buttons.
    pub const HEIGHT: Dp = Dp(40.0);
    /// Default border width.
    pub const BORDER_WIDTH: Dp = Dp(1.0);
    /// Default icon size.
    pub const ICON_SIZE: Dp = Dp(18.0);
    /// Default content padding.
    pub const CONTENT_PADDING: Padding = Padding::symmetric(Dp(12.0), Dp(8.0));

    /// Default shape for segmented buttons.
    pub fn shape() -> Shape {
        Shape::CAPSULE
    }

    /// Build the item shape for the button at `index` with `count` items.
    pub fn item_shape(index: usize, count: usize, base_shape: Shape) -> Shape {
        if count <= 1 {
            return base_shape;
        }

        let Shape::RoundedRectangle {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        } = base_shape
        else {
            return base_shape;
        };

        let is_first = index == 0;
        let is_last = index + 1 == count;

        Shape::RoundedRectangle {
            top_left: if is_first {
                top_left
            } else {
                RoundedCorner::ZERO
            },
            top_right: if is_last {
                top_right
            } else {
                RoundedCorner::ZERO
            },
            bottom_right: if is_last {
                bottom_right
            } else {
                RoundedCorner::ZERO
            },
            bottom_left: if is_first {
                bottom_left
            } else {
                RoundedCorner::ZERO
            },
        }
    }

    /// Default colors derived from the current theme.
    pub fn colors() -> SegmentedButtonColors {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        let disabled_content = scheme
            .on_surface
            .with_alpha(MaterialAlpha::DISABLED_CONTENT);
        let disabled_border = scheme.outline.with_alpha(MaterialAlpha::DISABLED_CONTAINER);

        SegmentedButtonColors {
            active_container_color: scheme.secondary_container,
            active_content_color: scheme.on_secondary_container,
            active_border_color: scheme.outline,
            inactive_container_color: Color::TRANSPARENT,
            inactive_content_color: scheme.on_surface,
            inactive_border_color: scheme.outline,
            disabled_active_container_color: scheme.secondary_container,
            disabled_active_content_color: disabled_content,
            disabled_active_border_color: disabled_border,
            disabled_inactive_container_color: Color::TRANSPARENT,
            disabled_inactive_content_color: disabled_content,
            disabled_inactive_border_color: disabled_border,
        }
    }
}

/// # segmented_button
///
/// Render a segmented button item with optional icon and label.
///
/// ## Usage
///
/// Use inside segmented button rows to represent selectable options.
///
/// ## Parameters
///
/// - `selected` — whether the segment is selected.
/// - `enabled` — whether the segment is enabled.
/// - `label` — label shown inside the segment.
/// - `icon` — optional icon displayed before the label.
/// - `modifier` — layout and behavior modifiers applied to the segment.
/// - `shape` — shape of the segment.
/// - `colors` — optional color overrides for the segment.
/// - `border_width` — border width around the segment.
/// - `content_padding` — internal padding for the segment content.
/// - `on_click` — optional click callback.
/// - `accessibility_label` — optional accessibility label.
/// - `accessibility_description` — optional accessibility description.
///
/// ## Examples
///
/// ```
/// use tessera_components::segmented_buttons::{
///     SegmentedButtonDefaults, segmented_button, single_choice_segmented_button_row,
/// };
/// use tessera_components::theme::{MaterialTheme, material_theme};
/// use tessera_ui::{LayoutResult, remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     material_theme()
///         .theme(|| MaterialTheme::default())
///         .child(|| {
///             let selected = remember(|| 0usize);
///             single_choice_segmented_button_row().content(move || {
///                 segmented_button()
///                     .label("List")
///                     .selected(selected.get() == 0)
///                     .shape(SegmentedButtonDefaults::item_shape(
///                         0,
///                         2,
///                         SegmentedButtonDefaults::shape(),
///                     ))
///                     .on_click(|| {});
///                 segmented_button()
///                     .label("Grid")
///                     .selected(selected.get() == 1)
///                     .shape(SegmentedButtonDefaults::item_shape(
///                         1,
///                         2,
///                         SegmentedButtonDefaults::shape(),
///                     ))
///                     .on_click(|| {});
///             });
///             selected.with_mut(|value| *value = 1);
///             assert_eq!(selected.get(), 1);
///         });
/// }
///
/// demo();
/// ```
#[tessera]
pub fn segmented_button(
    selected: bool,
    enabled: Option<bool>,
    #[prop(into)] label: String,
    #[prop(into)] icon: Option<IconContent>,
    modifier: Option<Modifier>,
    shape: Option<Shape>,
    colors: Option<SegmentedButtonColors>,
    border_width: Option<Dp>,
    content_padding: Option<Padding>,
    on_click: Option<Callback>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let enabled = enabled.unwrap_or(true);
    let modifier = modifier.unwrap_or_default();
    let shape = shape.unwrap_or_else(SegmentedButtonDefaults::shape);
    let row_context = use_context::<SegmentedButtonRowContext>().map(|context| context.get());
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let typography = theme.typography;
    let colors = colors.unwrap_or_else(SegmentedButtonDefaults::colors);
    let container_color = colors.container_color(enabled, selected);
    let content_color = colors.content_color(enabled, selected);
    let border_color = colors.border_color(enabled, selected);
    let border_width = border_width.unwrap_or(SegmentedButtonDefaults::BORDER_WIDTH);
    let content_padding = content_padding.unwrap_or(SegmentedButtonDefaults::CONTENT_PADDING);

    let surface_style = if border_width.0 > 0.0 {
        SurfaceStyle::FilledOutlined {
            fill_color: container_color,
            border_color,
            border_width,
        }
    } else {
        SurfaceStyle::Filled {
            color: container_color,
        }
    };

    let mut modifier = modifier.size_in(None, None, Some(SegmentedButtonDefaults::HEIGHT), None);
    if enabled
        && row_context.is_some_and(|context| context.select_on_focus)
        && let Some(on_click) = on_click
    {
        let is_selected = selected;
        modifier = modifier.on_focus_changed(move |focus_state: FocusState| {
            if focus_state.has_focus() && !is_selected {
                on_click.call();
            }
        });
    }

    let mut button = surface()
        .modifier(modifier)
        .style(surface_style)
        .shape(shape)
        .content_alignment(Alignment::Center)
        .content_color(content_color)
        .enabled(enabled)
        .ripple_color(content_color);

    if let Some(on_click) = on_click {
        button = button
            .on_click_shared(on_click)
            .accessibility_role(Role::Button)
            .accessibility_focusable(true);
    }

    let accessibility_label =
        accessibility_label.or_else(|| (!label.is_empty()).then(|| label.clone()));
    if let Some(label) = accessibility_label {
        button = button.accessibility_label(label);
    }
    if let Some(description) = accessibility_description {
        button = button.accessibility_description(description);
    }

    button.with_child(move || {
        let leading_icon = icon.clone();
        let padding = content_padding;
        let label_outer = label.clone();
        provide_text_style(typography.label_large, move || {
            let leading_icon = leading_icon.clone();
            let label_outer = label_outer.clone();
            layout()
                .modifier(Modifier::new().padding(padding))
                .child(move || {
                    let leading_icon = leading_icon.clone();
                    let label = label_outer.clone();
                    row()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children(move || {
                            let mut has_content = false;

                            if let Some(icon_content) = leading_icon.clone() {
                                has_content = true;
                                match icon_content.clone() {
                                    IconContent::Vector(data) => {
                                        icon_component()
                                            .vector(data)
                                            .size(SegmentedButtonDefaults::ICON_SIZE);
                                    }
                                    IconContent::Raster(data) => {
                                        icon_component()
                                            .raster(data)
                                            .size(SegmentedButtonDefaults::ICON_SIZE);
                                    }
                                }
                            }

                            if !label.is_empty() {
                                if has_content {
                                    {
                                        spacer().modifier(
                                            Modifier::new().width(SEGMENTED_ICON_SPACING),
                                        );
                                    };
                                }
                                {
                                    text().content(label.clone());
                                };
                            }
                        });
                });
        });
    });
}

/// # single_choice_segmented_button_row
///
/// Lay out segmented buttons for single-selection groups.
///
/// ## Usage
///
/// Use when only one option should be active at a time.
///
/// ## Parameters
///
/// - `modifier` — modifiers applied to the row container.
/// - `overlap` — overlap amount between adjacent segments.
/// - `cross_axis_alignment` — cross-axis alignment for segments.
/// - `equal_width` — whether segments should share equal width.
/// - `content` — row content slot.
///
/// ## Examples
///
/// ```
/// use tessera_components::segmented_buttons::{
///     SegmentedButtonDefaults, segmented_button, single_choice_segmented_button_row,
/// };
/// use tessera_components::theme::{MaterialTheme, material_theme};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     material_theme()
///         .theme(|| MaterialTheme::default())
///         .child(|| {
///             let selected = remember(|| 0usize);
///             single_choice_segmented_button_row().content(move || {
///                 segmented_button()
///                     .label("Day")
///                     .selected(selected.get() == 0)
///                     .shape(SegmentedButtonDefaults::item_shape(
///                         0,
///                         2,
///                         SegmentedButtonDefaults::shape(),
///                     ))
///                     .on_click(|| {});
///                 segmented_button()
///                     .label("Week")
///                     .selected(selected.get() == 1)
///                     .shape(SegmentedButtonDefaults::item_shape(
///                         1,
///                         2,
///                         SegmentedButtonDefaults::shape(),
///                     ))
///                     .on_click(|| {});
///             });
///             selected.with_mut(|value| *value = 0);
///             assert_eq!(selected.get(), 0);
///         });
/// }
///
/// demo();
/// ```
#[tessera]
pub fn single_choice_segmented_button_row(
    modifier: Option<Modifier>,
    overlap: Option<Dp>,
    cross_axis_alignment: Option<CrossAxisAlignment>,
    equal_width: Option<bool>,
    content: Option<RenderSlot>,
) {
    let (modifier, layout_policy, content) = segmented_button_row_parts(
        modifier,
        overlap,
        cross_axis_alignment,
        equal_width,
        content,
    );
    let modifier = modifier
        .focus_group()
        .focus_traversal_policy(FocusTraversalPolicy::horizontal().wrap(true));
    layout().modifier(modifier).child(move || {
        let content = content;
        layout()
            .layout_policy(layout_policy.clone())
            .child(move || {
                let content = content;
                provide_context(
                    || SegmentedButtonRowContext {
                        select_on_focus: true,
                    },
                    move || {
                        content.render();
                    },
                );
            });
    });
}

/// # multi_choice_segmented_button_row
///
/// Lay out segmented buttons for multi-selection groups.
///
/// ## Usage
///
/// Use when multiple options can be active simultaneously.
///
/// ## Parameters
///
/// - `modifier` — modifiers applied to the row container.
/// - `overlap` — overlap amount between adjacent segments.
/// - `cross_axis_alignment` — cross-axis alignment for segments.
/// - `equal_width` — whether segments should share equal width.
/// - `content` — row content slot.
///
/// ## Examples
///
/// ```
/// use tessera_components::segmented_buttons::{
///     SegmentedButtonDefaults, multi_choice_segmented_button_row, segmented_button,
/// };
/// use tessera_components::theme::{MaterialTheme, material_theme};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     material_theme()
///         .theme(|| MaterialTheme::default())
///         .child(|| {
///             let selected = remember(|| [true, false]);
///             multi_choice_segmented_button_row().content(move || {
///                 segmented_button()
///                     .label("Email")
///                     .selected(selected.get()[0])
///                     .shape(SegmentedButtonDefaults::item_shape(
///                         0,
///                         2,
///                         SegmentedButtonDefaults::shape(),
///                     ))
///                     .on_click(|| {});
///                 segmented_button()
///                     .label("Push")
///                     .selected(selected.get()[1])
///                     .shape(SegmentedButtonDefaults::item_shape(
///                         1,
///                         2,
///                         SegmentedButtonDefaults::shape(),
///                     ))
///                     .on_click(|| {});
///             });
///             selected.with_mut(|value| value[1] = true);
///             assert!(selected.get()[1]);
///         });
/// }
///
/// demo();
/// ```
#[tessera]
pub fn multi_choice_segmented_button_row(
    modifier: Option<Modifier>,
    overlap: Option<Dp>,
    cross_axis_alignment: Option<CrossAxisAlignment>,
    equal_width: Option<bool>,
    content: Option<RenderSlot>,
) {
    let (modifier, layout_policy, content) = segmented_button_row_parts(
        modifier,
        overlap,
        cross_axis_alignment,
        equal_width,
        content,
    );
    let modifier = modifier
        .focus_group()
        .focus_traversal_policy(FocusTraversalPolicy::horizontal().wrap(true));
    layout().modifier(modifier).child(move || {
        let content = content;
        layout()
            .layout_policy(layout_policy.clone())
            .child(move || {
                let content = content;
                provide_context(
                    || SegmentedButtonRowContext {
                        select_on_focus: false,
                    },
                    move || {
                        content.render();
                    },
                );
            });
    });
}

fn segmented_button_row_parts(
    modifier: Option<Modifier>,
    overlap: Option<Dp>,
    cross_axis_alignment: Option<CrossAxisAlignment>,
    equal_width: Option<bool>,
    content: Option<RenderSlot>,
) -> (Modifier, SegmentedButtonRowLayout, RenderSlot) {
    let modifier = modifier.unwrap_or_default();
    let content = content.unwrap_or_else(RenderSlot::empty);
    let overlap = Px::from(overlap.unwrap_or(SegmentedButtonDefaults::BORDER_WIDTH)).max(Px::ZERO);
    let cross_axis_alignment = cross_axis_alignment.unwrap_or(CrossAxisAlignment::Center);
    let equal_width = equal_width.unwrap_or(true);
    let modifier = modifier.size_in(None, None, Some(SegmentedButtonDefaults::HEIGHT), None);
    let layout_policy = SegmentedButtonRowLayout {
        overlap,
        cross_axis_alignment,
        equal_width,
    };
    (modifier, layout_policy, content)
}

#[derive(Clone, PartialEq)]
struct SegmentedButtonRowLayout {
    overlap: Px,
    cross_axis_alignment: CrossAxisAlignment,
    equal_width: bool,
}

impl LayoutPolicy for SegmentedButtonRowLayout {
    fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        if children.is_empty() {
            return Ok(result.with_size(ComputedData::ZERO));
        }

        let row_constraint = *input.parent_constraint().as_ref();
        let child_constraint = Constraint::new(
            AxisConstraint::new(Px::ZERO, row_constraint.width.resolve_max()),
            row_constraint.height,
        );

        let mut children_sizes = vec![None; children.len()];
        let mut total_width = Px::ZERO;
        let mut max_width = Px::ZERO;
        let mut max_height = Px::ZERO;
        for (index, &child) in children.iter().enumerate() {
            let child_result = child.measure(&child_constraint)?;
            children_sizes[index] = Some(child_result.size());
            total_width += child_result.width;
            max_width = max_width.max(child_result.width);
            max_height = max_height.max(child_result.height);
        }

        if self.equal_width && max_width > Px::ZERO {
            let equal_constraint = Constraint::new(max_width, row_constraint.height);
            total_width = Px::ZERO;
            max_height = Px::ZERO;

            for (index, &child) in children.iter().enumerate() {
                let child_result = child.measure(&equal_constraint)?;
                children_sizes[index] = Some(child_result.size());
                total_width += child_result.width;
                max_height = max_height.max(child_result.height);
            }
        }

        let min_child_width = children_sizes
            .iter()
            .filter_map(|size| size.map(|size| size.width))
            .min()
            .unwrap_or(Px::ZERO);
        let overlap = if children.len() > 1 {
            self.overlap.min(min_child_width)
        } else {
            Px::ZERO
        };
        let total_width_with_overlap = if children.len() > 1 {
            total_width - overlap * (children.len() as i32 - 1)
        } else {
            total_width
        };
        let total_width_with_overlap = total_width_with_overlap.max(Px::ZERO);
        let final_width = calculate_final_row_width(&row_constraint, total_width_with_overlap);
        let final_height = calculate_final_row_height(&row_constraint, max_height);

        let mut current_x = Px::ZERO;
        for (index, child_size_opt) in children_sizes.iter().enumerate() {
            if let Some(child_size) = child_size_opt {
                let child = children[index];
                let y_offset = calculate_cross_axis_offset(
                    child_size,
                    final_height,
                    self.cross_axis_alignment,
                );
                result.place_child(child, PxPosition::new(current_x, y_offset));
                current_x += child_size.width;
                if index < children.len() - 1 {
                    current_x -= overlap;
                }
            }
        }

        Ok(result.with_size(ComputedData {
            width: final_width,
            height: final_height,
        }))
    }
}

fn calculate_final_row_width(
    row_effective_constraint: &Constraint,
    total_children_measured_width: Px,
) -> Px {
    row_effective_constraint
        .width
        .clamp(total_children_measured_width)
}

fn calculate_final_row_height(row_effective_constraint: &Constraint, max_child_height: Px) -> Px {
    row_effective_constraint.height.clamp(max_child_height)
}

fn calculate_cross_axis_offset(
    child_actual_size: &ComputedData,
    final_row_height: Px,
    cross_axis_alignment: CrossAxisAlignment,
) -> Px {
    match cross_axis_alignment {
        CrossAxisAlignment::Start => Px::ZERO,
        CrossAxisAlignment::Center => {
            (final_row_height - child_actual_size.height).max(Px::ZERO) / 2
        }
        CrossAxisAlignment::End => (final_row_height - child_actual_size.height).max(Px::ZERO),
        CrossAxisAlignment::Stretch => Px::ZERO,
    }
}
