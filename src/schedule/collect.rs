use core::{
    marker::PhantomData,
    ops::{Deref, Index},
};

use crossbeam_utils::CachePadded;

use crate::{
    Collection,
    IODescription,
    schedule::{Hook, Hooked, Schedule},
    storage::StorageBackend,
    sync::atomic::{AtomicUsize, Ordering},
};

/// Does not collect any remainign items
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct NoCollect<S>(S);

impl<Q: Collection, S: Schedule<Q>> Schedule<Q> for NoCollect<S> {
    type Arm = S::Arm;

    fn choose_offer_shard(
        &self,
        state: &impl StorageBackend<<Self::Arm as Hooked>::State>,
        arm: &mut Self::Arm,
    ) -> usize {
        self.0.choose_offer_shard(state, arm)
    }

    fn choose_poll_shard(
        &self,
        choose_to: &impl StorageBackend<<Self::Arm as Hooked>::State>,
        arm: &mut Self::Arm,
    ) -> usize {
        self.0.choose_poll_shard(choose_to, arm)
    }

    fn fork_arm(&self, arm: &mut Self::Arm) -> Self::Arm {
        self.0.fork_arm(arm)
    }

    fn create_arm(&self) -> Self::Arm {
        self.0.create_arm()
    }

    fn collect<'b, 'c>(
        &self,
        _state: &impl StorageBackend<<Self::Arm as Hooked>::State>,
        _sub_collections: &'c impl StorageBackend<Q>,
        _input: <Q::PollIO as IODescription>::Input<'b>,
    ) -> Option<(<Q::PollIO as IODescription>::Output<'b, 'c>, usize)>
    where
        Q: 'c,
    {
        None
    }
}

#[allow(unreachable_pub)]
pub trait View<'a, T> {
    fn project(&'a self) -> &'a T;
}

impl<'a, K, U, T> View<'a, T> for K
where
    K: Deref<Target = U>,
    U: View<'a, T> + 'a,
{
    fn project(&'a self) -> &'a T {
        U::project(self)
    }
}

#[allow(unreachable_pub)]
pub struct StorageView<'a, B, T, K> {
    b: &'a B,
    _phantom: PhantomData<(&'a T, &'a K)>,
}

impl<'a, B, T, K> StorageView<'a, B, T, K> {
    fn new(b: &'a B) -> Self {
        Self {
            b,
            _phantom: PhantomData,
        }
    }
}

impl<'a, B: Index<usize>, T, K> Index<usize> for StorageView<'a, B, T, K>
where
    B::Output: View<'a, T>,
{
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.b[index].project()
    }
}

impl<'b, B, T, K> StorageBackend<T> for StorageView<'b, B, T, K>
where
    B: StorageBackend<K>,
    K: View<'b, T>,
{
    type Rebind<U> = B::Rebind<U>;

    fn len(&self) -> usize {
        self.b.len()
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a,
    {
        self.b.iter().map(|i| i.project())
    }

    fn is_empty(&self) -> bool {
        self.b.is_empty()
    }

    fn map_to_buffer<U>(&self, f: impl Fn(usize) -> U) -> Self::Rebind<U> {
        self.b.map_to_buffer(f)
    }
}

/// Runs a double collect on a dequeu. This strategy promises empty-linearizability
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct DoubleCollect<S>(S);

#[allow(unnameable_types)]
pub struct DoubleCollectArm<A> {
    a: A,
}

#[allow(unnameable_types)]
#[derive(Default, Debug)]
pub struct DoubleCollectState<S> {
    s: S,
    e: AtomicUsize,
}

impl<'a, S> View<'a, S> for DoubleCollectState<S> {
    fn project(&'a self) -> &'a S {
        &self.s
    }
}

impl<A: Hooked> Hooked for DoubleCollectArm<A> {
    type State = CachePadded<DoubleCollectState<A::State>>;
}

impl<T: Hook> Hook for DoubleCollectState<T> {
    fn on_offer_succ(&self) {
        self.s.on_offer_succ();
    }

    fn on_poll_succ(&self) {
        self.s.on_poll_succ();
    }
}

impl<S: Schedule<Q>, Q: Collection> Schedule<Q> for DoubleCollect<S> {
    type Arm = DoubleCollectArm<S::Arm>;

    fn choose_offer_shard(
        &self,
        state: &impl StorageBackend<<Self::Arm as Hooked>::State>,
        arm: &mut Self::Arm,
    ) -> usize {
        let idx = self
            .0
            .choose_offer_shard(&StorageView::new(state), &mut arm.a);
        state[idx].e.fetch_add(1, Ordering::Release);
        idx
    }

    fn choose_poll_shard(
        &self,
        choose_to: &impl StorageBackend<<Self::Arm as Hooked>::State>,
        arm: &mut Self::Arm,
    ) -> usize {
        self.0
            .choose_poll_shard(&StorageView::new(choose_to), &mut arm.a)
    }

    fn fork_arm(&self, arm: &mut Self::Arm) -> Self::Arm {
        DoubleCollectArm {
            a: self.0.fork_arm(&mut arm.a),
        }
    }

    fn create_arm(&self) -> Self::Arm {
        DoubleCollectArm {
            a: self.0.create_arm(),
        }
    }

    fn collect<'b, 'c>(
        &self,
        state: &impl StorageBackend<<Self::Arm as Hooked>::State>,
        sub_collections: &'c impl StorageBackend<Q>,
        input: <Q::PollIO as IODescription>::Input<'b>,
    ) -> Option<(<Q::PollIO as IODescription>::Output<'b, 'c>, usize)>
    where
        Q: 'c,
    {
        let mut versions = state.map_to_buffer(|_| None);

        'collect: loop {
            for (i, item) in state.iter().enumerate() {
                let epoch = item.e.load(Ordering::Acquire);
                if let Ok(item) = sub_collections[i].poll(input) {
                    return Some((item, i));
                }
                versions[i].replace(epoch);
            }

            for (stored_epoch, item) in versions.iter().zip(state.iter()) {
                let epoch = item.e.load(Ordering::Acquire);
                if stored_epoch.is_some_and(|e| e < epoch) {
                    continue 'collect;
                }
            }

            return None;
        }
    }
}
