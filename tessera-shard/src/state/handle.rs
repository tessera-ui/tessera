use std::{
    any::{TypeId, type_name},
    hash::Hash,
    marker::PhantomData,
    sync::{Arc, Weak},
};

use parking_lot::RwLock;

use crate::state::{
    ShardStateSlot,
    storage::{ErasedShardStateHandle, ShardSlotEntry, ShardSlotTable, ShardSlotTableHandle},
};

/// Failure while resolving a [`ShardState`] against its controller table.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShardStateError {
    /// The owning controller (and its table) has been dropped.
    ControllerDropped,
    /// The slot index is outside the owning table.
    MissingSlot,
    /// The slot was recycled, or the handle belongs to another controller.
    Stale {
        /// Controller id recorded in the handle.
        controller: u64,
        /// Slot index recorded in the handle.
        slot: u32,
        /// Generation recorded in the handle.
        generation: u64,
    },
    /// The live slot stores a different type than `T`.
    TypeMismatch {
        /// Slot index recorded in the handle.
        slot: u32,
    },
    /// The slot generation matches but the value was already taken.
    Recycled {
        /// Slot index recorded in the handle.
        slot: u32,
    },
}

/// `Weak` is not ordinarily `Copy`. The handle stores its bits as `usize` and
/// copies those bits. The controller's strong `Arc` owns the table.
///
/// # Safety invariant
///
/// - `from_slot` clones one `Weak`, converts it with `into_raw`, and forgets
///   the original, so the weak count stays at least 1 for the copied bits.
/// - resolving rebuilds that `Weak` with `from_raw` and forgets it again, so
///   the weak count does not change. `upgrade` returns the controller table, or
///   `None` after the last strong `Arc` is dropped.
/// - controller id and generation are checked before the value is touched, so a
///   recycled slot number cannot return another controller's `T`.
///
/// Using a handle after its controller drops the table returns
/// [`ShardStateError::ControllerDropped`] from the fallible methods.
pub struct ShardState<T> {
    controller: u64,
    slot: u32,
    generation: u64,
    /// Bits of `Weak<RwLock<ShardSlotTable>>`. See the safety invariant above.
    table: usize,
    _marker: PhantomData<T>,
}

impl<T> Copy for ShardState<T> {}

impl<T> Clone for ShardState<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for ShardState<T> {
    fn eq(&self, other: &Self) -> bool {
        self.controller == other.controller
            && self.slot == other.slot
            && self.generation == other.generation
            && self.table == other.table
    }
}

impl<T> Eq for ShardState<T> {}

impl<T> Hash for ShardState<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.controller.hash(state);
        self.slot.hash(state);
        self.generation.hash(state);
        self.table.hash(state);
    }
}

