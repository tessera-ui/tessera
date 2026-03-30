//! Alignment types shared by foundational layout APIs.
//!
//! ## Usage
//!
//! Describe how containers position children along main, cross, or layered
//! axes.

/// Specifies how children are placed along the main axis in layout
/// containers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MainAxisAlignment {
    /// Place children at the start (left or top).
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

impl Default for MainAxisAlignment {
    /// Returns [`MainAxisAlignment::Start`] as the default value.
    ///
    /// # Example
    ///
    /// ```
    /// use tessera_foundation::alignment::MainAxisAlignment;
    /// assert_eq!(MainAxisAlignment::default(), MainAxisAlignment::Start);
    /// ```
    fn default() -> Self {
        Self::Start
    }
}

/// Specifies how children are aligned along the cross axis in layout
/// containers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CrossAxisAlignment {
    /// Align children to the start (left or top).
    Start,
    /// Center children along the cross axis.
    Center,
    /// Align children to the end (right or bottom).
    End,
    /// Stretch children to fill the entire cross axis.
    Stretch,
}

impl Default for CrossAxisAlignment {
    /// Returns [`CrossAxisAlignment::Start`] as the default value.
    fn default() -> Self {
        Self::Start
    }
}

/// Specifies the alignment of a child within its parent container.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Alignment {
    /// Top-left corner.
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

impl Default for Alignment {
    /// Returns [`Alignment::TopStart`] as the default value.
    ///
    /// # Example
    ///
    /// ```
    /// use tessera_foundation::alignment::Alignment;
    /// assert_eq!(Alignment::default(), Alignment::TopStart);
    /// ```
    fn default() -> Self {
        Self::TopStart
    }
}
