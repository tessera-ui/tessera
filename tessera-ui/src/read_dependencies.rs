use std::{
    collections::{hash_map::Entry, hash_set},
    hash::Hash,
};

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use smallvec::SmallVec;

pub(crate) struct ReadDependencies<K, R> {
    readers_by_key: HashMap<K, Readers<R>>,
    keys_by_reader: HashMap<R, SmallVec<[K; 1]>>,
}

impl<K, R> Default for ReadDependencies<K, R> {
    fn default() -> Self {
        Self {
            readers_by_key: HashMap::default(),
            keys_by_reader: HashMap::default(),
        }
    }
}

impl<K: Copy + Eq + Hash, R: Copy + Eq + Hash> ReadDependencies<K, R> {
    pub(crate) fn record(&mut self, key: K, reader: R) {
        match self.readers_by_key.entry(key) {
            Entry::Vacant(entry) => {
                entry.insert(Readers::One(reader));
            }
            Entry::Occupied(mut entry) => {
                if !entry.get_mut().insert(reader) {
                    return;
                }
            }
        }
        self.keys_by_reader.entry(reader).or_default().push(key);
    }

    pub(crate) fn readers(&self, key: &K) -> Option<impl Iterator<Item = R> + '_> {
        self.readers_by_key.get(key).map(Readers::iter)
    }

    pub(crate) fn remove_reader(&mut self, reader: &R) {
        let Some(keys) = self.keys_by_reader.remove(reader) else {
            return;
        };
        for key in keys {
            if let Entry::Occupied(mut entry) = self.readers_by_key.entry(key)
                && entry.get_mut().remove_and_is_empty(reader)
            {
                entry.remove();
            }
        }
    }

    pub(crate) fn remove_readers(&mut self, readers: &HashSet<R>) {
        for reader in readers {
            self.remove_reader(reader);
        }
    }
}

enum Readers<R> {
    One(R),
    Many(Box<HashSet<R>>),
}

impl<R: Copy + Eq + Hash> Readers<R> {
    fn insert(&mut self, reader: R) -> bool {
        match self {
            Self::One(existing) if *existing == reader => false,
            Self::One(existing) => {
                let mut readers = HashSet::default();
                readers.insert(*existing);
                readers.insert(reader);
                *self = Self::Many(Box::new(readers));
                true
            }
            Self::Many(readers) => readers.insert(reader),
        }
    }

    fn iter(&self) -> impl Iterator<Item = R> + '_ {
        let (one, many) = match self {
            Self::One(reader) => (Some(*reader), hash_set::Iter::default()),
            Self::Many(readers) => (None, readers.iter()),
        };
        one.into_iter().chain(many.copied())
    }

    fn remove_and_is_empty(&mut self, reader: &R) -> bool {
        match self {
            Self::One(existing) => existing == reader,
            Self::Many(readers) => {
                readers.remove(reader);
                // A layout phase can clear and record the same reader in one frame.
                readers.is_empty()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ReadDependencies;

    #[test]
    fn removing_a_reader_preserves_other_readers_of_shared_keys() {
        let mut dependencies = ReadDependencies::default();
        dependencies.record("shared", 1);
        dependencies.record("shared", 1);
        dependencies.record("private", 1);
        dependencies.record("shared", 2);
        assert_eq!(dependencies.readers(&"shared").unwrap().count(), 2);

        dependencies.remove_reader(&1);
        assert!(dependencies.readers(&"shared").unwrap().eq([2]));
        assert!(dependencies.readers(&"private").is_none());

        dependencies.record("shared", 1);
        dependencies.remove_reader(&2);
        assert!(dependencies.readers(&"shared").unwrap().eq([1]));
        dependencies.remove_reader(&1);
        assert!(dependencies.readers(&"shared").is_none());
    }
}
