//! Text tables with content-sized columns and merged cells.
//!
//! ## Usage
//!
//! Compare values with accessible headers and merged cells.

use std::sync::Arc;

use parking_lot::Mutex;
use tessera_ui::{
    AxisConstraint, Color, ComputedData, Constraint, Dp, LayoutModifierNode, LayoutResult,
    MeasurementError, Modifier, Px, PxPosition,
    accesskit::Role,
    layout::{MeasureScope, PlacementResult, PlacementScope, layout},
    remember, remember_with_key, tessera, use_context,
};
use unicode_linebreak::linebreaks;

#[cfg(test)]
use tessera_ui::modifier::FocusModifierExt as _;

use crate::{
    alignment::Alignment,
    boxed::boxed,
    modifier::{ModifierExt as _, Padding, SemanticsArgs},
    pipelines::text::{command::TextConstraint, pipeline::TextData},
    spacer::spacer,
    text::text,
    theme::{MaterialTheme, TextStyle},
};

const MAX_GRID_POSITIONS: usize = 1_048_576;

/// The meaning of a text table cell.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextTableHeading {
    /// An ordinary data cell.
    #[default]
    None,
    /// A heading for the column or columns covered by the cell.
    Column,
    /// A heading for the row or rows covered by the cell.
    Row,
}

/// Text and grid coordinates for a table cell.
///
/// Indices start at zero. Spans must be positive and cells must not overlap.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TextTableCell {
    /// The displayed text, including any explicit line breaks.
    pub text: String,
    /// The first covered row.
    pub row: usize,
    /// The first covered column.
    pub column: usize,
    /// The number of covered rows.
    pub row_span: usize,
    /// The number of covered columns.
    pub column_span: usize,
    /// Whether the cell names rows or columns.
    pub heading: TextTableHeading,
}

