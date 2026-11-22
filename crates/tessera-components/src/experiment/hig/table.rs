//! Tables of rows and sortable columns.
//!
//! ## Usage
//!
//! Compare files, contacts, or records by several attributes at once.

use tessera_foundation::modifier::{ClickableArgs, ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    AxisConstraint, Callback, CallbackWith, Color, ComputedData, Constraint, Dp,
    DrawModifierContent, DrawModifierContext, DrawModifierNode, FocusDirection, FocusRequester,
    FocusState, FocusTraversalPolicy, LayoutPolicy, LayoutResult, MeasurementError, Modifier, Px,
    PxPosition, RenderSlotWith, SemanticsModifierNode,
    accesskit::Role,
    focus,
    layout::{MeasureScope, Stack, layout},
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    remember, tessera, use_context,
};

use crate::{
    column::column,
    modifier::ModifierExt as _,
    pipelines::perceptual_fill::{PerceptualFillCommand, PerceptualStroke},
};

use super::{
    disclosure_button::disclosure_button,
    scroll_view::scroll_view,
    text::text,
    theme::{Appearance, HigTheme, TextStyle, hig_theme},
};

#[cfg(test)]
#[path = "table_tests.rs"]
mod tests;

const HEADER_HEIGHT: f64 = 28.0;
const ROW_HEIGHT: f64 = 24.0;
const CELL_PADDING: f64 = 6.0;

/// The appearance of a [`table`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TableStyle {
    /// The platform default, an inset table on macOS.
    #[default]
    Automatic,
    /// Rows inset from the edges, with rounded selection.
    Inset,
    /// A table inside a border, with rows edge to edge.
    Bordered,
}

/// A column of a [`table`].
#[derive(Clone, Debug, PartialEq)]
pub struct TableColumn {
    /// Header title and the cells' accessible column name.
    pub title: String,
    /// Fixed width; columns without one share the remaining width.
    pub width: Option<Dp>,
}

impl TableColumn {
    /// Creates a column that shares the remaining width.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            width: None,
        }
    }

    /// Gives the column a fixed width.
    pub fn width(mut self, width: Dp) -> Self {
        self.width = Some(width);
        self
    }
}

/// The column a [`table`] is sorted by, and its direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableSort {
    /// Index of the sorted column.
    pub column: usize,
    /// Whether values increase down the table.
    pub ascending: bool,
}

/// Where a row sits in an outline of expandable rows in a [`table`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TableDisclosure {
    /// Nesting depth; top-level rows are at depth 0.
    pub depth: usize,
    /// Whether the row's children are shown, or `None` for a row without children.
    pub expanded: Option<bool>,
}

/// Indent of one outline level in the first column.
const OUTLINE_INDENT: f64 = 16.0;

/// Lays out cells side by side at their column widths.
#[derive(Clone, PartialEq)]
struct Columns(Vec<Option<Dp>>);

impl Columns {
    fn widths(&self, available: Px) -> Vec<Px> {
        let fixed: Px = self
            .0
            .iter()
            .flatten()
            .map(|width| Px::from(*width))
            .fold(Px::ZERO, |sum, width| sum + width);
        let flexible = self.0.iter().filter(|width| width.is_none()).count();
        let share = if flexible > 0 {
            Px(((available - fixed).0.max(0)) / flexible as i32)
        } else {
            Px::ZERO
        };
        self.0
            .iter()
            .map(|width| width.map(Px::from).unwrap_or(share))
            .collect()
    }
}

impl LayoutPolicy for Columns {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = *scope.parent_constraint().as_ref();
        let width = parent
            .width
            .resolve_max()
            .unwrap_or_else(|| Px::from(Dp(120.0 * self.0.len() as f64)));
        let height = parent.height.clamp(Px::ZERO);
        let widths = self.widths(width);
        let mut result = LayoutResult::new(ComputedData { width, height });
        let mut x = Px::ZERO;
        for (child, column) in scope.children().into_iter().zip(widths) {
            child.measure(
                scope,
                &Constraint::new(AxisConstraint::exact(column), AxisConstraint::exact(height)),
            )?;
            result.place_child(child, PxPosition::new(x, Px::ZERO));
            x += column;
        }
        Ok(result)
    }
}

#[derive(Clone, PartialEq)]
struct Fill {
    color: Color,
    radius: Dp,
    inset: Dp,
}

