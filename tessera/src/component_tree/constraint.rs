use crate::Px;

/// Defines how a dimension (width or height) should be calculated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DimensionValue {
    /// The dimension is a fixed value in logical pixels.
    Fixed(Px),
    /// The dimension should wrap its content, optionally bounded by min and/or max logical pixels.
    Wrap { min: Option<Px>, max: Option<Px> },
    /// The dimension should fill the available space, optionally bounded by min and/or max logical pixels.
    Fill { min: Option<Px>, max: Option<Px> },
}

impl Default for DimensionValue {
    fn default() -> Self {
        DimensionValue::Wrap {
            min: None,
            max: None,
        }
    }
}

/// Represents layout constraints for a component node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Constraint {
    pub width: DimensionValue,
    pub height: DimensionValue,
}

impl Constraint {
    /// A constraint that specifies no preference (Wrap { None, None } for both width and height).
    pub const NONE: Self = Self {
        width: DimensionValue::Wrap {
            min: None,
            max: None,
        },
        height: DimensionValue::Wrap {
            min: None,
            max: None,
        },
    };

    /// Creates a new constraint.
    pub fn new(width: DimensionValue, height: DimensionValue) -> Self {
        Self { width, height }
    }

    /// Merges this constraint with a parent constraint.
    ///
    /// Rules:
    /// - If self is Fixed, it overrides parent (Fixed wins).
    /// - If self is Wrap, it keeps its own min and combines max constraints:
    ///   - If parent is Fixed(p_val): result is Wrap with child's min and max capped by p_val.
    ///   - If parent is Wrap: result is Wrap with child's min and combined max.
    ///   - If parent is Fill: result is Wrap with child's min and combined max.
    /// - If self is Fill:
    ///   - If parent is Fixed(p_val): result is Fill with child's min and max capped by p_val.
    ///   - If parent is Wrap: result is Fill (child fills available space within parent's bounds).
    ///   - If parent is Fill: result is Fill with combined constraints.
    pub fn merge(&self, parent_constraint: &Constraint) -> Self {
        let new_width = Self::merge_dimension(self.width, parent_constraint.width);
        let new_height = Self::merge_dimension(self.height, parent_constraint.height);
        Constraint::new(new_width, new_height)
    }

