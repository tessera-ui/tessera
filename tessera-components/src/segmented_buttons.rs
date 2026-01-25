//! Material Design segmented buttons for compact selections.
//!
//! ## Usage
//!
//! Switch between views or filters with a connected control.

use std::sync::Arc;

use derive_setters::Setters;

use tessera_ui::{
    Color, ComputedData, Constraint, DimensionValue, Dp, LayoutInput, LayoutOutput, LayoutSpec,
    MeasurementError, Modifier, Px, PxPosition, accesskit::Role, tessera, use_context,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment},
    icon::{IconArgs, icon},
    modifier::{ModifierExt as _, Padding},
    row::{RowArgs, row},
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    surface::{SurfaceArgs, SurfaceStyle, surface},
    text::{TextArgs, text},
    theme::{MaterialAlpha, MaterialTheme, provide_text_style},
};

const SEGMENTED_ICON_SPACING: Dp = Dp(8.0);

/// Color values for segmented buttons in different states.
#[derive(Clone, Copy, Debug)]
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
        Shape::capsule()
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

/// Arguments for [`segmented_button`].
#[derive(Clone, Setters)]
pub struct SegmentedButtonArgs {
    /// Whether the button is selected.
    pub selected: bool,
    /// Whether the button is enabled.
    pub enabled: bool,
    /// Text label for the segment.
    #[setters(into)]
    pub label: String,
    /// Optional icon displayed before the label.
    #[setters(strip_option)]
    pub icon: Option<IconArgs>,
    /// Modifier chain applied to the button.
    pub modifier: Modifier,
    /// Shape of the segment.
    pub shape: Shape,
    /// Optional color overrides.
    #[setters(strip_option)]
    pub colors: Option<SegmentedButtonColors>,
    /// Border width for the segment.
    pub border_width: Dp,
    /// Padding inside the segment.
    pub content_padding: Padding,
    /// Click handler for the segment.
    #[setters(skip)]
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Optional accessibility label.
    #[setters(strip_option, into)]
    pub accessibility_label: Option<String>,
    /// Optional accessibility description.
    #[setters(strip_option, into)]
    pub accessibility_description: Option<String>,
}

impl SegmentedButtonArgs {
    /// Create arguments with the required label.
    pub fn new(label: impl Into<String>) -> Self {
        Self::default().label(label)
    }

    /// Set the click handler.
    pub fn on_click<F>(mut self, on_click: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Arc::new(on_click));
        self
    }

    /// Set the click handler using a shared callback.
    pub fn on_click_shared(mut self, on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.on_click = Some(on_click);
        self
    }
}

impl Default for SegmentedButtonArgs {
    fn default() -> Self {
        Self {
            selected: false,
            enabled: true,
            label: String::new(),
            icon: None,
            modifier: Modifier::new(),
            shape: SegmentedButtonDefaults::shape(),
            colors: None,
            border_width: SegmentedButtonDefaults::BORDER_WIDTH,
            content_padding: SegmentedButtonDefaults::CONTENT_PADDING,
            on_click: None,
            accessibility_label: None,
            accessibility_description: None,
        }
    }
}

/// Arguments for segmented button rows.
#[derive(Clone, Setters)]
pub struct SegmentedButtonRowArgs {
    /// Modifier chain applied to the row.
    pub modifier: Modifier,
    /// Overlap amount between adjacent segments.
    pub overlap: Dp,
    /// Cross-axis alignment for segments.
    pub cross_axis_alignment: CrossAxisAlignment,
    /// Whether segments should be forced to equal width.
    pub equal_width: bool,
}