impl DrawModifierNode for Fill {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        if self.color.a > 0.0 {
            context
                .render_input
                .metadata_mut()
                .fragment_mut()
                .push_draw_command(PerceptualFillCommand {
                    outline_width: Dp::ZERO,
                    snap_to_pixels: false,
                    path: None,
                    color: self.color,
                    radii: [self.radius; 2],
                    insets: [Dp::ZERO, self.inset, Dp::ZERO, self.inset],
                    clip_radius: None,
                    line: None,
                    edge_smoothness: 1.0,
                    stroke: None,
                });
        }
        content.draw(context.render_input);
    }
}

#[derive(Clone, PartialEq)]
struct SortIndicator {
    ascending: bool,
    color: Color,
}

impl DrawModifierNode for SortIndicator {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        let points = if self.ascending {
            [[0.1, 0.85], [0.5, 0.15], [0.9, 0.85]]
        } else {
            [[0.1, 0.15], [0.5, 0.85], [0.9, 0.15]]
        };
        context
            .render_input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(PerceptualFillCommand {
                outline_width: Dp::ZERO,
                snap_to_pixels: false,
                path: None,
                color: Color::TRANSPARENT,
                radii: [Dp::ZERO; 2],
                insets: [Dp::ZERO; 4],
                clip_radius: None,
                line: None,
                edge_smoothness: 1.0,
                stroke: Some(PerceptualStroke {
                    raster_mask: true,
                    color: self.color,
                    width: Dp(1.5),
                    points,
                }),
            });
        content.draw(context.render_input);
    }
}

#[derive(Clone, Copy, PartialEq)]
struct TableContext {
    dark: bool,
}

/// # table
///
/// Shows rows of values under sortable column headers, with optional single selection.
///
/// ## Usage
///
/// Compare files by name, size, and date, or browse other records with several attributes.
///
/// ## Parameters
///
/// - `columns` — column titles and widths, in order.
/// - `row_count` — number of rows.
/// - `cell` — content of the cell at `(row, column)`.
/// - `selection`, `on_select` — selected row and requested changes.
/// - `sort`, `on_sort` — the sorted column and requested changes from header clicks; the caller
///   sorts its rows.
/// - `disclosure`, `on_expanded_change` — outline position of each row and requested expansion
///   changes as `(row, expanded)`; the first column shows indentation and disclosure triangles.
///   The caller lists only the rows whose ancestors are expanded.
/// - `style` — inset or bordered; defaults to [`TableStyle::Automatic`].
/// - `alternating_row_backgrounds` — shades every other row; defaults to true.
/// - `modifier` — size and placement; the table fills the space its container offers.
/// - `accessibility_label` — spoken name of the table.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{
///     table::{TableColumn, table},
///     text::text,
/// };
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// let files = [("Report.pdf", "2 MB"), ("Notes.txt", "4 KB")];
/// table(vec![TableColumn::new("Name"), TableColumn::new("Size")], files.len())
///     .cell(move |(row, column)| {
///         let (name, size) = files[row];
///         text(if column == 0 { name } else { size });
///     });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn table(
    columns: Vec<TableColumn>,
    row_count: usize,
    cell: Option<RenderSlotWith<(usize, usize)>>,
    selection: Option<usize>,
    on_select: Option<CallbackWith<usize>>,
    sort: Option<TableSort>,
    on_sort: Option<CallbackWith<TableSort>>,
    disclosure: Option<Vec<TableDisclosure>>,
    on_expanded_change: Option<CallbackWith<(usize, bool)>>,
    style: Option<TableStyle>,
    alternating_row_backgrounds: Option<bool>,
    modifier: Option<Modifier>,
    #[prop(into)] accessibility_label: Option<String>,
) {
    let style = style.unwrap_or_default();
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    let alternating = alternating_row_backgrounds.unwrap_or(true);
    let widths = Columns(columns.iter().map(|column| column.width).collect());
    let background = if dark {
        Color::from_rgb(0.0129, 0.0129, 0.0129)
    } else {
        Color::WHITE
    };
    let mut modifier = modifier
        .unwrap_or_default()
        .push_draw(Fill {
            color: background,
            radius: Dp::ZERO,
            inset: Dp::ZERO,
        })
        .semantics(SemanticsArgs {
            role: Some(Role::Table),
            label: accessibility_label,
            collection_info: Some((row_count, columns.len(), false)),
            ..Default::default()
        });
    if style == TableStyle::Bordered {
        modifier = modifier.border(
            Dp(1.0),
            if dark { Color::WHITE } else { Color::BLACK }.with_alpha(0.15),
        );
    }
    let titles: Vec<String> = columns.iter().map(|column| column.title.clone()).collect();
    let inset = style != TableStyle::Bordered;
    tessera_ui::provide_context(TableContext { dark }, move || {
        let titles = titles.clone();
        let widths = widths.clone();
        column().modifier(modifier.clone()).children(move || {
            header_row(titles.clone(), widths.clone(), dark)
                .sort_optional(sort)
                .on_sort_optional(on_sort);
            let widths = widths.clone();
            let disclosure = disclosure.clone();
            scroll_view().child(move || {
                rows(row_count, widths.clone(), alternating, inset, dark)
                    .selection_optional(selection)
                    .on_select_optional(on_select)
                    .disclosure_optional(disclosure.clone())
                    .on_expanded_change_optional(on_expanded_change)
                    .cell_optional(cell);
            });
        });
    });
}

