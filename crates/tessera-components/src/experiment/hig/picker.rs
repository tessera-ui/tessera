//! Pickers for choosing one option from a set.
//!
//! ## Usage
//!
//! Choose a mode, sort order, or form value from a short list of mutually exclusive options.

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    CallbackWith, ComputedData, Dp, LayoutPolicy, LayoutResult, MeasurementError, Modifier, Px,
    PxPosition,
    layout::{MeasureScope, layout},
    tessera, use_context,
};

use crate::{alignment::CrossAxisAlignment, column::column, row::row, spacer::spacer};

use super::{
    radio_button::{RADIO_ROW_SPACING, radio_button, radio_group},
    segmented_control::{Segment, segmented_control},
    text::text,
    theme::{HigTheme, TextStyle},
};

#[cfg(test)]
#[path = "picker_tests.rs"]
mod tests;

/// Rounds its child's size up to whole points and centres the child, snapped to device pixels.
#[derive(Clone, Copy, PartialEq)]
struct WholePoints;

impl LayoutPolicy for WholePoints {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = *scope.parent_constraint().as_ref();
        let point = Dp(1.0).to_pixels_f32();
        let round_up = |px: Px| Px(((px.to_f32() / point).ceil() * point).round() as i32);
        let children = scope.children();
        let mut size = ComputedData {
            width: Px::ZERO,
            height: Px::ZERO,
        };
        let mut measured = Vec::with_capacity(children.len());
        for child in &children {
            let child_size = child.measure(scope, &parent)?;
            size.width = size.width.max(round_up(child_size.width));
            size.height = size.height.max(round_up(child_size.height));
            measured.push(child_size);
        }
        size.width = parent.width.clamp(size.width);
        size.height = parent.height.clamp(size.height);
        let mut result = LayoutResult::new(size);
        for (child, child_size) in children.into_iter().zip(measured) {
            let offset = |outer: Px, inner: Px| Px(((outer - inner).0.max(0) + 1) / 2);
            result.place_child(
                child,
                PxPosition::new(
                    offset(size.width, child_size.width),
                    offset(size.height, child_size.height),
                ),
            );
        }
        Ok(result)
    }
}

/// The appearance of a [`picker`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PickerStyle {
    /// A row of joined segments with the selected one highlighted.
    #[default]
    Segmented,
    /// A column of radio buttons.
    RadioGroup,
    /// The options shown in place; a radio group on macOS.
    Inline,
    /// A compact row of options; segments on macOS.
    Palette,
}

/// # picker
///
/// Chooses one option from a set as segments or radio buttons.
///
/// ## Usage
///
/// Pick a delivery method, view mode, or other setting from a few mutually exclusive options.
///
/// ## Parameters
///
/// - `selection`, `on_select` — index of the chosen option and requested changes.
/// - `options` — option titles, in order.
/// - `label` — leading title and default accessible name; omit it to hide the label.
/// - `style` — segments or radio buttons; defaults to [`PickerStyle::Segmented`].
/// - `enabled`, `window_active` — input availability and containing window appearance.
/// - `modifier` — layout and additional semantics.
/// - `accessibility_label`, `accessibility_description` — spoken content overrides.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::picker::{PickerStyle, picker};
/// use tessera_ui::{remember, tessera};
/// # #[tessera]
/// # fn component() {
///
/// let delivery = remember(|| 0);
/// picker(delivery.get(), vec!["Standard".into(), "Express".into()])
///     .label("Delivery")
///     .style(PickerStyle::RadioGroup)
///     .on_select(move |index| delivery.set(index));
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn picker(
    selection: usize,
    options: Vec<String>,
    on_select: Option<CallbackWith<usize>>,
    #[prop(into)] label: Option<String>,
    style: Option<PickerStyle>,
    enabled: Option<bool>,
    window_active: Option<bool>,
    modifier: Option<Modifier>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let label = label.filter(|label| !label.is_empty());
    let spoken = accessibility_label.or_else(|| label.clone());
    let radio = matches!(
        style.unwrap_or_default(),
        PickerStyle::RadioGroup | PickerStyle::Inline
    );
    layout(WholePoints)
        .modifier(modifier.unwrap_or_default())
        .child(move || {
            picker_content(selection, options.clone(), radio)
                .on_select_optional(on_select)
                .label_optional(label.clone())
                .enabled_optional(enabled)
                .window_active_optional(window_active)
                .spoken_optional(spoken.clone())
                .accessibility_description_optional(accessibility_description.clone());
        });
}

#[tessera]
fn picker_content(
    selection: usize,
    options: Vec<String>,
    on_select: Option<CallbackWith<usize>>,
    label: Option<String>,
    radio: bool,
    enabled: Option<bool>,
    window_active: Option<bool>,
    spoken: Option<String>,
    accessibility_description: Option<String>,
) {
    row()
        .cross_axis_alignment(if radio {
            CrossAxisAlignment::Start
        } else {
            CrossAxisAlignment::Center
        })
        .children(move || {
            if let Some(label) = label.clone() {
                picker_label(label);
                spacer().modifier(Modifier::new().width(Dp(8.0)));
            }
            let options = options.clone();
            if radio {
                radio_group()
                    .accessibility_label_optional(spoken.clone())
                    .accessibility_description_optional(accessibility_description.clone())
                    .child(move |group| {
                        let options = options.clone();
                        column().children(move || {
                            for (index, option) in options.iter().enumerate() {
                                if index > 0 {
                                    spacer().modifier(Modifier::new().height(RADIO_ROW_SPACING));
                                }
                                let button = radio_button(selection == index)
                                    .group(group)
                                    .label(option.clone())
                                    .enabled_optional(enabled)
                                    .window_active_optional(window_active);
                                if let Some(on_select) = on_select {
                                    button.on_select(move || on_select.call(index));
                                }
                            }
                        });
                    });
            } else {
                let control = segmented_control(options.into_iter().map(Segment::new).collect())
                    .selected(selection)
                    .enabled_optional(enabled)
                    .window_active_optional(window_active)
                    .accessibility_label_optional(spoken.clone())
                    .accessibility_description_optional(accessibility_description.clone());
                if let Some(on_select) = on_select {
                    control.on_select_shared(on_select);
                }
            }
        });
}

#[tessera]
fn picker_label(content: String) {
    let theme = use_context::<HigTheme>()
        .map(|theme| theme.get())
        .unwrap_or_default();
    layout(WholePoints).child(move || {
        text(content.clone())
            .style(TextStyle {
                line_height: Dp(16.0),
                ..Default::default()
            })
            .color(theme.label_color())
            .single_line(true)
            .modifier(Modifier::new().clear_and_set_semantics(SemanticsArgs {
                hidden: true,
                ..Default::default()
            }));
    });
}
