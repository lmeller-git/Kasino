use core::ops::{Index, IndexMut};

use crate::{
    Collection,
    WithCapacity,
    construction::{BanditCore, BanditHandle, DEFAULT_QUEUE_CAP},
    storage::StorageBackend,
    strategy::{Hooked, Strategy},
};

/// an array
#[derive(PartialEq, Eq, Debug, Clone, PartialOrd, Ord, Hash, Copy)]
pub struct InlineStorage<T, const N: usize> {
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
    type Rebind<U> = InlineStorage<U, N>;

    fn len(&self) -> usize {
        self.arr.len()
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a,
    {
        self.arr.iter()
    }

    fn map_to_buffer<U>(&self, f: impl Fn(usize) -> U) -> Self::Rebind<U> {
        InlineStorage {
            arr: core::array::from_fn(f),
        }
    }
}

impl<T, const N: usize> InlineStorage<T, N> {
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

/// A handle to the core subcollection storage that is stored inline
#[allow(type_alias_bounds)]
pub type InlineBanditHandle<
    'a,
    Q: Collection,
    S: Strategy<Q>,
    const N: usize,
    const SUB_CAP: usize = DEFAULT_QUEUE_CAP,
> = BanditHandle<
    'a,
    Q,
    S,
    InlineStorage<Q, N>,
    InlineStorage<<S::Gambler as Hooked>::Stake, N>,
    SUB_CAP,
>;

/// a subcollections storage that is stored inline
pub struct InlineBandit<
    Q: Collection,
    S: Strategy<Q>,
    const N: usize,
    const SUB_CAP: usize = DEFAULT_QUEUE_CAP,
> {
    raw: BanditCore<
        Q,
        S,
        InlineStorage<Q, N>,
        InlineStorage<<S::Gambler as Hooked>::Stake, N>,
        SUB_CAP,
    >,
}

impl<Q: Collection, S, const N: usize, const SUB_CAP: usize> InlineBandit<Q, S, N, SUB_CAP>
where
    Q: WithCapacity<SUB_CAP>,
    S: Strategy<Q> + Default,
{
    /// constructs a new `InlineLope`
    pub fn new() -> Self {
        Self {
            raw: BanditCore::new_with(
                InlineStorage::from_fn(|_| <Q as WithCapacity<SUB_CAP>>::with_capacity()),
                InlineStorage::from_fn(|_| Default::default()),
            ),
        }
    }

    /// constructs a new handle to this collection
    pub fn buy_in(&self) -> InlineBanditHandle<'_, Q, S, N, SUB_CAP> {
        self.raw.buy_in()
    }
}

impl<Q, S, const N: usize, const SUB_CAP: usize> Default for InlineBandit<Q, S, N, SUB_CAP>
where
    Q: WithCapacity<SUB_CAP>,
    S: Strategy<Q> + Default,
    Q: Collection,
{
    fn default() -> Self {
        Self::new()
    }
}
