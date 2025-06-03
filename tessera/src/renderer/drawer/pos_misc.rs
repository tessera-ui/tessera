pub fn pixel_to_ndc(pos: [f32; 2], screen_size: [u32; 2]) -> [f32; 2] {
    let x = pos[0] / screen_size[0] as f32 * 2.0 - 1.0;
    let y = pos[1] / screen_size[1] as f32 * 2.0 - 1.0;
    // Invert y axis
    // because the origin is at the bottom left corner in OpenGL
    // but we want the origin to be at the top left corner, since
    // ui is always top-down
    let y = -y;

    [x, y]
}
