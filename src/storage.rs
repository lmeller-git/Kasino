use core::ops::{Index, IndexMut};

pub trait StorageBackend<T>: Index<usize, Output = T> + IndexMut<usize> {
    fn capacity(&self) -> usize;
    fn len(&self) -> usize;

    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a;

    fn from_fn(f: impl Fn(usize) -> T) -> Self;
}

#[cfg(feature = "alloc")]
pub trait GrowingBackend<T>: StorageBackend<T> {
    fn push(&self, item: T);
}