    fn merge_dimension(child_dim: DimensionValue, parent_dim: DimensionValue) -> DimensionValue {
        match child_dim {
            DimensionValue::Fixed(cv) => DimensionValue::Fixed(cv), // Child's Fixed overrides
            DimensionValue::Wrap {
                min: c_min,
                max: c_max,
            } => match parent_dim {
                DimensionValue::Fixed(pv) => DimensionValue::Wrap {
                    // Wrap stays as Wrap, but constrained by parent's fixed size
                    min: c_min, // Keep child's own min
                    max: match c_max {
                        Some(c) => Some(c.min(pv)), // Child's max capped by parent's fixed size
                        None => Some(pv),           // Parent's fixed size becomes the max
                    },
                },
                DimensionValue::Wrap {
                    min: _p_min,
                    max: p_max,
                } => DimensionValue::Wrap {
                    // Combine min/max from parent and child for Wrap
                    min: c_min, // Wrap always keeps its own min, never inherits from parent
                    max: match (c_max, p_max) {
                        (Some(c), Some(p)) => Some(c.min(p)), // Take the more restrictive max
                        (Some(c), None) => Some(c),
                        (None, Some(p)) => Some(p),
                        (None, None) => None,
                    },
                },
                DimensionValue::Fill {
                    min: _p_fill_min,
                    max: p_fill_max,
                } => DimensionValue::Wrap {
                    // Child wants to wrap, so it stays as Wrap
                    min: c_min, // Keep child's own min, don't inherit from parent's Fill
                    max: match (c_max, p_fill_max) {
                        (Some(c), Some(p)) => Some(c.min(p)), // Child's max should cap parent's fill max
                        (Some(c), None) => Some(c),
                        (None, Some(p)) => Some(p),
                        (None, None) => None,
                    },
                },
            },
            DimensionValue::Fill {
                min: c_fill_min,
                max: c_fill_max,
            } => match parent_dim {
                DimensionValue::Fixed(pv) => {
                    // Child wants to fill, parent is fixed. Result is Fill with parent's fixed size as max.
                    DimensionValue::Fill {
                        min: c_fill_min, // Keep child's own min
                        max: match c_fill_max {
                            Some(c) => Some(c.min(pv)), // Child's max capped by parent's fixed size
                            None => Some(pv),           // Parent's fixed size becomes the max
                        },
                    }
                }
                DimensionValue::Wrap {
                    min: p_wrap_min,
                    max: p_wrap_max,
                } => DimensionValue::Fill {
                    // Fill remains Fill, parent Wrap offers no concrete size unless it has max
                    min: c_fill_min.or(p_wrap_min), // Child's fill min, or parent's wrap min
                    max: match (c_fill_max, p_wrap_max) {
                        // Child's fill max, potentially capped by parent's wrap max
                        (Some(cf), Some(pw)) => Some(cf.min(pw)),
                        (Some(cf), None) => Some(cf),
                        (None, Some(pw)) => Some(pw),
                        (None, None) => None,
                    },
                },
                DimensionValue::Fill {
                    min: p_fill_min,
                    max: p_fill_max,
                } => {
                    // Both are Fill. Combine min and max.
                    // New min is the greater of the two mins (or the existing one).
                    // New max is the smaller of the two maxes (or the existing one).
                    let new_min = match (c_fill_min, p_fill_min) {
                        (Some(cm), Some(pm)) => Some(cm.max(pm)),
                        (Some(cm), None) => Some(cm),
                        (None, Some(pm)) => Some(pm),
                        (None, None) => None,
                    };
                    let new_max = match (c_fill_max, p_fill_max) {
                        (Some(cm), Some(pm)) => Some(cm.min(pm)),
                        (Some(cm), None) => Some(cm),
                        (None, Some(pm)) => Some(pm),
                        (None, None) => None,
                    };
                    // Ensure min <= max if both are Some
                    let (final_min, final_max) = match (new_min, new_max) {
                        (Some(n_min), Some(n_max)) if n_min > n_max => (Some(n_max), Some(n_max)), // Or handle error/warning
                        _ => (new_min, new_max),
                    };
                    DimensionValue::Fill {
                        min: final_min,
                        max: final_max,
                    }
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_parent_wrap_child_wrap_grandchild() {
        // 父组件 Fixed(100) -> 子组件 Wrap {min: Some(Px(20)), max: Some(Px(80))} -> 子子组件 Wrap {min: Some(Px(10)), max: Some(Px(50))}

        // 父组件约束
        let parent = Constraint::new(
            DimensionValue::Fixed(Px(100)),
            DimensionValue::Fixed(Px(100)),
        );

        // 子组件约束
        let child = Constraint::new(
            DimensionValue::Wrap {
                min: Some(Px(20)),
                max: Some(Px(80)),
            },
            DimensionValue::Wrap {
                min: Some(Px(20)),
                max: Some(Px(80)),
            },
        );

        // 子子组件约束
        let grandchild = Constraint::new(
            DimensionValue::Wrap {
                min: Some(Px(10)),
                max: Some(Px(50)),
            },
            DimensionValue::Wrap {
                min: Some(Px(10)),
                max: Some(Px(50)),
            },
        );

        // 第一层合并：子组件 merge 父组件
        let merged_child = child.merge(&parent);

        // 子组件是 Wrap，父组件是 Fixed，结果应该是 Wrap，但 max 被父组件的 Fixed 值限制
        assert_eq!(
            merged_child.width,
            DimensionValue::Wrap {
                min: Some(Px(20)),
                max: Some(Px(80))
            }
        );
        assert_eq!(
            merged_child.height,
            DimensionValue::Wrap {
                min: Some(Px(20)),
                max: Some(Px(80))
            }
        );

        // 第二层合并：子子组件 merge 已合并的子组件
        let final_result = grandchild.merge(&merged_child);

        // 子子组件是 Wrap，已合并的子组件也是 Wrap，结果应该是 Wrap，max 取更小的值
        assert_eq!(
            final_result.width,
            DimensionValue::Wrap {
                min: Some(Px(10)),
                max: Some(Px(50))
            }
        );
        assert_eq!(
            final_result.height,
            DimensionValue::Wrap {
                min: Some(Px(10)),
                max: Some(Px(50))
            }
        );
    }

    #[test]
    fn test_fill_parent_wrap_child() {
        // 父组件 Fill {min: Some(Px(50)), max: Some(Px(200))} -> 子组件 Wrap {min: Some(Px(30)), max: Some(Px(150))}

        let parent = Constraint::new(
            DimensionValue::Fill {
                min: Some(Px(50)),
                max: Some(Px(200)),
            },
            DimensionValue::Fill {
                min: Some(Px(50)),
                max: Some(Px(200)),
            },
        );

        let child = Constraint::new(
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(150)),
            },
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(150)),
            },
        );

        let result = child.merge(&parent);

        // 子组件是 Wrap，父组件是 Fill，结果应该是 Wrap
        // min 保持子组件自己的值 (Px(30))
        // max 应该是子组件和父组件的较小值 (Px(150))
        assert_eq!(
            result.width,
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(150))
            }
        );
        assert_eq!(
            result.height,
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(150))
            }
        );
    }

    #[test]
    fn test_fill_parent_wrap_child_no_child_min() {
        // 测试子组件没有 min 的情况
        // 父组件 Fill {min: Some(Px(50)), max: Some(Px(200))} -> 子组件 Wrap {min: None, max: Some(Px(150))}

        let parent = Constraint::new(
            DimensionValue::Fill {
                min: Some(Px(50)),
                max: Some(Px(200)),
            },
            DimensionValue::Fill {
                min: Some(Px(50)),
                max: Some(Px(200)),
            },
        );

        let child = Constraint::new(
            DimensionValue::Wrap {
                min: None,
                max: Some(Px(150)),
            },
            DimensionValue::Wrap {
                min: None,
                max: Some(Px(150)),
            },
        );

        let result = child.merge(&parent);

        // 子组件是 Wrap，应该保持自己的 min (None)，不继承父组件的 min
        assert_eq!(
            result.width,
            DimensionValue::Wrap {
                min: None,
                max: Some(Px(150))
            }
        );
        assert_eq!(
            result.height,
            DimensionValue::Wrap {
                min: None,
                max: Some(Px(150))
            }
        );
    }

    #[test]
    fn test_fill_parent_wrap_child_no_parent_max() {
        // 测试父组件没有 max 的情况
        // 父组件 Fill {min: Some(Px(50)), max: None} -> 子组件 Wrap {min: Some(Px(30)), max: Some(Px(150))}

        let parent = Constraint::new(
            DimensionValue::Fill {
                min: Some(Px(50)),
                max: None,
            },
            DimensionValue::Fill {
                min: Some(Px(50)),
                max: None,
            },
        );

        let child = Constraint::new(
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(150)),
            },
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(150)),
            },
        );

        let result = child.merge(&parent);

        // 子组件应该保持自己的约束
        assert_eq!(
            result.width,
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(150))
            }
        );
        assert_eq!(
            result.height,
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(150))
            }
        );
    }

    #[test]
    fn test_fixed_parent_wrap_child() {
        // 测试 Fixed 父组件与 Wrap 子组件的合并
        // 父组件 Fixed(Px(100)) -> 子组件 Wrap {min: Some(Px(30)), max: Some(Px(120))}

        let parent = Constraint::new(
            DimensionValue::Fixed(Px(100)),
            DimensionValue::Fixed(Px(100)),
        );

        let child = Constraint::new(
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(120)),
            },
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(120)),
            },
        );

        let result = child.merge(&parent);

        // 子组件应该保持 Wrap，但 max 被父组件的 Fixed 值限制
        // min 保持子组件自己的值 (Px(30))
        // max 应该是子组件 max 和父组件 Fixed 值的较小值 (Px(100))
        assert_eq!(
            result.width,
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(100))
            }
        );
        assert_eq!(
            result.height,
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(100))
            }
        );
    }

    #[test]
    fn test_fixed_parent_wrap_child_no_child_max() {
        // 测试子组件没有 max 的情况
        // 父组件 Fixed(Px(100)) -> 子组件 Wrap {min: Some(Px(30)), max: None}

        let parent = Constraint::new(
            DimensionValue::Fixed(Px(100)),
            DimensionValue::Fixed(Px(100)),
        );

        let child = Constraint::new(
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: None,
            },
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: None,
            },
        );

        let result = child.merge(&parent);

        // 子组件应该保持 Wrap，父组件的 Fixed 值成为 max
        assert_eq!(
            result.width,
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(100))
            }
        );
        assert_eq!(
            result.height,
            DimensionValue::Wrap {
                min: Some(Px(30)),
                max: Some(Px(100))
            }
        );
    }

    #[test]
    fn test_fixed_parent_fill_child() {
        // 测试 Fixed 父组件与 Fill 子组件的合并
        // 父组件 Fixed(Px(100)) -> 子组件 Fill {min: Some(Px(30)), max: Some(Px(120))}

        let parent = Constraint::new(
            DimensionValue::Fixed(Px(100)),
            DimensionValue::Fixed(Px(100)),
        );

        let child = Constraint::new(
            DimensionValue::Fill {
                min: Some(Px(30)),
                max: Some(Px(120)),
            },
            DimensionValue::Fill {
                min: Some(Px(30)),
                max: Some(Px(120)),
            },
        );

        let result = child.merge(&parent);

        // 子组件应该保持 Fill，但 max 被父组件的 Fixed 值限制
        // min 保持子组件自己的值 (Px(30))
        // max 应该是子组件 max 和父组件 Fixed 值的较小值 (Px(100))
        assert_eq!(
            result.width,
            DimensionValue::Fill {
                min: Some(Px(30)),
                max: Some(Px(100))
            }
        );
        assert_eq!(
            result.height,
            DimensionValue::Fill {
                min: Some(Px(30)),
                max: Some(Px(100))
            }
        );
    }

    #[test]
    fn test_fixed_parent_fill_child_no_child_max() {
        // 测试子组件没有 max 的情况
        // 父组件 Fixed(Px(100)) -> 子组件 Fill {min: Some(Px(30)), max: None}

        let parent = Constraint::new(
            DimensionValue::Fixed(Px(100)),
            DimensionValue::Fixed(Px(100)),
        );

        let child = Constraint::new(
            DimensionValue::Fill {
                min: Some(Px(30)),
                max: None,
            },
            DimensionValue::Fill {
                min: Some(Px(30)),
                max: None,
            },
        );

        let result = child.merge(&parent);

        // 子组件应该保持 Fill，父组件的 Fixed 值成为 max
        assert_eq!(
            result.width,
            DimensionValue::Fill {
                min: Some(Px(30)),
                max: Some(Px(100))
            }
        );
        assert_eq!(
            result.height,
            DimensionValue::Fill {
                min: Some(Px(30)),
                max: Some(Px(100))
            }
        );
    }

    #[test]
    fn test_fixed_parent_fill_child_no_child_min() {
        // 测试子组件没有 min 的情况
        // 父组件 Fixed(Px(100)) -> 子组件 Fill {min: None, max: Some(Px(120))}

        let parent = Constraint::new(
            DimensionValue::Fixed(Px(100)),
            DimensionValue::Fixed(Px(100)),
        );

        let child = Constraint::new(
            DimensionValue::Fill {
                min: None,
                max: Some(Px(120)),
            },
            DimensionValue::Fill {
                min: None,
                max: Some(Px(120)),
            },
        );

        let result = child.merge(&parent);

        // 子组件应该保持 Fill，min 保持 None，max 被父组件限制
        assert_eq!(
            result.width,
            DimensionValue::Fill {
                min: None,
                max: Some(Px(100))
            }
        );
        assert_eq!(
            result.height,
            DimensionValue::Fill {
                min: None,
                max: Some(Px(100))
            }
        );
    }
}
