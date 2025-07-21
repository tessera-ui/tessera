//! Defines alignment options for layout components.

/// Specifies how children are placed along the main axis (the direction of layout)
/// in layout containers such as [`Row`](crate::row::Row) or [`Column`](crate::column::Column).
///
/// # Variants
/// - `Start`: Place children at the start (left or top).
/// - `Center`: Center children along the main axis.
/// - `End`: Place children at the end (right or bottom).
/// - `SpaceEvenly`: Evenly distribute children, including space at the start and end.
/// - `SpaceBetween`: Evenly distribute children, with no space at the start and end.
/// - `SpaceAround`: Evenly distribute children, with half-space at the start and end.
///
/// # Example
/// ```rust,ignore
/// use tessera_ui_basic_components::{Row, MainAxisAlignment};
///
/// let row = Row::new()
///     .main_axis_alignment(MainAxisAlignment::SpaceBetween);
/// ```
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
    /// ```rust,ignore
    /// use tessera_ui_basic_components::MainAxisAlignment;
    /// assert_eq!(MainAxisAlignment::default(), MainAxisAlignment::Start);
    /// ```
    fn default() -> Self {
        Self::Start
    }
}

/// Specifies how children are aligned along the cross axis (perpendicular to the layout direction)
/// in layout containers such as [`Row`](crate::row::Row) or [`Column`](crate::column::Column).
///
/// # Variants
/// - `Start`: Align children to the start (left or top).
/// - `Center`: Center children along the cross axis.
/// - `End`: Align children to the end (right or bottom).
/// - `Stretch`: Stretch children to fill the cross axis.
///
/// # Example
/// ```rust,ignore
/// use tessera_ui_basic_components::{Column, CrossAxisAlignment};
///
/// let column = Column::new()
///     .cross_axis_alignment(CrossAxisAlignment::Stretch);
/// ```
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
    ///
    /// # Example
    /// ```rust,ignore
    /// use tessera_ui_basic_components::CrossAxisAlignment;
    /// assert_eq!(CrossAxisAlignment::default(), CrossAxisAlignment::Start);
    /// ```
    fn default() -> Self {
        Self::Start
    }
}

/// Specifies the alignment of a child within its parent container, both vertically and horizontally.
/// Useful for positioning a single child inside a container, such as in a [`Boxed`](crate::boxed::Boxed) component.
///
/// # Variants
/// - `TopStart`: Top-left corner.
/// - `TopCenter`: Top edge, centered horizontally.
/// - `TopEnd`: Top-right corner.
/// - `CenterStart`: Center vertically, left edge.
/// - `Center`: Center both vertically and horizontally.
/// - `CenterEnd`: Center vertically, right edge.
/// - `BottomStart`: Bottom-left corner.
/// - `BottomCenter`: Bottom edge, centered horizontally.
/// - `BottomEnd`: Bottom-right corner.
///
/// # Example
/// ```rust,ignore
/// use tessera_ui_basic_components::{Boxed, Alignment};
///
/// let boxed = Boxed::new()
///     .alignment(Alignment::BottomEnd);
/// ```
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
    /// Returns [`Alignment::Center`] as the default value.
    ///
    /// # Example
    /// ```rust,ignore
    /// use tessera_ui_basic_components::Alignment;
    /// assert_eq!(Alignment::default(), Alignment::Center);
    /// ```
    fn default() -> Self {
        Self::Center
    }
}