impl Default for SegmentedButtonRowArgs {
    fn default() -> Self {
        Self {
            modifier: Modifier::new()
                .constrain(Some(DimensionValue::WRAP), Some(DimensionValue::WRAP)),
            overlap: SegmentedButtonDefaults::BORDER_WIDTH,
            cross_axis_alignment: CrossAxisAlignment::Center,
            equal_width: true,
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
/// - `args` — configures selection state, label, and appearance; see
///   [`SegmentedButtonArgs`].
///
/// ## Examples
///
/// ```
/// use tessera_components::segmented_buttons::{
///     SegmentedButtonArgs, SegmentedButtonDefaults, SegmentedButtonRowArgs, segmented_button,
///     single_choice_segmented_button_row,
/// };
/// use tessera_components::theme::{MaterialTheme, material_theme};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     material_theme(
///         || MaterialTheme::default(),
///         || {
///             let selected = remember(|| 0usize);
///             single_choice_segmented_button_row(SegmentedButtonRowArgs::default(), || {
///                 segmented_button(
///                     SegmentedButtonArgs::new("List")
///                         .selected(selected.get() == 0)
///                         .shape(SegmentedButtonDefaults::item_shape(
///                             0,
///                             2,
///                             SegmentedButtonDefaults::shape(),
///                         ))
///                         .on_click(|| {}),
///                 );
///                 segmented_button(
///                     SegmentedButtonArgs::new("Grid")
///                         .selected(selected.get() == 1)
///                         .shape(SegmentedButtonDefaults::item_shape(
///                             1,
///                             2,
///                             SegmentedButtonDefaults::shape(),
///                         ))
///                         .on_click(|| {}),
///                 );
///             });
///             selected.with_mut(|value| *value = 1);
///             assert_eq!(selected.get(), 1);
///         },
///     );
/// }
///
/// demo();
/// ```
#[tessera]
pub fn segmented_button(args: impl Into<SegmentedButtonArgs>) {
    let args: SegmentedButtonArgs = args.into();
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let typography = theme.typography;
    let colors = args.colors.unwrap_or_else(SegmentedButtonDefaults::colors);
    let container_color = colors.container_color(args.enabled, args.selected);
    let content_color = colors.content_color(args.enabled, args.selected);
    let border_color = colors.border_color(args.enabled, args.selected);
    let border_width = args.border_width;

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

    let label = args.label;
    let mut surface_args = SurfaceArgs::default()
        .modifier(
            args.modifier
                .size_in(None, None, Some(SegmentedButtonDefaults::HEIGHT), None),
        )
        .style(surface_style)
        .shape(args.shape)
        .content_alignment(Alignment::Center)
        .content_color(content_color)
        .enabled(args.enabled)
        .ripple_color(content_color);

    if let Some(on_click) = args.on_click {
        surface_args = surface_args
            .on_click_shared(on_click)
            .accessibility_role(Role::Button)
            .accessibility_focusable(true);
    }

    let accessibility_label = args
        .accessibility_label
        .or_else(|| (!label.is_empty()).then(|| label.clone()));
    if let Some(label) = accessibility_label {
        surface_args = surface_args.accessibility_label(label);
    }
    if let Some(description) = args.accessibility_description {
        surface_args = surface_args.accessibility_description(description);
    }

    let icon_args = args.icon.map(|mut icon_args| {
        icon_args.size = SegmentedButtonDefaults::ICON_SIZE;
        icon_args
    });

    surface(surface_args, move || {
        provide_text_style(typography.label_large, move || {
            Modifier::new().padding(args.content_padding).run(move || {
                row(
                    RowArgs::default().cross_axis_alignment(CrossAxisAlignment::Center),
                    move |scope| {
                        let mut has_content = false;

                        if let Some(icon_args) = icon_args {
                            has_content = true;
                            scope.child(move || {
                                icon(icon_args);
                            });
                        }

                        if !label.is_empty() {
                            if has_content {
                                scope.child(move || {
                                    spacer(Modifier::new().width(SEGMENTED_ICON_SPACING));
                                });
                            }
                            scope.child(move || {
                                text(TextArgs::default().text(label));
                            });
                        }
                    },
                );
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
/// - `args` — configures overlap and sizing; see [`SegmentedButtonRowArgs`].
/// - `content` — renders the segmented buttons within the row.
///
/// ## Examples
///
/// ```
/// use tessera_components::segmented_buttons::{
///     SegmentedButtonArgs, SegmentedButtonDefaults, SegmentedButtonRowArgs, segmented_button,
///     single_choice_segmented_button_row,
/// };
/// use tessera_components::theme::{MaterialTheme, material_theme};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     material_theme(
///         || MaterialTheme::default(),
///         || {
///             let selected = remember(|| 0usize);
///             single_choice_segmented_button_row(SegmentedButtonRowArgs::default(), || {
///                 segmented_button(
///                     SegmentedButtonArgs::new("Day")
///                         .selected(selected.get() == 0)
///                         .shape(SegmentedButtonDefaults::item_shape(
///                             0,
///                             2,
///                             SegmentedButtonDefaults::shape(),
///                         ))
///                         .on_click(|| {}),
///                 );
///                 segmented_button(
///                     SegmentedButtonArgs::new("Week")
///                         .selected(selected.get() == 1)
///                         .shape(SegmentedButtonDefaults::item_shape(
///                             1,
///                             2,
///                             SegmentedButtonDefaults::shape(),
///                         ))
///                         .on_click(|| {}),
///                 );
///             });
///             selected.with_mut(|value| *value = 0);
///             assert_eq!(selected.get(), 0);
///         },
///     );
/// }
///
/// demo();
/// ```
#[tessera]
pub fn single_choice_segmented_button_row(
    args: impl Into<SegmentedButtonRowArgs>,
    content: impl FnOnce() + Send + Sync + 'static,
) {
    segmented_button_row_impl(args.into(), content);
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
/// - `args` — configures overlap and sizing; see [`SegmentedButtonRowArgs`].
/// - `content` — renders the segmented buttons within the row.
///
/// ## Examples
///
/// ```
/// use tessera_components::segmented_buttons::{
///     SegmentedButtonArgs, SegmentedButtonDefaults, SegmentedButtonRowArgs,
///     multi_choice_segmented_button_row, segmented_button,
/// };
/// use tessera_components::theme::{MaterialTheme, material_theme};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     material_theme(
///         || MaterialTheme::default(),
///         || {
///             let selected = remember(|| [true, false]);
///             multi_choice_segmented_button_row(SegmentedButtonRowArgs::default(), || {
///                 segmented_button(
///                     SegmentedButtonArgs::new("Email")
///                         .selected(selected.get()[0])
///                         .shape(SegmentedButtonDefaults::item_shape(
///                             0,
///                             2,
///                             SegmentedButtonDefaults::shape(),
///                         ))
///                         .on_click(|| {}),
///                 );
///                 segmented_button(
///                     SegmentedButtonArgs::new("Push")
///                         .selected(selected.get()[1])
///                         .shape(SegmentedButtonDefaults::item_shape(
///                             1,
///                             2,
///                             SegmentedButtonDefaults::shape(),
///                         ))
///                         .on_click(|| {}),
///                 );
///             });
///             selected.with_mut(|value| value[1] = true);
///             assert!(selected.get()[1]);
///         },
///     );
/// }
///
/// demo();
/// ```
#[tessera]
pub fn multi_choice_segmented_button_row(
    args: impl Into<SegmentedButtonRowArgs>,
    content: impl FnOnce() + Send + Sync + 'static,
) {
    segmented_button_row_impl(args.into(), content);
}

fn segmented_button_row_impl(
    args: SegmentedButtonRowArgs,
    content: impl FnOnce() + Send + Sync + 'static,
) {
    let modifier = args
        .modifier
        .size_in(None, None, Some(SegmentedButtonDefaults::HEIGHT), None);
    modifier.run(move || segmented_button_row_inner(args, content));
}

#[tessera]
fn segmented_button_row_inner(
    args: SegmentedButtonRowArgs,
    content: impl FnOnce() + Send + Sync + 'static,
) {
    let overlap = Px::from(args.overlap).max(Px::ZERO);
    layout(SegmentedButtonRowLayout {
        overlap,
        cross_axis_alignment: args.cross_axis_alignment,
        equal_width: args.equal_width,
    });
    content();
}

#[derive(Clone, PartialEq)]
struct SegmentedButtonRowLayout {
    overlap: Px,
    cross_axis_alignment: CrossAxisAlignment,
    equal_width: bool,
}

impl LayoutSpec for SegmentedButtonRowLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let child_ids = input.children_ids();
        if child_ids.is_empty() {
            return Ok(ComputedData::ZERO);
        }

        let row_constraint = Constraint::new(
            input.parent_constraint().width(),
            input.parent_constraint().height(),
        );
        let child_constraint = Constraint::new(
            DimensionValue::Wrap {
                min: None,
                max: row_constraint.width.get_max(),
            },
            row_constraint.height,
        );

        let children_to_measure: Vec<_> = child_ids
            .iter()
            .map(|&child_id| (child_id, child_constraint))
            .collect();
        let mut children_sizes = vec![None; child_ids.len()];
        let mut total_width = Px::ZERO;
        let mut max_width = Px::ZERO;
        let mut max_height = Px::ZERO;
        let results = input.measure_children(children_to_measure)?;

        for (index, &child_id) in child_ids.iter().enumerate() {
            if let Some(child_result) = results.get(&child_id) {
                children_sizes[index] = Some(*child_result);
                total_width += child_result.width;
                max_width = max_width.max(child_result.width);
                max_height = max_height.max(child_result.height);
            }
        }

        if self.equal_width && max_width > Px::ZERO {
            let equal_constraint =
                Constraint::new(DimensionValue::Fixed(max_width), row_constraint.height);
            let children_to_measure: Vec<_> = child_ids
                .iter()
                .map(|&child_id| (child_id, equal_constraint))
                .collect();
            let results = input.measure_children(children_to_measure)?;
            total_width = Px::ZERO;
            max_height = Px::ZERO;

            for (index, &child_id) in child_ids.iter().enumerate() {
                if let Some(child_result) = results.get(&child_id) {
                    children_sizes[index] = Some(*child_result);
                    total_width += child_result.width;
                    max_height = max_height.max(child_result.height);
                }
            }
        }

        let min_child_width = children_sizes
            .iter()
            .filter_map(|size| size.map(|size| size.width))
            .min()
            .unwrap_or(Px::ZERO);
        let overlap = if child_ids.len() > 1 {
            self.overlap.min(min_child_width)
        } else {
            Px::ZERO
        };
        let total_width_with_overlap = if child_ids.len() > 1 {
            total_width - overlap * (child_ids.len() as i32 - 1)
        } else {
            total_width
        };
        let total_width_with_overlap = total_width_with_overlap.max(Px::ZERO);
        let final_width = calculate_final_row_width(&row_constraint, total_width_with_overlap);
        let final_height = calculate_final_row_height(&row_constraint, max_height);

        let mut current_x = Px::ZERO;
        for (index, child_size_opt) in children_sizes.iter().enumerate() {
            if let Some(child_size) = child_size_opt {
                let child_id = child_ids[index];
                let y_offset = calculate_cross_axis_offset(
                    child_size,
                    final_height,
                    self.cross_axis_alignment,
                );
                output.place_child(child_id, PxPosition::new(current_x, y_offset));
                current_x += child_size.width;
                if index < child_ids.len() - 1 {
                    current_x -= overlap;
                }
            }
        }

        Ok(ComputedData {
            width: final_width,
            height: final_height,
        })
    }
}

fn calculate_final_row_width(
    row_effective_constraint: &Constraint,
    total_children_measured_width: Px,
) -> Px {
    match row_effective_constraint.width {
        DimensionValue::Fixed(w) => w,
        DimensionValue::Fill { min, max } => {
            if let Some(max) = max {
                let w = max;
                if let Some(min) = min { w.max(min) } else { w }
            } else {
                panic!(
                    "Fill width without max constraint is not supported in segmented button rows."
                );
            }
        }
        DimensionValue::Wrap { min, max } => {
            let mut w = total_children_measured_width;
            if let Some(min_w) = min {
                w = w.max(min_w);
            }
            if let Some(max_w) = max {
                w = w.min(max_w);
            }
            w
        }
    }
}

fn calculate_final_row_height(row_effective_constraint: &Constraint, max_child_height: Px) -> Px {
    match row_effective_constraint.height {
        DimensionValue::Fixed(h) => h,
        DimensionValue::Fill { min, max } => {
            if let Some(max_h) = max {
                let h = max_h;
                if let Some(min_h) = min {
                    h.max(min_h)
                } else {
                    h
                }
            } else {
                panic!(
                    "Fill height without max constraint is not supported in segmented button rows."
                );
            }
        }
        DimensionValue::Wrap { min, max } => {
            let mut h = max_child_height;
            if let Some(min_h) = min {
                h = h.max(min_h);
            }
            if let Some(max_h) = max {
                h = h.min(max_h);
            }
            h
        }
    }
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
