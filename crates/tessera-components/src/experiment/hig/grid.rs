//! Grids that arrange views in columns.
//!
//! ## Usage
//!
//! Show photos, app icons, or settings tiles in rows and columns.

use tessera_ui::{
    AxisConstraint, ComputedData, Constraint, Dp, LayoutPolicy, LayoutResult, MeasurementError,
    Modifier, Px, PxPosition, RenderSlot,
    layout::{MeasureScope, layout},
    tessera,
};

#[cfg(test)]
#[path = "grid_tests.rs"]
mod tests;

/// How one column of a [`grid`] is sized.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GridItem {
    /// A column of exactly this width.
    Fixed(Dp),
    /// A column sharing the width left after fixed columns, within `min` and `max`.
    Flexible {
        /// Smallest width.
        min: Dp,
        /// Largest width.
        max: Dp,
    },
    /// As many columns as fit, each at least `min` wide, sharing the width.
    Adaptive {
        /// Smallest width of each column.
        min: Dp,
    },
}

impl GridItem {
    /// A flexible column without limits.
    pub fn flexible() -> Self {
        Self::Flexible {
            min: Dp::ZERO,
            max: Dp(f64::INFINITY),
        }
    }
}

#[derive(Clone, PartialEq)]
struct GridLayout {
    columns: Vec<GridItem>,
    spacing: Dp,
}

impl GridLayout {
    fn widths(&self, available: Px) -> Vec<Px> {
        let spacing = Px::from(self.spacing);
        if let [GridItem::Adaptive { min }] = self.columns.as_slice() {
            let min = Px::from(*min).max(Px(1));
            let count = ((available + spacing).0 / (min + spacing).0).max(1);
            let width = Px((available.0 - spacing.0 * (count - 1)) / count);
            return vec![width; count as usize];
        }
        let gaps = spacing * (self.columns.len().saturating_sub(1) as i32);
        let fixed: Px = self
            .columns
            .iter()
            .filter_map(|column| match column {
                GridItem::Fixed(width) => Some(Px::from(*width)),
                _ => None,
            })
            .fold(Px::ZERO, |sum, width| sum + width);
        let flexible = self
            .columns
            .iter()
            .filter(|column| !matches!(column, GridItem::Fixed(_)))
            .count()
            .max(1) as i32;
        let share = Px(((available - fixed - gaps).0 / flexible).max(0));
        self.columns
            .iter()
            .map(|column| match column {
                GridItem::Fixed(width) => Px::from(*width),
                GridItem::Flexible { min, max } => {
                    let max = if max.0.is_finite() {
                        Px::from(*max)
                    } else {
                        share
                    };
                    share.min(max).max(Px::from(*min))
                }
                GridItem::Adaptive { min } => share.max(Px::from(*min)),
            })
            .collect()
    }
}

impl LayoutPolicy for GridLayout {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = *scope.parent_constraint().as_ref();
        let spacing = Px::from(self.spacing);
        let available = parent
            .width
            .resolve_max()
            .unwrap_or_else(|| Px::from(Dp(100.0 * self.columns.len().max(1) as f64)));
        let widths = self.widths(available);
        let count = widths.len().max(1);
        let children = scope.children();
        let mut sizes = Vec::with_capacity(children.len());
        for (index, child) in children.iter().enumerate() {
            let width = widths[index % count];
            let size = child
                .measure(
                    scope,
                    &Constraint::new(
                        AxisConstraint::new(Px::ZERO, Some(width)),
                        AxisConstraint::new(Px::ZERO, None),
                    ),
                )?
                .size();
            sizes.push(size);
        }
        let row_heights: Vec<Px> = sizes
            .chunks(count)
            .map(|row| row.iter().map(|size| size.height).fold(Px::ZERO, Px::max))
            .collect();
        let width = widths.iter().fold(Px::ZERO, |sum, width| sum + *width)
            + spacing * (count.saturating_sub(1) as i32);
        let height = row_heights
            .iter()
            .fold(Px::ZERO, |sum, height| sum + *height)
            + spacing * (row_heights.len().saturating_sub(1) as i32);
        let mut result = LayoutResult::new(ComputedData {
            width: parent.width.clamp(width),
            height: parent.height.clamp(height),
        });
        let mut y = Px::ZERO;
        for (row, height) in row_heights.iter().enumerate() {
            let mut x = Px::ZERO;
            for column in 0..count {
                let index = row * count + column;
                let Some(child) = children.get(index) else {
                    break;
                };
                // Cells are centred in their column and row, like SwiftUI's default alignment.
                let size = sizes[index];
                let dx = Px((widths[column] - size.width).0.max(0) / 2);
                let dy = Px((*height - size.height).0.max(0) / 2);
                result.place_child(*child, PxPosition::new(x + dx, y + dy));
                x += widths[column] + spacing;
            }
            y += *height + spacing;
        }
        Ok(result)
    }
}

/// # grid
///
/// Arranges views in rows, filling the columns described by its grid items.
///
/// ## Usage
///
/// Show photos, icons, or tiles in a grid that adapts to the available width.
///
/// ## Parameters
///
/// - `columns` — how each column is sized; one adaptive item fits as many columns as possible.
/// - `spacing` — space between rows and columns; defaults to 8pt.
/// - `modifier` — size and placement.
/// - `content` — the views, placed row by row.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{
///     grid::{GridItem, grid},
///     text::text,
/// };
/// use tessera_ui::Dp;
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// grid(vec![GridItem::Adaptive { min: Dp(80.0) }]).content(|| {
///     for name in ["Photos", "Music", "Books", "News"] {
///         text(name);
///     }
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn grid(
    columns: Vec<GridItem>,
    spacing: Option<Dp>,
    modifier: Option<Modifier>,
    content: Option<RenderSlot>,
) {
    layout(GridLayout {
        columns,
        spacing: spacing.unwrap_or(Dp(8.0)),
    })
    .modifier(modifier.unwrap_or_default())
    .child(move || {
        if let Some(content) = content {
            content.render();
        }
    });
}
