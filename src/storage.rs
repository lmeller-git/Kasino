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

/// a storage that may dynamically grow
#[cfg(feature = "alloc")]
pub trait GrowingBackend<T>: StorageBackend<T> {
    /// pushes an item to the storage, which may grow it
    fn push(&self, item: T);
}
