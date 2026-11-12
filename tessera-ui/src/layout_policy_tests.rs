use std::{cell::Cell, rc::Rc};

use crate::{
    MeasurementError,
    layout::{LayoutPolicyHandle, LayoutResult, MeasureScope},
    modifier::LayoutPolicy,
};

struct CopyTrackedPolicy {
    value: u32,
    clones: Rc<Cell<usize>>,
}

impl Clone for CopyTrackedPolicy {
    fn clone(&self) -> Self {
        self.clones.set(self.clones.get() + 1);
        Self {
            value: self.value,
            clones: self.clones.clone(),
        }
    }
}

impl PartialEq for CopyTrackedPolicy {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl LayoutPolicy for CopyTrackedPolicy {
    fn measure(&self, _: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        unreachable!("policy identity test does not measure")
    }
}

#[test]
fn equal_policy_recomposition_keeps_the_stored_value_without_cloning() {
    let clones = Rc::new(Cell::new(0));
    let mut stored = LayoutPolicyHandle::from(CopyTrackedPolicy {
        value: 1,
        clones: clones.clone(),
    });
    let original = stored.as_policy() as *const _ as *const () as usize;
    let equal_candidate = CopyTrackedPolicy {
        value: 1,
        clones: clones.clone(),
    };

    assert!(!stored.update_if_changed(&equal_candidate));
    assert_eq!(clones.get(), 0);
    assert_eq!(
        stored.as_policy() as *const _ as *const () as usize,
        original
    );

    let changed_candidate = CopyTrackedPolicy {
        value: 2,
        clones: clones.clone(),
    };
    assert!(stored.update_if_changed(&changed_candidate));
    assert_eq!(clones.get(), 1);
    assert_ne!(
        stored.as_policy() as *const _ as *const () as usize,
        original
    );
    assert_eq!(
        stored
            .as_policy()
            .as_any()
            .downcast_ref::<CopyTrackedPolicy>()
            .expect("stored policy type")
            .value,
        2
    );
}