#[tessera]
fn header_row(
    titles: Vec<String>,
    widths: Columns,
    dark: bool,
    sort: Option<TableSort>,
    on_sort: Option<CallbackWith<TableSort>>,
) {
    let tint = if dark { Color::WHITE } else { Color::BLACK };
    layout(widths)
        .modifier(
            Modifier::new()
                .height(Dp(HEADER_HEIGHT))
                .semantics(SemanticsArgs {
                    role: Some(Role::Row),
                    ..Default::default()
                }),
        )
        .child(move || {
            for (index, title) in titles.iter().cloned().enumerate() {
                let sorted = sort.filter(|sort| sort.column == index);
                header_cell(title, index, tint)
                    .sorted_optional(sorted)
                    .on_sort_optional(on_sort);
            }
        });
}

#[tessera]
fn header_cell(
    title: String,
    index: usize,
    tint: Color,
    sorted: Option<TableSort>,
    on_sort: Option<CallbackWith<TableSort>>,
) {
    let mut modifier = Modifier::new().semantics(SemanticsArgs {
        role: Some(Role::ColumnHeader),
        label: Some(title.clone()),
        ..Default::default()
    });
    if let Some(on_sort) = on_sort {
        modifier = modifier.clickable(ClickableArgs {
            on_click: Callback::new(move || {
                on_sort.call(TableSort {
                    column: index,
                    ascending: sorted.is_none_or(|sort| !sort.ascending),
                })
            }),
            role: Some(Role::ColumnHeader),
            label: Some(title.clone()),
            ..Default::default()
        });
    }
    crate::row::row()
        .cross_axis_alignment(crate::alignment::CrossAxisAlignment::Center)
        .modifier(
            modifier
                .fill_max_size()
                .padding_symmetric(Dp(CELL_PADDING + 4.0), Dp::ZERO),
        )
        .children(move || {
            text(title.clone())
                .style(TextStyle {
                    font_weight: 600,
                    ..Default::default()
                })
                .color(tint.with_alpha(0.5))
                .single_line(true)
                .modifier(Modifier::new().clear_and_set_semantics(SemanticsArgs {
                    hidden: true,
                    ..Default::default()
                }));
            if let Some(sort) = sorted {
                crate::spacer::spacer().modifier(Modifier::new().weight(1.0));
                layout(Stack).modifier(Modifier::new().size(Dp(8.0), Dp(5.0)).push_draw(
                    SortIndicator {
                        ascending: sort.ascending,
                        color: tint.with_alpha(0.5),
                    },
                ));
            }
        });
}

#[tessera]
fn rows(
    row_count: usize,
    widths: Columns,
    alternating: bool,
    inset: bool,
    dark: bool,
    selection: Option<usize>,
    on_select: Option<CallbackWith<usize>>,
    disclosure: Option<Vec<TableDisclosure>>,
    on_expanded_change: Option<CallbackWith<(usize, bool)>>,
    cell: Option<RenderSlotWith<(usize, usize)>>,
) {
    let columns = widths.0.len();
    column()
        .modifier(
            Modifier::new()
                .focus_group(None)
                .focus_traversal_policy(FocusTraversalPolicy::linear().single_tab_stop(true)),
        )
        .children(move || {
            for row in 0..row_count {
                table_row(
                    row,
                    columns,
                    widths.clone(),
                    selection == Some(row),
                    alternating,
                    inset,
                    dark,
                )
                .on_select_optional(on_select)
                .disclosure_optional(disclosure.as_ref().and_then(|rows| rows.get(row).copied()))
                .on_expanded_change_optional(on_expanded_change)
                .cell_optional(cell);
            }
        });
}

