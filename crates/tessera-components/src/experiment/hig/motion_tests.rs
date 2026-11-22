use std::time::Duration;

use crate::animation::{Motion as _, test_instant};

use super::Eased;

fn ms(value: u64) -> tessera_ui::time::Instant {
    test_instant(value * 1_000_000)
}

#[test]
fn reversing_midway_continues_from_the_current_value_for_the_remaining_distance() {
    let mut eased = Eased::new(0.0, Duration::from_millis(200));
    eased.retarget(1.0, ms(0));
    eased.advance(ms(100));
    assert!((eased.value() - 0.5).abs() < 1.0e-3);
    eased.retarget(0.0, ms(100));
    assert!((eased.value() - 0.5).abs() < 1.0e-3, "no jump on reversal");
    assert!(eased.advance(ms(150)));
    let midway = eased.value();
    assert!(0.0 < midway && midway < 0.5);
    assert!(
        !eased.advance(ms(200)),
        "half the distance takes half the time"
    );
    assert_eq!(eased.value(), 0.0);
    assert!(eased.is_settled_at(&0.0));
}
