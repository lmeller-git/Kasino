use core::ops::{Index, IndexMut};

use crate::{
    Collection,
    construction::LopeCore,
    schedule::{DCBO, Hooked, Schedule},
    storage::StorageBackend,
};

struct InlineStorage<T, const N: usize> {
    arr: [T; N],
}

impl<T: Default, const N: usize> Default for InlineStorage<T, N> {
    fn default() -> Self {
        Self {
            arr: core::array::from_fn(|_| Default::default()),
        }
    }
}

impl<T, const N: usize> StorageBackend<T> for InlineStorage<T, N> {
    fn capacity(&self) -> usize {
        self.arr.len()
    }

    fn len(&self) -> usize {
        self.arr.len()
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a,
    {
        self.arr.iter()
    }

    fn from_fn<R>(f: impl Fn(usize) -> R) -> impl StorageBackend<R> {
        InlineStorage::<R, N> {
            arr: core::array::from_fn(f),
        }
    }
}

impl<T, const N: usize> Index<usize> for InlineStorage<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &<InlineStorage<T, N> as Index<usize>>::Output {
        &self.arr[index]
    }
}

impl<T, const N: usize> IndexMut<usize> for InlineStorage<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut <InlineStorage<T, N> as Index<usize>>::Output {
        &mut self.arr[index]
    }
}

pub type InlineLope<Q: Collection, S: Schedule<Q::Item>, const N: usize> =
    LopeCore<Q, S, InlineStorage<Q, N>, InlineStorage<<S::Arm as Hooked>::State, N>>;

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

fn foo() {
    let f: InlineLope<Foo, DCBO, 3> = InlineLope::new();
    let mut a = f.new_root();
    let mut b = a.fork();
    let mut c = a.fork();
    let mut d = c.fork();

    a.push(());
    b.pop();

    c.len();
}
