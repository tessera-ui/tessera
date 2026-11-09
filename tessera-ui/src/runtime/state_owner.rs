//! Reactive state with an explicit controller or route lifetime.
//!
//! ## Usage
//!
//! Create a state owner and share its keyed state through component props.

use std::{
    any::TypeId,
    hash::{Hash, Hasher},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use rustc_hash::FxHasher;

use super::{SlotKey, State, resolve_state_key, with_slot_table_mut};

struct OwnerLease {
    id: u128,
    session: u128,
    closed: AtomicBool,
    dispatcher: crate::UiDispatcher,
}

impl OwnerLease {
    fn close(&self) {
        if self.closed.swap(true, Ordering::AcqRel) {
            return;
        }
        let owner = self.id;
        if crate::session::current_session_identity() == Some(self.session) {
            release_slots(owner);
        } else {
            self.dispatcher.dispatch(move || release_slots(owner));
        }
    }
}

impl Drop for OwnerLease {
    fn drop(&mut self) {
        self.close();
    }
}

/// Owns reactive state for a controller, route, or other explicit UI lifetime.
///
/// Clones share the lifetime. Closing the owner, or dropping its final clone,
/// releases its state. Cleanup requested from another thread runs on the owning
/// UI session before its next frame.
#[derive(Clone)]
pub struct StateOwner(Arc<OwnerLease>);

impl Default for StateOwner {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for StateOwner {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for StateOwner {}

impl StateOwner {
    /// Creates an owner in the current UI session.
    pub fn new() -> Self {
        let owner = Arc::new(OwnerLease {
            id: uuid::Uuid::new_v4().as_u128(),
            session: crate::session::current_session_identity()
                .expect("state owners require an active UI session"),
            closed: AtomicBool::new(false),
            dispatcher: crate::dispatcher::current(),
        });
        Self(owner)
    }

    /// Returns whether this lifetime still accepts state.
    pub fn is_alive(&self) -> bool {
        !self.0.closed.load(Ordering::Acquire) && self.0.dispatcher.is_alive()
    }

    /// Closes the lifetime even when other clones still exist.
    pub fn close(&self) {
        self.0.close();
    }

    /// Captures result delivery that stops when this owner closes.
    pub fn dispatcher(&self) -> crate::UiDispatcher {
        let owner = Arc::downgrade(&self.0);
        self.0.dispatcher.clone().scoped(move || {
            owner
                .upgrade()
                .is_some_and(|owner| !owner.closed.load(Ordering::Acquire))
        })
    }

    /// Gets or initializes reactive state by a key within this lifetime.
    ///
    /// Reads and writes use the same build and layout invalidation mechanisms
    /// as remembered state. The owning UI thread performs initialization.
    pub fn state_with_key<K, T>(&self, key: K, init: impl FnOnce() -> T) -> State<T>
    where
        K: Hash,
        T: 'static,
    {
        assert_eq!(
            crate::session::current_session_identity(),
            Some(self.0.session),
            "state owner belongs to a different UI session"
        );
        assert!(self.is_alive(), "state owner is closed");
        let mut hasher = FxHasher::default();
        key.hash(&mut hasher);
        let key = SlotKey {
            kind: super::SlotKind::State,
            owned_owner: Some(self.0.id),
            instance_logic_id: 0,
            slot_hash: hasher.finish(),
            type_id: TypeId::of::<T>(),
        };
        resolve_state_key(key, true, || {
            let value = init();
            assert!(self.is_alive(), "state owner closed during initialization");
            value
        })
    }
}

fn release_slots(owner: u128) {
    let removed = with_slot_table_mut(|table| {
        let slots = table.owned_slots.remove(&owner).unwrap_or_default();
        slots
            .into_iter()
            .filter_map(|slot| table.remove_slot(slot))
            .collect::<Vec<_>>()
    });
    drop(removed);
}