struct RowSelection(bool);

impl SemanticsModifierNode for RowSelection {
    fn apply(
        &self,
        node: &mut tessera_ui::AccessibilityNode,
        _: &mut Option<tessera_ui::AccessibilityActionHandler>,
    ) {
        node.selected = Some(self.0);
    }
}

#[tessera]
fn table_row(
    row: usize,
    columns: usize,
    widths: Columns,
    selected: bool,
    alternating: bool,
    inset: bool,
    dark: bool,
    on_select: Option<CallbackWith<usize>>,
    disclosure: Option<TableDisclosure>,
    on_expanded_change: Option<CallbackWith<(usize, bool)>>,
    cell: Option<RenderSlotWith<(usize, usize)>>,
) {
    let tint = if dark { Color::WHITE } else { Color::BLACK };
    let fill = if selected {
        Color::from_rgb(0.0, 0.19461783, 1.0)
    } else if alternating && row % 2 == 1 {
        tint.with_alpha(0.04)
    } else {
        Color::TRANSPARENT
    };
    let requester = remember(FocusRequester::new).get();
    let mut modifier = Modifier::new()
        .height(Dp(ROW_HEIGHT))
        .push_draw(Fill {
            color: fill,
            radius: if inset { Dp(5.0) } else { Dp::ZERO },
            inset: if inset { Dp(5.0) } else { Dp::ZERO },
        })
        .semantics(SemanticsArgs {
            role: Some(Role::Row),
            collection_item_info: Some((row, 1, 0, columns, false)),
            ..Default::default()
        });
    if let Some(select) = on_select {
        modifier = modifier
            .clickable(ClickableArgs {
                on_click: Callback::new(move || select.call(row)),
                role: Some(Role::Row),
                focus_requester: Some(requester),
                ..Default::default()
            })
            .preferred_tab_stop(selected)
            .on_focus_changed(move |state: FocusState| {
                if state.is_focused()
                    && matches!(
                        focus::focus_change_direction(),
                        Some(FocusDirection::Up | FocusDirection::Down)
                    )
                {
                    select.call(row);
                }
            })
            .push_semantics(RowSelection(selected));
    }
    let appearance = if selected || dark {
        Appearance::Dark
    } else {
        Appearance::Light
    };
    layout(widths).modifier(modifier).child(move || {
        hig_theme().theme(HigTheme { appearance }).child(move || {
            for column in 0..columns {
                let outline = disclosure.filter(|_| column == 0);
                table_cell(row, column)
                    .disclosure_optional(outline)
                    .on_expanded_change_optional(on_expanded_change)
                    .cell_optional(cell);
            }
        });
    });
}

#[tessera]
fn table_cell(
    row: usize,
    column: usize,
    disclosure: Option<TableDisclosure>,
    on_expanded_change: Option<CallbackWith<(usize, bool)>>,
    cell: Option<RenderSlotWith<(usize, usize)>>,
) {
    crate::row::row()
        .cross_axis_alignment(crate::alignment::CrossAxisAlignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_size()
                .clip_to_bounds()
                .padding_symmetric(Dp(CELL_PADDING + 4.0), Dp::ZERO)
                .semantics(SemanticsArgs {
                    role: Some(Role::Cell),
                    collection_item_info: Some((row, 1, column, 1, false)),
                    ..Default::default()
                }),
        )
        .children(move || {
            if let Some(outline) = disclosure {
                crate::spacer::spacer()
                    .modifier(Modifier::new().width(Dp(OUTLINE_INDENT * outline.depth as f64)));
                match outline.expanded {
                    Some(expanded) => {
                        let triangle = disclosure_button(expanded);
                        if let Some(on_change) = on_expanded_change {
                            triangle.on_expanded_change(move |next| on_change.call((row, next)));
                        }
                    }
                    None => {
                        crate::spacer::spacer().modifier(Modifier::new().width(Dp(13.0)));
                    }
                }
                crate::spacer::spacer().modifier(Modifier::new().width(Dp(4.0)));
            }
            if let Some(cell) = cell {
                cell.render((row, column));
            }
        });
}
