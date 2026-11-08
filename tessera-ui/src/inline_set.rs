use std::{
    collections::{hash_map::Entry, hash_set},
    hash::Hash,
};

use rustc_hash::FxHashSet as HashSet;

pub(crate) enum InlineSet<T> {
    One(T),
    Many(Box<HashSet<T>>),
}

impl<T: Copy + Eq + Hash> InlineSet<T> {
    pub(crate) fn insert_at<K>(entry: Entry<'_, K, Self>, value: T) -> bool {
        match entry {
            Entry::Vacant(entry) => {
                entry.insert(Self::One(value));
                true
            }
            Entry::Occupied(mut entry) => {
                let set = entry.get_mut();
                match set {
                    Self::One(existing) if *existing == value => false,
                    Self::One(existing) => {
                        *set = Self::Many(Box::new([*existing, value].into_iter().collect()));
                        true
                    }
                    Self::Many(values) => values.insert(value),
                }
            }
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = T> + '_ {
        let (one, many) = match self {
            Self::One(value) => (Some(*value), hash_set::Iter::default()),
            Self::Many(values) => (None, values.iter()),
        };
        one.into_iter().chain(many.copied())
    }

    pub(crate) fn remove_and_is_empty(&mut self, value: &T) -> bool {
        match self {
            Self::One(existing) => existing == value,
            Self::Many(values) => {
                values.remove(value);
                // Keep capacity when the next operation can add a value back.
                values.is_empty()
            }
        }
    }
}
