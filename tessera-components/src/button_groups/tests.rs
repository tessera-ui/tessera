//! Width exchange and press completion regressions.
//!
//! ## Usage
//!
//! Preserve the width budget while buttons expand and return to rest.

use tessera_ui::Px;

use super::width_motion::{PressWidthMotion, WidthData, redistribute_widths};

fn data(progress: [f64; 3], limit: i32) -> [WidthData; 3] {
    progress.map(|progress| WidthData {
        progress,
        compression_limit: Px(limit),
    })
}

#[test]
fn width_exchange_uses_neighbors_and_preserves_the_total() {
    for (progress, expected) in [
        ([1.0, 0.0, 0.0], [115, 85, 100]),
        ([0.0, 1.0, 0.0], [92, 116, 92]),
        ([0.0, 0.0, 1.0], [100, 85, 115]),
        ([1.0, 0.0, 1.0], [115, 70, 115]),
    ] {
        let mut widths = [Px(100); 3];
        redistribute_widths(&mut widths, &data(progress, 16), 0.15);
        assert_eq!(widths.map(|width| width.0), expected);
        assert_eq!(widths.iter().map(|width| width.0).sum::<i32>(), 300);
    }
    let mut widths = [Px(100); 3];
    redistribute_widths(&mut widths, &data([0.0, 1.0, 0.0], 4), 1.0);
    assert_eq!(widths, [Px(96), Px(108), Px(96)]);
    let mut widths = [Px(100); 3];
    redistribute_widths(&mut widths, &data([0.0, -0.03125, 0.0], 16), 1.0);
    assert_eq!(
        widths,
        [Px(100); 3],
        "negative half-pixel spring overshoot rounds toward positive infinity"
    );
}

#[test]
fn width_exchange_respects_tiny_neighbors_and_singletons() {
    let mut widths = [Px(6), Px(100), Px(4)];
    redistribute_widths(&mut widths, &data([0.0, 1.0, 0.0], 16), 0.15);
    assert_eq!(widths, [Px(0), Px(110), Px(0)]);
    let mut singleton = [Px(100)];
    redistribute_widths(&mut singleton, &data([1.0, 0.0, 0.0], 16), 1.0);
    assert_eq!(singleton, [Px(100)]);
    let mut widths = [Px(100); 3];
    redistribute_widths(&mut widths, &data([0.0, -100.0, 0.0], 16), 1.0);
    assert_eq!(widths, [Px(150), Px(0), Px(150)]);
}

#[test]
fn short_press_finishes_a_visible_expansion_before_returning() {
    let mut motion = PressWidthMotion::default();
    motion.update(false, true, 0);
    motion.update(true, true, 0);
    motion.update(false, true, 1);
    motion.advance(80_000_000);
    assert!(motion.value() >= 0.75);
    assert!(motion.is_animating());
    motion.advance(1_000_000_000);
    assert_eq!(motion.value(), 0.0);
    assert!(!motion.is_animating());
}

#[test]
fn held_press_stays_expanded_and_release_stops_the_clock() {
    let mut motion = PressWidthMotion::default();
    motion.update(true, true, 0);
    motion.advance(1_000_000_000);
    assert_eq!(motion.value(), 1.0);
    assert!(!motion.is_animating());
    motion.update(false, true, 1_000_000_000);
    assert!(motion.is_animating());
    motion.advance(2_000_000_000);
    assert_eq!(motion.value(), 0.0);
    assert!(!motion.is_animating());
}

#[test]
fn disabled_motion_snaps_and_reactivation_keeps_interrupted_values() {
    let mut motion = PressWidthMotion::default();
    motion.update(false, true, 0);
    motion.pulse(0);
    motion.advance(80_000_000);
    let interrupted = motion.value();
    motion.update(true, true, 80_000_000);
    assert_eq!(motion.value(), interrupted);
    motion.update(false, false, 100_000_000);
    assert_eq!(motion.value(), 0.0);
    motion.pulse(100_000_000);
    assert!(!motion.is_animating());
    motion.update(false, true, 100_000_000);
    motion.pulse(100_000_000);
    assert!(motion.is_animating());
}
