//! Tab views that switch between pages of content.
//!
//! ## Usage
//!
//! Divide preferences or inspectors into pages chosen from a row of tabs.

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    AxisConstraint, CallbackWith, Color, ComputedData, Constraint, Dp, LayoutPolicy, LayoutResult,
    MeasurementError, Modifier, Px, PxPosition, RenderSlotWith,
    accesskit::Role,
    layout::{MeasureScope, Stack, layout},
    tessera, use_context,
};

use crate::{modifier::ModifierExt as _, shape_def::Shape};

use super::{
    segmented_control::{Segment, segmented_control},
    theme::{Appearance, HigTheme},
};

#[cfg(test)]
#[path = "tab_view_tests.rs"]
mod tests;

/// Places the tabs centred on the top edge of the page box.
#[derive(Clone, Copy, PartialEq)]
struct TabLayout;

impl LayoutPolicy for TabLayout {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = *scope.parent_constraint().as_ref();
        let children = scope.children();
        let [tabs, page] = children.as_slice() else {
            return Ok(LayoutResult::new(ComputedData::min_from_constraint(
                &parent,
            )));
        };
        let tab_size = tabs
            .measure(
                scope,
                &Constraint::new(
                    AxisConstraint::new(Px::ZERO, parent.width.resolve_max()),
                    AxisConstraint::new(Px::ZERO, None),
                ),
            )?
            .size();
        let overlap = Px(tab_size.height.0 / 2);
        let page_height = match parent.height.resolve_max() {
            Some(max) => AxisConstraint::exact((max - overlap).max(Px::ZERO)),
            None => AxisConstraint::new(Px::ZERO, None),
        };
        let page_size = page
            .measure(
                scope,
                &Constraint::new(
                    match parent.width.resolve_max() {
                        Some(max) => AxisConstraint::exact(max),
                        None => AxisConstraint::new(tab_size.width, None),
                    },
                    page_height,
                ),
            )?
            .size();
        let width = page_size.width.max(tab_size.width);
        let mut result = LayoutResult::new(ComputedData {
            width,
            height: page_size.height + overlap,
        });
        result.place_child(*page, PxPosition::new(Px::ZERO, overlap));
        result.place_child(
            *tabs,
            PxPosition::new(Px((width - tab_size.width).0 / 2), Px::ZERO),
        );
        Ok(result)
    }
}

/// # tab_view
///
/// Switches between pages of content with a row of tabs above a box.
///
/// ## Usage
///
/// Divide a settings window or inspector into General, Appearance, and Advanced pages.
///
/// ## Parameters
///
/// - `selection`, `on_select` — index of the shown page and requested changes.
/// - `tabs` — tab titles, in order.
/// - `content` — the page for a tab index.
/// - `modifier` — size and placement.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{tab_view::tab_view, text::text};
/// use tessera_ui::{remember, tessera};
/// # #[tessera]
/// # fn component() {
///
/// let page = remember(|| 0usize);
/// tab_view(page.get(), vec!["General".into(), "Advanced".into()])
///     .on_select(move |index| page.set(index))
///     .content(|index| {
///         text(if index == 0 { "General settings" } else { "Advanced settings" });
///     });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn tab_view(
    selection: usize,
    tabs: Vec<String>,
    on_select: Option<CallbackWith<usize>>,
    content: Option<RenderSlotWith<usize>>,
    modifier: Option<Modifier>,
) {
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    let tint = if dark { Color::WHITE } else { Color::BLACK };
    layout(TabLayout)
        .modifier(modifier.unwrap_or_default().semantics(SemanticsArgs {
            role: Some(Role::TabList),
            ..Default::default()
        }))
        .child(move || {
            let control = segmented_control(tabs.iter().cloned().map(Segment::new).collect())
                .selected(selection);
            if let Some(on_select) = on_select {
                control.on_select_shared(on_select);
            }
            tab_page(selection, tint).content_optional(content);
        });
}

#[tessera]
fn tab_page(selection: usize, tint: Color, content: Option<RenderSlotWith<usize>>) {
    let shape = Shape::rounded_rectangle(Dp(8.0));
    layout(Stack)
        .modifier(
            Modifier::new()
                .background_with_shape(tint.with_alpha(0.04), shape)
                .border_with_shape(Dp(1.0), tint.with_alpha(0.08), shape)
                .padding_symmetric(Dp(16.0), Dp(20.0))
                .semantics(SemanticsArgs {
                    role: Some(Role::TabPanel),
                    ..Default::default()
                }),
        )
        .child(move || {
            if let Some(content) = content {
                content.render(selection);
            }
        });
}
