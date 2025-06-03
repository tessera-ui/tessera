/// Defines how a dimension (width or height) should be calculated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DimensionValue {
    /// The dimension is a fixed value.
    Fixed(u32),
    /// The dimension should wrap its content.
    Wrap,
    /// The dimension should fill the available space, optionally up to a maximum.
    Fill { max: Option<u32> },
}

impl Default for DimensionValue {
    fn default() -> Self {
        DimensionValue::Wrap // Default to wrapping content
    }
}

/// Represents layout constraints for a component node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Constraint {
    pub width: DimensionValue,
    pub height: DimensionValue,
}

impl Constraint {
    /// A constraint that specifies no preference (Wrap for both width and height).
    pub const NONE: Self = Self {
        width: DimensionValue::Wrap,
        height: DimensionValue::Wrap,
    };

    /// Creates a new constraint.
    pub fn new(width: DimensionValue, height: DimensionValue) -> Self {
        Self { width, height }
    }

    /// Merges this constraint with a parent constraint.
    ///
    /// Rules:
    /// - If self is Fixed, it overrides parent (Fixed wins).
    /// - If self is Wrap, parent's constraint is used (parent might provide a bound).
    /// - If self is Fill:
    ///   - If parent is Fixed(p_val): result is Fixed(min(p_val, self.max.unwrap_or(p_val))).
    ///   - If parent is Wrap: result is self (Fill, as parent offers no concrete size to fill).
    ///     This case means the Fill component will behave like Wrap if its children don't expand.
    ///   - If parent is Fill {max: p_max}: result is Fill {max: combined_max}.
    ///     combined_max is min(self.max, p_max) if both exist, or the existing one, or None.
    pub fn merge(&self, parent_constraint: &Constraint) -> Self {
        let new_width = Self::merge_dimension(self.width, parent_constraint.width);
        let new_height = Self::merge_dimension(self.height, parent_constraint.height);
        Constraint::new(new_width, new_height)
    }

    fn merge_dimension(child_dim: DimensionValue, parent_dim: DimensionValue) -> DimensionValue {
        match child_dim {
            DimensionValue::Fixed(cv) => DimensionValue::Fixed(cv), // Child's Fixed overrides
            DimensionValue::Wrap => match parent_dim {
                DimensionValue::Fixed(pv) => DimensionValue::Fixed(pv), // Parent provides a fixed bound for Wrap
                DimensionValue::Wrap => DimensionValue::Wrap,           // Wrap remains Wrap
                DimensionValue::Fill { max: p_max } => DimensionValue::Fill { max: p_max }, // Wrap behaves as Fill within parent's Fill bounds
            },
            DimensionValue::Fill { max: c_max } => match parent_dim {
                DimensionValue::Fixed(pv) => {
                    DimensionValue::Fixed(c_max.map_or(pv, |cm| pv.min(cm)))
                }
                DimensionValue::Wrap => DimensionValue::Fill { max: c_max }, // Fill remains Fill, parent Wrap offers no concrete size
                DimensionValue::Fill { max: p_max } => {
                    let new_max = match (c_max, p_max) {
                        (Some(cm), Some(pm)) => Some(cm.min(pm)),
                        (Some(cm), None) => Some(cm),
                        (None, Some(pm)) => Some(pm),
                        (None, None) => None,
                    };
                    DimensionValue::Fill { max: new_max }
                }
            },
        }
    }
}
