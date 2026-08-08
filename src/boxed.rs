use core::ops::{Index, IndexMut};

#[cfg(feature = "alloc")]
use crate::storage::GrowingBackend;
use crate::storage::StorageBackend;

struct BoxedStorage<T> {
    arr: boxcar::Vec<T>,
}

impl<T: Default> Default for BoxedStorage<T> {
    fn default() -> Self {
        Self {
            arr: Default::default(),
        }
    }
}

impl<T> StorageBackend<T> for BoxedStorage<T> {
    fn capacity(&self) -> usize {
        self.arr.count()
    }

    fn len(&self) -> usize {
        self.arr.count()
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a,
    {
        self.arr.iter().map(|(_, item)| item)
    }

    fn from_fn<R>(_: impl Fn(usize) -> R) -> impl StorageBackend<R> {
        BoxedStorage::<R> {
            arr: boxcar::vec::Vec::default(),
        }
    }
}

impl<T> Index<usize> for BoxedStorage<T> {
    type Output = T;

    fn index(&self, index: usize) -> &<BoxedStorage<T> as Index<usize>>::Output {
        &self.arr[index]
    }
}

impl<T> IndexMut<usize> for BoxedStorage<T> {
    fn index_mut(&mut self, index: usize) -> &mut <BoxedStorage<T> as Index<usize>>::Output {
        self.arr.get_mut(index).unwrap()
    }
}

#[cfg(feature = "alloc")]
impl<T> GrowingBackend<T> for BoxedStorage<T> {
    fn push(&self, item: T) {
        self.arr.push(item);
    }
}
