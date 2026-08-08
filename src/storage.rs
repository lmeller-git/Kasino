use core::ops::{Index, IndexMut};

pub trait StorageBackend<T>: Index<usize, Output = T> + IndexMut<usize> {
    fn capacity(&self) -> usize;
    fn len(&self) -> usize;

    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a;

    fn from_fn<R>(f: impl Fn(usize) -> R) -> impl StorageBackend<R>;
}

#[cfg(feature = "alloc")]
pub trait GrowingBackend<T>: StorageBackend<T> {
    fn push(&self, item: T);
}