impl Default for TextTableCell {
    fn default() -> Self {
        Self {
            text: String::new(),
            row: 0,
            column: 0,
            row_span: 1,
            column_span: 1,
            heading: TextTableHeading::None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct ShapingStyleKey {
    font_size: u64,
    line_height: Option<u64>,
    font_weight: u16,
    letter_spacing: u64,
    font_family: Option<&'static str>,
}

impl From<TextStyle> for ShapingStyleKey {
    fn from(style: TextStyle) -> Self {
        Self {
            font_size: style.font_size.0.to_bits(),
            line_height: style.line_height.map(|height| height.0.to_bits()),
            font_weight: style.font_weight,
            letter_spacing: style.letter_spacing.0.to_bits(),
            font_family: style.font_family,
        }
    }
}

/// # text_table
///
/// Compare text with content-sized columns and accessible merged cells.
///
/// ## Usage
///
/// Compare specifications or settings with row and column headers.
///
/// ## Parameters
///
/// - `cells` — text, zero-based grid coordinates, spans and heading roles
/// - `columns` — column count; omitted values infer it from the cells
/// - `modifier` — table sizing and node behavior
/// - `style`, `header_style` — data and heading typography
/// - `cell_padding` — content insets, defaulting to 24dp horizontally and 16dp
///   vertically
/// - `color`, `header_color` — body and heading text colors; headings use the
///   body color when no heading color is supplied
/// - `header_background`, `divider_color` — heading fill and row-rule colors
///
/// Columns adapt to measured text. Narrow columns use the text component's
/// wrapping behavior. Physical-pixel rounding preserves the total allocated
/// width without gaps. Cells are read in row and column order. Grids cannot
/// exceed 1,048,576 positions.
///
/// ## Examples
///
/// ```
/// use tessera_components::text_table::{TextTableCell, TextTableHeading, text_table};
/// #[tessera_ui::tessera]
/// fn comparison() {
///     let heading = TextTableCell {
///         text: "Size".into(),
///         heading: TextTableHeading::Column,
///         ..Default::default()
///     };
///     let value = TextTableCell {
///         text: "Medium".into(),
///         row: 1,
///         ..Default::default()
///     };
///     assert_eq!(heading.row, 0);
///     assert_eq!(value.row, 1);
///     text_table(vec![heading, value]).columns(1);
/// }
/// comparison();
/// ```
#[tessera]
pub fn text_table(
    cells: Vec<TextTableCell>,
    columns: Option<usize>,
    modifier: Option<Modifier>,
    style: Option<TextStyle>,
    header_style: Option<TextStyle>,
    cell_padding: Option<Padding>,
    color: Option<Color>,
    header_color: Option<Color>,
    header_background: Option<Color>,
    divider_color: Option<Color>,
) {
    let theme = use_context::<MaterialTheme>()
        .map(|theme| theme.get())
        .unwrap_or_default();
    let style = style.unwrap_or(theme.typography.body_large);
    let header_style = header_style.unwrap_or(TextStyle {
        font_weight: 500,
        ..style
    });
    let padding = cell_padding.unwrap_or_else(|| Padding::symmetric(Dp(24.0), Dp(16.0)));
    let color = color.unwrap_or(theme.color_scheme.on_surface);
    let header_color = header_color.unwrap_or(color);
    let header_background = header_background.unwrap_or(theme.color_scheme.surface_container_low);
    let divider_color = divider_color.unwrap_or(theme.color_scheme.outline_variant);
    let mut cells = cells;
    cells.sort_by_key(|cell| (cell.row, cell.column));
    let columns = columns.unwrap_or_else(|| {
        cells
            .iter()
            .map(|cell| {
                cell.column
                    .checked_add(cell.column_span)
                    .expect("Table column overflow")
            })
            .max()
            .unwrap_or(0)
    });
    let rows = validate(&cells, columns);
    let metrics_key = (
        cells.clone(),
        ShapingStyleKey::from(style),
        ShapingStyleKey::from(header_style),
        [
            padding.left.0.to_bits(),
            padding.top.0.to_bits(),
            padding.right.0.to_bits(),
            padding.bottom.0.to_bits(),
        ],
        Dp(1.0).to_pixels_f32().to_bits(),
    );
    let intrinsic = remember_with_key(metrics_key, move || {
        measure_intrinsics(&cells, style, header_style, padding)
    })
    .get();
    let cache = remember(|| Arc::new(Mutex::new(None::<TrackLayout>))).get();
    let cells = intrinsic.cells.clone();
    let modifier = modifier.unwrap_or_default().semantics(SemanticsArgs {
        role: Some(Role::Table),
        merge_descendants: true,
        collection_info: Some((rows, columns, false)),
        ..Default::default()
    });
    layout()
        .modifier(modifier)
        .policy(TableLayout {
            intrinsic,
            rows,
            columns,
            cache,
        })
        .child(move || {
            for row in 0..rows {
                table_row(
                    cells
                        .iter()
                        .filter(|cell| cell.row == row)
                        .cloned()
                        .collect(),
                    row,
                    columns,
                    style,
                    header_style,
                    padding,
                    color,
                    header_color,
                    header_background,
                    divider_color,
                );
            }
        });
}

#[tessera]
fn table_row(
    cells: Vec<TextTableCell>,
    row: usize,
    columns: usize,
    style: TextStyle,
    header_style: TextStyle,
    padding: Padding,
    color: Color,
    header_color: Color,
    header_background: Color,
    divider_color: Color,
) {
    Modifier::new()
        .semantics(SemanticsArgs {
            role: Some(Role::Row),
            merge_descendants: true,
            collection_item_info: Some((row, 1, 0, columns, false)),
            ..Default::default()
        })
        .attach();
    for cell in cells {
        let heading = cell.heading != TextTableHeading::None;
        let modifier = Modifier::new().semantics(SemanticsArgs {
            role: Some(match cell.heading {
                TextTableHeading::None => Role::Cell,
                TextTableHeading::Column => Role::ColumnHeader,
                TextTableHeading::Row => Role::RowHeader,
            }),
            label: Some(cell.text.clone()),
            merge_descendants: false,
            collection_item_info: Some((
                cell.row,
                cell.row_span,
                cell.column,
                cell.column_span,
                heading,
            )),
            ..Default::default()
        });
        let modifier = if heading {
            modifier.background(header_background)
        } else {
            modifier
        };
        #[cfg(test)]
        let modifier = {
            let key = (cell.row, cell.column);
            modifier.on_layout_coordinates(move |coordinates| {
                tests::CELLS.with_borrow_mut(|cells| {
                    cells.insert(key, coordinates);
                });
            })
        };
        boxed()
            .alignment(Alignment::CenterStart)
            .modifier(modifier)
            .children(move || {
                text()
                    .content(cell.text.clone())
                    .style(if heading { header_style } else { style })
                    .color(if heading { header_color } else { color })
                    .modifier(Modifier::new().padding(padding));
                spacer().modifier(
                    Modifier::new()
                        .fill_max_width()
                        .height(Dp(1.0))
                        .background(divider_color)
                        .align(Alignment::BottomStart),
                );
            });
    }
}

fn validate(cells: &[TextTableCell], columns: usize) -> usize {
    if cells.is_empty() {
        return 0;
    }
    assert!(columns > 0, "A nonempty table requires columns");
    let rows = cells
        .iter()
        .map(|cell| {
            assert!(
                cell.row_span > 0 && cell.column_span > 0,
                "Table spans must be positive"
            );
            let end = cell
                .column
                .checked_add(cell.column_span)
                .expect("Table column overflow");
            assert!(end <= columns, "A table cell exceeds the column count");
            cell.row
                .checked_add(cell.row_span)
                .expect("Table row overflow")
        })
        .max()
        .unwrap_or(0);
    let count = rows
        .checked_mul(columns)
        .filter(|count| *count <= MAX_GRID_POSITIONS)
        .expect("Table exceeds the supported grid allocation");
    let mut occupied = vec![false; count];
    for cell in cells {
        for row in cell.row..cell.row + cell.row_span {
            for column in cell.column..cell.column + cell.column_span {
                let slot = &mut occupied[row * columns + column];
                assert!(!*slot, "Table cells overlap");
                *slot = true;
            }
        }
    }
    rows
}

#[derive(Clone, PartialEq)]
struct Intrinsics {
    cells: Vec<TextTableCell>,
    widths: Vec<(i32, i32)>,
}

fn shaped_width(content: &str, style: TextStyle) -> i32 {
    let width = TextData::shape(
        content.to_owned(),
        Color::BLACK,
        style,
        TextConstraint {
            max_width: None,
            max_height: None,
        },
    )
    .measure_info()
    .size[0];
    i32::try_from(width).expect("Table text exceeds the supported physical width")
}

fn measure_intrinsics(
    cells: &[TextTableCell],
    style: TextStyle,
    header_style: TextStyle,
    padding: Padding,
) -> Intrinsics {
    let horizontal: Px = padding.left.into();
    let right: Px = padding.right.into();
    let padding_width = horizontal.0 + right.0;
    let widths = cells
        .iter()
        .map(|cell| {
            let style = if cell.heading == TextTableHeading::None {
                style
            } else {
                header_style
            };
            let maximum = shaped_width(&cell.text, style) + padding_width;
            let mut start = 0;
            let mut minimum = 0;
            for (end, _) in linebreaks(&cell.text) {
                let segment = cell.text[start..end].trim_end_matches(char::is_whitespace);
                minimum = minimum.max(shaped_width(segment, style));
                start = end;
            }
            (
                minimum + padding_width,
                maximum.max(minimum + padding_width),
            )
        })
        .collect();
    Intrinsics {
        cells: cells.to_vec(),
        widths,
    }
}

#[derive(Clone)]
struct TableLayout {
    intrinsic: Intrinsics,
    rows: usize,
    columns: usize,
    cache: Arc<Mutex<Option<TrackLayout>>>,
}
impl PartialEq for TableLayout {
    fn eq(&self, other: &Self) -> bool {
        self.intrinsic == other.intrinsic
            && self.rows == other.rows
            && self.columns == other.columns
    }
}

#[derive(Clone)]
struct TrackLayout {
    size: ComputedData,
    positions: Vec<PxPosition>,
    sizes: Vec<ComputedData>,
}

impl LayoutModifierNode for TableLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let children = input.children();
        assert_eq!(
            children.len(),
            self.intrinsic.cells.len(),
            "Table cells must create one layout child each"
        );
        let parent = *input.parent_constraint().as_ref();
        if children.is_empty() {
            let size = ComputedData {
                width: parent.width.clamp(Px::ZERO),
                height: parent.height.clamp(Px::ZERO),
            };
            *self.cache.lock() = Some(TrackLayout {
                size,
                positions: Vec::new(),
                sizes: Vec::new(),
            });
            return Ok(LayoutResult::new(size));
        }
        let mut minimum = vec![0; self.columns];
        let mut maximum = vec![0; self.columns];
        let mut order = (0..children.len()).collect::<Vec<_>>();
        order.sort_by_key(|&index| self.intrinsic.cells[index].column_span);
        for index in order {
            let cell = &self.intrinsic.cells[index];
            let range = cell.column..cell.column + cell.column_span;
            grow_span(&mut minimum[range.clone()], self.intrinsic.widths[index].0);
            grow_span(&mut maximum[range], self.intrinsic.widths[index].1);
        }
        for (maximum, minimum) in maximum.iter_mut().zip(&minimum) {
            *maximum = (*maximum).max(*minimum);
        }
        let desired = maximum.iter().sum::<i32>();
        let width = parent.width.clamp(Px(desired));
        let columns = fit_columns(width.0, &minimum, &maximum);
        let x = prefix(&columns);
        let mut rows = vec![0; self.rows];
        let mut heights = vec![0; children.len()];
        for (index, child) in children.iter().enumerate() {
            let cell = &self.intrinsic.cells[index];
            let cell_width = x[cell.column + cell.column_span] - x[cell.column];
            let measured = child.probe_intrinsic(
                input,
                &Constraint::new(AxisConstraint::exact(Px(cell_width)), AxisConstraint::NONE),
            )?;
            heights[index] = measured.height.0;
            if cell.row_span == 1 {
                rows[cell.row] = rows[cell.row].max(measured.height.0);
            }
        }
        let mut order = (0..children.len()).collect::<Vec<_>>();
        order.sort_by_key(|&index| self.intrinsic.cells[index].row_span);
        for index in order {
            let cell = &self.intrinsic.cells[index];
            grow_span(
                &mut rows[cell.row..cell.row + cell.row_span],
                heights[index],
            );
        }
        let y = prefix(&rows);
        let size = ComputedData {
            width,
            height: parent.height.clamp(Px(*y.last().unwrap_or(&0))),
        };
        let mut result = LayoutResult::new(size);
        let mut positions = Vec::with_capacity(children.len());
        let mut sizes = Vec::with_capacity(children.len());
        for (index, child) in children.iter().enumerate() {
            let cell = &self.intrinsic.cells[index];
            let constraint = Constraint::new(
                AxisConstraint::exact(Px(x[cell.column + cell.column_span] - x[cell.column])),
                AxisConstraint::exact(Px(y[cell.row + cell.row_span] - y[cell.row])),
            );
            let measured = child.measure(input, &constraint)?;
            let position = PxPosition::new(Px(x[cell.column]), Px(y[cell.row]));
            result.place_child(*child, position);
            positions.push(position);
            sizes.push(measured.size());
        }
        *self.cache.lock() = Some(TrackLayout {
            size,
            positions,
            sizes,
        });
        Ok(result)
    }

    fn place_children(
        &self,
        scope: &PlacementScope<'_>,
        _: &PlacementResult,
    ) -> Option<PlacementResult> {
        let tracks = self.cache.lock();
        let tracks = tracks.as_ref()?;
        let children = scope.children();
        if tracks.size != scope.size()
            || children.len() != tracks.positions.len()
            || children
                .iter()
                .zip(&tracks.sizes)
                .any(|(child, size)| child.size() != *size)
        {
            return None;
        }
        let mut result = LayoutResult::new(scope.size());
        for (child, position) in children.iter().zip(&tracks.positions) {
            result.place_child(*child, *position);
        }
        Some(result.into_placement_result())
    }
}

fn prefix(values: &[i32]) -> Vec<i32> {
    std::iter::once(0)
        .chain(values.iter().scan(0, |sum, value| {
            *sum += value;
            Some(*sum)
        }))
        .collect()
}

fn distribute(total: i32, weights: &[i32]) -> Vec<i32> {
    if weights.is_empty() {
        return Vec::new();
    }
    let denominator = weights
        .iter()
        .map(|value| (*value).max(0) as i64)
        .sum::<i64>();
    let denominator = if denominator == 0 {
        weights.len() as i64
    } else {
        denominator
    };
    let equal = weights.iter().all(|weight| *weight == 0);
    let mut remainders = Vec::with_capacity(weights.len());
    let mut values = weights
        .iter()
        .enumerate()
        .map(|(index, weight)| {
            let numerator = i64::from(total.max(0))
                * if equal {
                    1
                } else {
                    i64::from((*weight).max(0))
                };
            remainders.push((numerator % denominator, index));
            (numerator / denominator) as i32
        })
        .collect::<Vec<_>>();
    remainders.sort_by_key(|&(remainder, index)| (std::cmp::Reverse(remainder), index));
    let residual = total.max(0) - values.iter().sum::<i32>();
    for &(_, index) in remainders.iter().take(residual as usize) {
        values[index] += 1;
    }
    values
}

fn grow_span(values: &mut [i32], required: i32) {
    let deficit = (required - values.iter().sum::<i32>()).max(0);
    let extra = distribute(deficit, &vec![1; values.len()]);
    for (value, extra) in values.iter_mut().zip(extra) {
        *value += extra;
    }
}

fn fit_columns(width: i32, minimum: &[i32], maximum: &[i32]) -> Vec<i32> {
    let min_total = minimum.iter().sum::<i32>();
    let max_total = maximum.iter().sum::<i32>();
    if width < min_total {
        return distribute(width, minimum);
    }
    let weights = if width < max_total {
        maximum
            .iter()
            .zip(minimum)
            .map(|(max, min)| max - min)
            .collect::<Vec<_>>()
    } else {
        maximum.to_vec()
    };
    let base = if width < max_total { minimum } else { maximum };
    let extra = distribute(width - base.iter().sum::<i32>(), &weights);
    base.iter()
        .zip(extra)
        .map(|(base, extra)| base + extra)
        .collect()
}

#[cfg(test)]
#[path = "text_table_tests.rs"]
mod tests;