impl<T> ShardState<T>
where
    T: Send + Sync + 'static,
{
    pub(crate) fn from_slot(table_handle: &ShardSlotTableHandle, slot: ShardStateSlot) -> Self {
        debug_assert_eq!(table_handle.id(), slot.controller);
        let table = table_handle.downgrade();
        let pinned = table.clone();
        std::mem::forget(table);
        Self {
            controller: slot.controller,
            slot: slot.slot,
            generation: slot.generation,
            table: Weak::into_raw(pinned) as usize,
            _marker: PhantomData,
        }
    }

    fn resolve_table(&self) -> Result<Arc<RwLock<ShardSlotTable>>, ShardStateError> {
        if self.table == 0 {
            return Err(ShardStateError::ControllerDropped);
        }
        // Rebuild the leaked `Weak` without changing its refcount.
        let weak = unsafe { Weak::from_raw(self.table as *const RwLock<ShardSlotTable>) };
        let table = weak.upgrade();
        std::mem::forget(weak);
        let table = table.ok_or(ShardStateError::ControllerDropped)?;
        let guard = table.read();
        if guard.controller_id != self.controller {
            return Err(ShardStateError::Stale {
                controller: self.controller,
                slot: self.slot,
                generation: self.generation,
            });
        }
        drop(guard);
        Ok(table)
    }

    fn check_entry<'a>(
        &self,
        entry: Option<&'a ShardSlotEntry>,
    ) -> Result<&'a ErasedShardStateHandle, ShardStateError> {
        let Some(entry) = entry else {
            return Err(ShardStateError::MissingSlot);
        };
        if entry.generation != self.generation {
            return Err(ShardStateError::Stale {
                controller: self.controller,
                slot: self.slot,
                generation: self.generation,
            });
        }
        if entry.type_id != TypeId::of::<T>() {
            return Err(ShardStateError::TypeMismatch { slot: self.slot });
        }
        entry
            .value
            .as_ref()
            .ok_or(ShardStateError::Recycled { slot: self.slot })
    }

    fn panic_on_error<R>(&self, result: Result<R, ShardStateError>) -> R {
        match result {
            Ok(value) => value,
            Err(ShardStateError::ControllerDropped) => {
                panic!("ShardState controller has been dropped")
            }
            Err(ShardStateError::MissingSlot) => {
                panic!("ShardState points to freed slot: {}", self.slot)
            }
            Err(ShardStateError::Stale {
                controller,
                slot,
                generation,
            }) => {
                panic!(
                    "ShardState is stale (controller {controller}, slot {slot}, generation {generation})"
                )
            }
            Err(ShardStateError::TypeMismatch { slot }) => {
                panic!(
                    "ShardState type mismatch for slot {slot}: expected {}",
                    type_name::<T>()
                )
            }
            Err(ShardStateError::Recycled { slot }) => {
                panic!("ShardState slot {slot} has been recycled")
            }
        }
    }

    /// Execute a closure with a shared reference to the value.
    ///
    /// The table lock is held for the duration of `f`.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.panic_on_error(self.try_with(f))
    }

    /// Execute a closure with a mutable reference to the value.
    ///
    /// The value is moved out of the table for `f`, then written back if this
    /// generation is still current. The table lock is not held across `f`.
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        self.panic_on_error(self.try_with_mut(f))
    }

    /// Fallible shared access. See [`Self::with`].
    pub fn try_with<R>(&self, f: impl FnOnce(&T) -> R) -> Result<R, ShardStateError> {
        let table = self.resolve_table()?;
        let guard = table.read();
        let value = self.check_entry(guard.entries.get(self.slot as usize))?;
        let value = value
            .downcast_ref::<T>()
            .ok_or(ShardStateError::TypeMismatch { slot: self.slot })?;
        Ok(f(value))
    }

    /// Fallible mutable access. See [`Self::with_mut`].
    pub fn try_with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> Result<R, ShardStateError> {
        let table = self.resolve_table()?;
        let mut owned = {
            let mut guard = table.write();
            let Some(entry) = guard.entries.get_mut(self.slot as usize) else {
                return Err(ShardStateError::MissingSlot);
            };
            if entry.generation != self.generation {
                return Err(ShardStateError::Stale {
                    controller: self.controller,
                    slot: self.slot,
                    generation: self.generation,
                });
            }
            if entry.type_id != TypeId::of::<T>() {
                return Err(ShardStateError::TypeMismatch { slot: self.slot });
            }
            let erased = entry
                .value
                .take()
                .ok_or(ShardStateError::Recycled { slot: self.slot })?;
            erased
                .downcast::<T>()
                .map_err(|_| ShardStateError::TypeMismatch { slot: self.slot })?
        };

        let result = f(Arc::get_mut(&mut owned).unwrap_or_else(|| {
            panic!(
                "ShardState slot {} is still shared; values are not wrapped in a lock",
                self.slot
            )
        }));

        let mut guard = table.write();
        if let Some(entry) = guard.entries.get_mut(self.slot as usize)
            && entry.generation == self.generation
            && entry.value.is_none()
        {
            entry.value = Some(owned);
        }
        Ok(result)
    }

    /// Get a cloned value. Requires `T: Clone`.
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.with(Clone::clone)
    }

    /// Replace the stored value.
    pub fn set(&self, value: T) {
        self.with_mut(|slot| *slot = value);
    }
}
