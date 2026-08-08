use core::ops::{Index, IndexMut};

#[cfg(feature = "alloc")]
use crate::storage::GrowingBackend;
use crate::{
    NewSized,
    construction::{DEFAULT_QUEUE_CAP, LopeCore, LopeCoreArm},
    schedule::{Hooked, Schedule},
    storage::StorageBackend,
};

#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) struct BoxedStorage<T> {
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
    pub(crate) fn from_fn_and_size(f: impl Fn(usize) -> T, size: usize) -> Self {
        Self {
            arr: (0..size).map(f).collect(),
        }
    }
}

impl<T> StorageBackend<T> for BoxedStorage<T> {
    fn len(&self) -> usize {
        self.arr.count()
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a,
    {
        self.arr.iter().map(|(_, item)| item)
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

#[allow(type_alias_bounds, private_interfaces)]
pub type BoxedArm<'a, Q, S: Schedule<Q>, const SUB_CAP: usize = DEFAULT_QUEUE_CAP> =
    LopeCoreArm<'a, Q, S, BoxedStorage<Q>, BoxedStorage<<S::Arm as Hooked>::State>, SUB_CAP>;

pub struct BoxedLope<Q, S: Schedule<Q>, const SUB_CAP: usize = DEFAULT_QUEUE_CAP> {
    raw: LopeCore<Q, S, BoxedStorage<Q>, BoxedStorage<<S::Arm as Hooked>::State>, SUB_CAP>,
}

impl<Q, S, const SUB_CAP: usize> BoxedLope<Q, S, SUB_CAP>
where
    Q: NewSized<SUB_CAP>,
    S: Schedule<Q> + Default,
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

    #[allow(private_interfaces)]
    pub fn new_root(&self) -> BoxedArm<'_, Q, S, SUB_CAP> {
        self.raw.new_root()
    }

    pub fn add_queue(&self) {
        self.raw.add_queue();
    }
}
