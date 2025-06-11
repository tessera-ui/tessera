//! Contains some convenience functions for positioning and sizing.

use tessera::ComputedData;

pub fn is_position_in_component(size: ComputedData, position: [i32; 2]) -> bool {
    is_position_in_rect(position, [0, 0, size.width as i32, size.height as i32])
}

pub fn is_position_in_rect(position: [i32; 2], rect: [i32; 4]) -> bool {
    let [x, y] = position;
    let [rect_x, rect_y, rect_width, rect_height] = rect;

    x >= rect_x && x <= rect_x + rect_width && y >= rect_y && y <= rect_y + rect_height
}
