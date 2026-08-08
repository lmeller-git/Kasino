use core::ops::{Index, IndexMut};

#[cfg(feature = "alloc")]
use crate::storage::GrowingBackend;
use crate::{
    Collection,
    NewSized,
    construction::LopeCore,
    schedule::{DCBO, Hooked, Schedule},
    storage::StorageBackend,
};

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

impl<T> BoxedStorage<T> {
    pub(crate) fn new(size: usize) -> Self {
        Self {
            arr: boxcar::Vec::with_capacity(size),
        }
    }

    pub(crate) fn from_fn_and_size(f: impl Fn(usize) -> T, size: usize) -> Self {
        Self {
            arr: (0..size).map(f).collect(),
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

    fn from_fn(_: impl Fn(usize) -> T) -> Self {
        BoxedStorage {
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

pub struct BoxedLope<Q: Collection, S: Schedule<Q>, const SUB_CAP: usize = 32> {
    raw: LopeCore<Q, S, BoxedStorage<Q>, BoxedStorage<<S::Arm as Hooked>::State>, SUB_CAP>,
}

impl<Q: Collection + NewSized<SUB_CAP>, S: Schedule<Q> + Default, const SUB_CAP: usize>
    BoxedLope<Q, S, SUB_CAP>
{
    pub fn new(n_cores: usize) -> Self {
        Self {
            raw: LopeCore::new_with(
                BoxedStorage::from_fn_and_size(
                    |_| <Q as NewSized<SUB_CAP>>::with_capacity(),
                    n_cores,
                ),
                BoxedStorage::from_fn_and_size(|_| Default::default(), n_cores),
            ),
        }
    }
}

#[derive(Default)]
struct Foo {}

impl Collection for Foo {
    type Item = ();

    fn push(&self, item: Self::Item) -> Result<(), Self::Item> {
        todo!()
    }

    fn pop(&self) -> Option<Self::Item> {
        todo!()
    }

    fn len(&self) -> usize {
        todo!()
    }

    fn cap(&self) -> usize {
        todo!()
    }
}

#[cfg(feature = "alloc")]
fn foo() {
    let f: BoxedLope<Foo, DCBO> = BoxedLope::new(3);
    let mut a = f.new_root();
    let mut b = a.fork();
    let mut c = a.fork();
    let mut d = c.fork();
    f.add_queue(Default::default());
    d.add_queue(Default::default());

    a.push(());
    b.pop();

    c.len();
}
