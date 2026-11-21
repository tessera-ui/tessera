//! Disclosure groups that show and hide content under a label.
//!
//! ## Usage
//!
//! Hide advanced settings or details until people ask to see them.

use tessera_foundation::modifier::{ClickableArgs, ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Callback, CallbackWith, ComputedData, Constraint, Dp, LayoutPolicy, LayoutResult,
    MeasurementError, Modifier, Px, PxPosition, RenderSlot, RenderSlotWith,
    accesskit::Role,
    launch,
    layout::{MeasureScope, layout},
    remember, tessera,
};

use crate::{
    alignment::CrossAxisAlignment, animation::animate_to, column::column, row::row, spacer::spacer,
};

use super::{
    disclosure_button::disclosure_button,
    motion::{DISCLOSURE, Eased},
    text::text,
};

#[cfg(test)]
#[path = "disclosure_group_tests.rs"]
mod tests;

/// Indent of disclosed content, aligning it with the label.
const INDENT: f64 = 19.0;

/// Shows its child clipped to `fraction` of the child's height.
#[derive(Clone, Copy, PartialEq)]
struct Reveal(f32);

impl LayoutPolicy for Reveal {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = *scope.parent_constraint().as_ref();
        let constraint = Constraint::new(
            parent.width,
            tessera_ui::AxisConstraint::new(Px::ZERO, None),
        );
        let mut size = ComputedData::ZERO;
        let children = scope.children();
        for child in &children {
            let child_size = child.measure(scope, &constraint)?.size();
            size.width = size.width.max(child_size.width);
            size.height = size.height.max(child_size.height);
        }
        let height = Px((size.height.to_f32() * self.0).round() as i32);
        let mut result = LayoutResult::new(ComputedData {
            width: size.width,
            height,
        });
        for child in children {
            result.place_child(child, PxPosition::ZERO);
        }
        Ok(result)
    }
}

/// # disclosure_group
///
/// Shows or hides content under a label with a disclosure triangle.
///
/// ## Usage
///
/// Collapse advanced settings, details, or nested items until people need them.
///
/// ## Parameters
///
/// - `label` — title next to the triangle; clicking it also toggles the group.
/// - `content` — the content shown while expanded, indented under the label.
/// - `expanded`, `on_expanded_change` — controlled state and requested changes; without
///   `expanded` the group keeps its own state, starting collapsed.
/// - `modifier` — layout and additional semantics.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{
///     disclosure_group::disclosure_group,
///     toggle::toggle,
/// };
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// disclosure_group("Advanced").content(|| {
///     toggle(false).label("Use hardware acceleration");
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn disclosure_group(
    #[prop(into)] label: String,
    content: Option<RenderSlot>,
    expanded: Option<bool>,
    on_expanded_change: Option<CallbackWith<bool>>,
    modifier: Option<Modifier>,
) {
    let owned = remember(|| false);
    let is_expanded = expanded.unwrap_or_else(|| owned.get());
    let request = CallbackWith::new(move |next: bool| {
        if expanded.is_none() {
            owned.set(next);
        }
        if let Some(callback) = on_expanded_change {
            callback.call(next);
        }
    });
    let target = if is_expanded { 1.0 } else { 0.0 };
    let reveal = remember(|| Eased::new(target, DISCLOSURE));
    launch(is_expanded, animate_to(reveal, target));
    let fraction = reveal.with(Eased::value);
    let content = match content {
        Some(content) => content,
        None => RenderSlot::new(|| {}),
    };
    column()
        .modifier(modifier.unwrap_or_default())
        .children(move || {
            group_header(label.clone(), is_expanded, request);
            if fraction > 0.0 {
                group_content(fraction, content);
            }
        });
}

#[tessera]
fn group_header(label: String, expanded: bool, request: CallbackWith<bool>) {
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            disclosure_button(expanded)
                .accessibility_label(label.clone())
                .on_expanded_change_shared(request);
            spacer().modifier(Modifier::new().width(Dp(4.0)));
            text(label.clone()).single_line(true).modifier(
                Modifier::new()
                    .clickable(ClickableArgs {
                        on_click: Callback::new(move || request.call(!expanded)),
                        ..Default::default()
                    })
                    .clear_and_set_semantics(SemanticsArgs {
                        hidden: true,
                        ..Default::default()
                    }),
            );
        });
}

#[tessera]
fn group_content(fraction: f32, content: RenderSlot) {
    layout(Reveal(fraction))
        .modifier(Modifier::new().clip_to_bounds().semantics(SemanticsArgs {
            role: Some(Role::Group),
            ..Default::default()
        }))
        .child(move || {
            column()
                .modifier(
                    Modifier::new()
                        .padding_symmetric(Dp::ZERO, Dp(2.0))
                        .padding(tessera_foundation::modifier::Padding {
                            left: Dp(INDENT),
                            ..Default::default()
                        }),
                )
                .children_shared(content);
        });
}

/// One node of an [`outline_group`] tree.
#[derive(Clone, Debug, PartialEq)]
pub struct OutlineItem {
    /// Title of the node.
    pub title: String,
    /// Child nodes; a node with children is shown as a disclosure group.
    pub children: Vec<OutlineItem>,
}

impl OutlineItem {
    /// Creates a leaf node.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            children: Vec::new(),
        }
    }

    /// Adds child nodes.
    pub fn with_children(mut self, children: Vec<OutlineItem>) -> Self {
        self.children = children;
        self
    }
}

/// # outline_group
///
/// Shows a tree of items as nested disclosure groups.
///
/// ## Usage
///
/// Browse folders, document outlines, and other hierarchies.
///
/// ## Parameters
///
/// - `items` — the top-level nodes.
/// - `row` — optional content for each node; defaults to its title.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::disclosure_group::{OutlineItem, outline_group};
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// outline_group(vec![
///     OutlineItem::new("Documents").with_children(vec![OutlineItem::new("Report.pdf")]),
///     OutlineItem::new("Downloads"),
/// ]);
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn outline_group(items: Vec<OutlineItem>, row: Option<RenderSlotWith<OutlineItem>>) {
    column().children(move || {
        for item in items.iter().cloned() {
            let node = outline_node(item);
            if let Some(row) = row {
                node.row_shared(row);
            }
        }
    });
}

#[tessera]
fn outline_node(item: OutlineItem, row: Option<RenderSlotWith<OutlineItem>>) {
    if item.children.is_empty() {
        let leaf = leaf(item);
        if let Some(row) = row {
            leaf.row_shared(row);
        }
    } else {
        let children = item.children.clone();
        disclosure_group(item.title.clone()).content(move || {
            let group = outline_group(children.clone());
            if let Some(row) = row {
                group.row_shared(row);
            }
        });
    }
}

#[tessera]
fn leaf(item: OutlineItem, row: Option<RenderSlotWith<OutlineItem>>) {
    let indent = Modifier::new().padding(tessera_foundation::modifier::Padding {
        left: Dp(INDENT),
        top: Dp(2.0),
        bottom: Dp(2.0),
        ..Default::default()
    });
    crate::row::row()
        .modifier(indent)
        .children(move || match row {
            Some(row) => row.render(item.clone()),
            None => {
                text(item.title.clone()).single_line(true);
            }
        });
}
