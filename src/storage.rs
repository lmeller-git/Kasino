//! Storage abstractions for different kinds of storages

use core::ops::{Index, IndexMut};

/// a generic storage
pub trait StorageBackend<T>: Index<usize, Output = T> + IndexMut<usize> {
    /// the current length of the storage
    fn len(&self) -> usize;
    /// returns an iterator over all items in the storage
    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a;
    /// is the storgae empty?
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
