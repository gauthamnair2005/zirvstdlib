//! Collection types.
//!
//! The implementations live in Rust's `alloc` crate.  They are re-exported
//! here so that kernel and userspace use one import surface and so that the
//! standard library can grow additional collection types without breaking
//! callers.

pub use alloc::collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque};
pub use alloc::string::{String, ToString};
pub use alloc::sync::Arc;
pub use alloc::vec::Vec;

/// A stack of values with a cheap `Clone` implementation.
pub type Stack<T> = Vec<T>;

/// A queue built on `VecDeque`.
pub type Queue<T> = VecDeque<T>;

/// A minimised, simple key-value map for the common small-map case.
#[derive(Clone, Debug, Default)]
pub struct VecMap<K, V> {
    entries: Vec<(K, V)>,
}

impl<K: PartialEq, V> VecMap<K, V> {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.entries
            .iter()
            .find(|(candidate, _)| candidate == key)
            .map(|(_, value)| value)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.entries
            .iter_mut()
            .find(|(candidate, _)| candidate == key)
            .map(|(_, value)| value)
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if let Some(existing) = self.get_mut(&key) {
            return Some(core::mem::replace(existing, value));
        }
        self.entries.push((key, value));
        None
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let index = self
            .entries
            .iter()
            .position(|(candidate, _)| candidate == key)?;
        Some(self.entries.swap_remove(index).1)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.entries.iter().map(|(key, value)| (key, value))
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
