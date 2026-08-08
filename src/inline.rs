use core::ops::{Index, IndexMut};

use crate::{
    NewSized,
    construction::{LopeCore, LopeCoreArm},
    schedule::{Hooked, Schedule},
    storage::StorageBackend,
};

#[derive(PartialEq, Eq, Debug, Clone, PartialOrd, Ord, Hash, Copy)]
pub(crate) struct InlineStorage<T, const N: usize> {
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

    fn from_fn(f: impl Fn(usize) -> T) -> Self {
        InlineStorage {
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

pub type InlineArm<'a, Q, S: Schedule<Q>, const N: usize, const SUB_CAP: usize> = LopeCoreArm<
    'a,
    Q,
    S,
    InlineStorage<Q, N>,
    InlineStorage<<S::Arm as Hooked>::State, N>,
    SUB_CAP,
>;

pub struct InlineLope<Q, S: Schedule<Q>, const N: usize, const SUB_CAP: usize = 32> {
    raw: LopeCore<Q, S, InlineStorage<Q, N>, InlineStorage<<S::Arm as Hooked>::State, N>, SUB_CAP>,
}

impl<Q, S, const N: usize, const SUB_CAP: usize> InlineLope<Q, S, N, SUB_CAP>
where
    Q: NewSized<SUB_CAP>,
    S: Schedule<Q> + Default,
{
    pub fn new() -> Self {
        Self {
            raw: LopeCore::new_with(
                InlineStorage::from_fn(|_| <Q as NewSized<SUB_CAP>>::with_capacity()),
                InlineStorage::from_fn(|_| Default::default()),
            ),
        }
    }

    pub fn new_root(&self) -> InlineArm<'_, Q, S, N, SUB_CAP> {
        self.raw.new_root()
    }
}

impl<Q, S, const N: usize, const SUB_CAP: usize> Default for InlineLope<Q, S, N, SUB_CAP>
where
    Q: NewSized<SUB_CAP>,
    S: Schedule<Q> + Default,
{
    fn default() -> Self {
        Self::new()
    }
}
