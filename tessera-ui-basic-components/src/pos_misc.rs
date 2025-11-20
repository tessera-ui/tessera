//! Convenience utilities for testing whether a cursor/point falls inside a
//! component or a rectangle. Functions operate using Px units and simple
//! inclusive comparisons on edges.

use tessera_ui::{ComputedData, Px, PxPosition};

/// Returns true if `position` is inside a component of the given `size`.
/// The component is assumed to be located at the origin (0, 0).
///
/// This is a small convenience wrapper around [`is_position_in_rect`].
pub fn is_position_in_component(size: ComputedData, position: PxPosition) -> bool {
    is_position_in_rect(position, PxPosition::ZERO, size.width, size.height)
}

/// Returns true when `position` lies within the rectangle defined by
/// `rect_pos`, `rect_width` and `rect_height` (inclusive).
///
/// Coordinates use Px units and comparisons are inclusive on edges. The check
/// evaluates X and Y independently and returns true only if both are within
/// bounds.
pub fn is_position_in_rect(
    position: PxPosition,
    rect_pos: PxPosition,
    rect_width: Px,
    rect_height: Px,
) -> bool {
    let x = position.x;
    let y = position.y;
    let rect_x = rect_pos.x;
    let rect_y = rect_pos.y;

    // Check X and Y independently and combine for clarity.
    let within_x = x >= rect_x && x <= rect_x + rect_width;
    let within_y = y >= rect_y && y <= rect_y + rect_height;
    within_x && within_y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_position_in_component() {
        let size = ComputedData {
            width: Px(100),
            height: Px(50),
        };
        assert!(is_position_in_component(
            size,
            PxPosition::new(Px(50), Px(25))
        ));
        assert!(!is_position_in_component(
            size,
            PxPosition::new(Px(150), Px(25))
        ));
        assert!(!is_position_in_component(
            size,
            PxPosition::new(Px(50), Px(75))
        ));
    }
}

