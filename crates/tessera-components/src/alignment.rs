//! Defines alignment options for layout components.

/// Specifies how children are placed along the main axis (the direction of
/// layout) in layout containers such as [`crate::row::row`] or
/// [`crate::column::column`].
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum MainAxisAlignment {
    /// Place children at the start (left or top).
    #[default]
    Start,
    /// Center children along the main axis.
    Center,
    /// Place children at the end (right or bottom).
    End,
    /// Evenly distribute children, including space at the start and end.
    SpaceEvenly,
    /// Evenly distribute children, with no space at the start and end.
    SpaceBetween,
    /// Evenly distribute children, with half-space at the start and end.
    SpaceAround,
}

/// Specifies how children are aligned along the cross axis (perpendicular to
/// the layout direction) in layout containers such as [`crate::row::row`] or
/// [`crate::column::column`].
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum CrossAxisAlignment {
    /// Align children to the start (left or top).
    #[default]
    Start,
    /// Center children along the cross axis.
    Center,
    /// Align children to the end (right or bottom).
    End,
    /// Stretch children to fill the entire cross axis.
    Stretch,
}

/// Specifies the alignment of a child within its parent container, both
/// vertically and horizontally. Useful for positioning a single child inside a
/// container, such as in a [`crate::boxed::boxed`] component.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Alignment {
    /// Top-left corner.
    #[default]
    TopStart,
    /// Top edge, centered horizontally.
    TopCenter,
    /// Top-right corner.
    TopEnd,
    /// Center vertically, left edge.
    CenterStart,
    /// Center both vertically and horizontally.
    Center,
    /// Center vertically, right edge.
    CenterEnd,
    /// Bottom-left corner.
    BottomStart,
    /// Bottom edge, centered horizontally.
    BottomCenter,
    /// Bottom-right corner.
    BottomEnd,
}
