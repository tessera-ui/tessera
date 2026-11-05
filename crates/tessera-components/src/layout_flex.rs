//! Shared pure layout math for row and column.

use tessera_ui::{AxisConstraint, Px};

use crate::alignment::{CrossAxisAlignment, MainAxisAlignment};

/// Returns the cross-axis constraint offered to a child of a flex container.
///
/// `Stretch` makes the child fill the container along the cross axis, while
/// every other alignment measures the child intrinsically so that the
/// cross-axis offset computed for `Center` or `End` stays meaningful.
pub(crate) fn cross_axis_child_constraint(
    container_cross: AxisConstraint,
    alignment: CrossAxisAlignment,
) -> AxisConstraint {
    match alignment {
        CrossAxisAlignment::Stretch => container_cross,
        _ => container_cross.without_min(),
    }
}
pub(crate) fn classify_children(child_weights: &[f32]) -> (Vec<usize>, Vec<usize>, f32) {
    let mut weighted_indices = Vec::new();
    let mut unweighted_indices = Vec::new();
    let mut total_weight = 0.0;
    for (i, &weight) in child_weights.iter().enumerate() {
        if weight > 0.0 {
            weighted_indices.push(i);
            total_weight += weight;
        } else {
            unweighted_indices.push(i);
        }
    }
    (weighted_indices, unweighted_indices, total_weight)
}

/// Returns the main-axis start offset and the gap between children.
pub(crate) fn main_axis_offsets(
    container: Px,
    children_total: Px,
    alignment: MainAxisAlignment,
    child_count: usize,
) -> (f32, f32) {
    let available_space = (container.0 - children_total.0).max(0) as f32;
    alignment_offsets(available_space, child_count, alignment)
}

/// Returns the cross-axis offset for one child.
pub(crate) fn cross_axis_offset(
    child_cross: Px,
    container_cross: Px,
    alignment: CrossAxisAlignment,
) -> f32 {
    match alignment {
        CrossAxisAlignment::Start | CrossAxisAlignment::Stretch => 0.0,
        CrossAxisAlignment::Center => (container_cross.0 - child_cross.0).max(0) as f32 / 2.0,
        CrossAxisAlignment::End => (container_cross.0 - child_cross.0).max(0) as f32,
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod drawing_tests;

pub(crate) fn px_mul(px: Px, times: usize) -> Px {
    if times == 0 {
        return Px::ZERO;
    }
    px_from_i64(px.0 as i64 * times as i64)
}

pub(crate) fn px_from_i64(value: i64) -> Px {
    if value > i64::from(i32::MAX) {
        Px(i32::MAX)
    } else if value < i64::from(i32::MIN) {
        Px(i32::MIN)
    } else {
        Px(value as i32)
    }
}

pub(crate) fn alignment_offsets(
    available_space: f32,
    count: usize,
    alignment: MainAxisAlignment,
) -> (f32, f32) {
    match alignment {
        MainAxisAlignment::Start => (0.0, 0.0),
        MainAxisAlignment::Center => (available_space / 2.0, 0.0),
        MainAxisAlignment::End => (available_space, 0.0),
        MainAxisAlignment::SpaceEvenly => {
            if count > 0 {
                let s = available_space / (count as f32 + 1.0);
                (s, s)
            } else {
                (0.0, 0.0)
            }
        }
        MainAxisAlignment::SpaceBetween => {
            if count > 1 {
                (0.0, available_space / (count as f32 - 1.0))
            } else if count == 1 {
                (available_space / 2.0, 0.0)
            } else {
                (0.0, 0.0)
            }
        }
        MainAxisAlignment::SpaceAround => {
            if count > 0 {
                let s = available_space / (count as f32);
                (s / 2.0, s)
            } else {
                (0.0, 0.0)
            }
        }
    }
}
