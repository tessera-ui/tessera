//! Decorative pointer geometry for instructional figures.
//!
//! ## Usage
//!
//! Show pointer input examples without adding input or accessibility targets.

pub(super) fn hand_paths(positions: &[(f64, f64)], scale: f64) -> String {
    let outline = "M9 18V4Q9 1 12 1Q15 1 15 4V8Q15 6 17 7Q19 7 19 10Q19 8 21 9Q23 9 23 12Q23 10 25 12Q27 12 27 15V27Q27 31 24 34V36H12V35Q8 33 7 29L1 19Q-1 16 2 14Q3 13 5 15Z";
    let fingers = "M15 8V15M19 10V15M23 12V15";
    let cuff = "M12 36H25Q27 36 27 38Q27 40 25 40H12Q10 40 10 38Q10 36 12 36Z";
    let mut svg = String::new();
    for (x, y) in positions {
        svg.push_str(&format!("<g transform='translate({x} {y}) scale({scale})' fill='#fef7ff' stroke='#1d1b20' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><path d='{outline}'/><path d='{fingers}' fill='none'/><path d='{cuff}'/></g>"));
    }
    svg
}
