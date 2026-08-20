use alloc::boxed::Box;
use core::ops::{Index, IndexMut};

use crate::{
    Collection,
    WithCapacity,
    construction::{BanditCore, BanditHandle, DEFAULT_QUEUE_CAP},
    storage::StorageBackend,
    strategy::{Hooked, Strategy},
};

/// a dynamicaly stored slice
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BoxedStorage<T> {
    arr: Box<[T]>,
}

impl<T: Default> Default for BoxedStorage<T> {
    fn default() -> Self {
        Self {
            arr: Default::default(),
        }
    }
}

impl<T> BoxedStorage<T> {
    pub(crate) fn from_fn_and_size(f: impl Fn(usize) -> T, size: usize) -> Self {
        Self {
            arr: (0..size).map(f).collect(),
        }
    }
}

impl<T> StorageBackend<T> for BoxedStorage<T> {
    type Rebind<U> = BoxedStorage<U>;

    fn len(&self) -> usize {
        self.arr.len()
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a,
    {
        self.arr.iter()
    }

    fn map_to_buffer<K>(&self, f: impl Fn(usize) -> K) -> Self::Rebind<K> {
        BoxedStorage::from_fn_and_size(f, self.arr.len())
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

/// a handle to the core subcollection container, which is stored dynamically
#[expect(type_alias_bounds)]
pub type BoxedBanditHandle<
    'a,
    Q: Collection,
    S: Strategy<Q>,
    const SUB_CAP: usize = DEFAULT_QUEUE_CAP,
> = BanditHandle<'a, Q, S, BoxedStorage<Q>, BoxedStorage<<S::Gambler as Hooked>::Stake>, SUB_CAP>;

/// a subcollection container, which is stored dynamically
pub struct BoxedBandit<Q: Collection, S: Strategy<Q>, const SUB_CAP: usize = DEFAULT_QUEUE_CAP> {
    raw: BanditCore<Q, S, BoxedStorage<Q>, BoxedStorage<<S::Gambler as Hooked>::Stake>, SUB_CAP>,
}

impl<Q, S, const SUB_CAP: usize> BoxedBandit<Q, S, SUB_CAP>
where
    Q: WithCapacity<SUB_CAP>,
    S: Strategy<Q> + Default,
    Q: Collection,
{
    /// constructs a new `BoxedLop`
    #[must_use]
    pub fn new(n_cores: usize) -> Self {
        Self {
            raw: BanditCore::new_with(
                BoxedStorage::from_fn_and_size(
                    |_| <Q as WithCapacity<SUB_CAP>>::with_capacity(),
                    n_cores,
                ),
                BoxedStorage::from_fn_and_size(|_| Default::default(), n_cores),
            ),
        }
    }

    /// constructs a new handle to this container
    pub fn buy_in(&self) -> BoxedBanditHandle<'_, Q, S, SUB_CAP> {
        self.raw.buy_in()
    }
}
