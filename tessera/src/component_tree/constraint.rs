/// Describes constraints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Constraint {
    /// max width(pixels)
    pub max_width: Option<u32>,
    /// min width(pixels)
    pub min_width: Option<u32>,
    /// max height(pixels)
    pub max_height: Option<u32>,
    /// min height(pixels)
    pub min_height: Option<u32>,
}

impl Constraint {
    /// Create a new constraint
    /// with all values set to None
    /// which means no constraint
    pub const NONE: Self = Self {
        max_width: None,
        min_width: None,
        max_height: None,
        min_height: None,
    };

    /// Merge parent constraint and self constraint
    /// Parent constraint should always override self constraint
    /// if it's stricter
    pub fn merge(&self, parent: &Self) -> Self {
        // width cannot be bigger than parent's max width
        let max_width = match (self.max_width, parent.max_width) {
            (Some(self_max), Some(parent_max)) => Some(self_max.min(parent_max)),
            (Some(self_max), None) => Some(self_max),
            (None, Some(parent_max)) => Some(parent_max),
            (None, None) => None,
        };
        let min_width = match (self.min_width, max_width) {
            (Some(self_min), Some(max_width)) => Some(self_min.min(max_width)),
            (Some(self_min), None) => Some(self_min),
            (None, Some(_)) => None,
            (None, None) => None,
        };
        // height cannot be bigger than parent's max height
        let max_height = match (self.max_height, parent.max_height) {
            (Some(self_max), Some(parent_max)) => Some(self_max.min(parent_max)),
            (Some(self_max), None) => Some(self_max),
            (None, Some(parent_max)) => Some(parent_max),
            (None, None) => None,
        };
        let min_height = match (self.min_height, max_height) {
            (Some(self_min), Some(max_height)) => Some(self_min.min(max_height)),
            (Some(self_min), None) => Some(self_min),
            (None, Some(_)) => None,
            (None, None) => None,
        };

        Self {
            max_width,
            min_width,
            max_height,
            min_height,
        }
    }
}
