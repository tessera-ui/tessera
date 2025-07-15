//! Defines alignment options for layout components.

/// Alignment along the main axis (the direction of layout).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MainAxisAlignment {
    /// Align to the start (left or top).
    Start,
    /// Align to the center.
    Center,
    /// Align to the end (right or bottom).
    End,
    /// Distribute space evenly, with space at the start and end.
    SpaceEvenly,
    /// Distribute space evenly, with no space at the start and end.
    SpaceBetween,
    /// Distribute space evenly, with half-space at the start and end.
    SpaceAround,
}

impl Default for MainAxisAlignment {
    fn default() -> Self {
        Self::Start
    }
}

/// Alignment along the cross axis (perpendicular to the layout direction).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CrossAxisAlignment {
    /// Align to the start (left or top).
    Start,
    /// Align to the center.
    Center,
    /// Align to the end (right or bottom).
    End,
    /// Stretch to fill the entire cross axis.
    Stretch,
}

impl Default for CrossAxisAlignment {
    fn default() -> Self {
        Self::Start
    }
}

/// Specifies the alignment of a child within its parent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Alignment {
    TopStart,
    TopCenter,
    TopEnd,
    CenterStart,
    Center,
    CenterEnd,
    BottomStart,
    BottomCenter,
    BottomEnd,
}

impl Default for Alignment {
    fn default() -> Self {
        Self::Center
    }
}
