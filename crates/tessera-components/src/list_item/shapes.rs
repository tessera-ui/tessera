//! Expressive list item corners for positions and interaction states.
//!
//! ## Usage
//!
//! Group related rows and highlight their active states.

use crate::{shape_def::Shape, theme::MaterialTheme};

/// Position of a row within a segmented list.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListItemPosition {
    /// An independent row without segmented outer corners.
    #[default]
    Standalone,
    /// First row, with larger upper corners.
    First,
    /// Interior row, with smaller corners on all sides.
    Middle,
    /// Last row, with larger lower corners.
    Last,
    /// The only row, with larger corners on all sides.
    Single,
}

/// Container shapes used by a list item in its interaction states.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ListItemShapes {
    /// Resting shape.
    pub shape: Shape,
    /// Shape when selected.
    pub selected_shape: Shape,
    /// Shape while pressed.
    pub pressed_shape: Shape,
    /// Shape while focused.
    pub focused_shape: Shape,
    /// Shape while hovered.
    pub hovered_shape: Shape,
    /// Shape while dragged.
    pub dragged_shape: Shape,
}

impl Default for ListItemShapes {
    fn default() -> Self {
        Self::from_theme(&MaterialTheme::default())
    }
}

impl ListItemShapes {
    /// Resolve list shapes from the current theme's corner sizes.
    pub fn from_theme(theme: &MaterialTheme) -> Self {
        Self {
            shape: theme.shapes.extra_small,
            selected_shape: theme.shapes.large,
            pressed_shape: theme.shapes.large,
            focused_shape: theme.shapes.large,
            hovered_shape: theme.shapes.medium,
            dragged_shape: theme.shapes.large,
        }
    }

    pub(super) fn segmented(mut self, position: ListItemPosition, outer: Shape) -> Self {
        if let (
            Shape::RoundedRectangle {
                top_left,
                top_right,
                bottom_right,
                bottom_left,
            },
            Shape::RoundedRectangle {
                top_left: outer_tl,
                top_right: outer_tr,
                bottom_right: outer_br,
                bottom_left: outer_bl,
            },
        ) = (self.shape, outer)
        {
            let top = matches!(position, ListItemPosition::First | ListItemPosition::Single);
            let bottom = matches!(position, ListItemPosition::Last | ListItemPosition::Single);
            self.shape = Shape::RoundedRectangle {
                top_left: if top { outer_tl } else { top_left },
                top_right: if top { outer_tr } else { top_right },
                bottom_right: if bottom { outer_br } else { bottom_right },
                bottom_left: if bottom { outer_bl } else { bottom_left },
            };
        }
        self
    }
}
