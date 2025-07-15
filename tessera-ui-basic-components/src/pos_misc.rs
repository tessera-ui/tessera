//! Contains some convenience functions for positioning and sizing.

use tessera_ui::{ComputedData, Px, PxPosition};

pub fn is_position_in_component(size: ComputedData, position: PxPosition) -> bool {
    is_position_in_rect(position, PxPosition::ZERO, size.width, size.height)
}

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

    x >= rect_x && x <= rect_x + rect_width && y >= rect_y && y <= rect_y + rect_height
}

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
