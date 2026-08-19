use core::{
    marker::PhantomData,
    ops::{Deref, Index},
};

use crossbeam_utils::CachePadded;

use crate::{
    Collection,
    Signature,
    storage::StorageBackend,
    strategy::{Hook, Hooked, Strategy},
    sync::atomic::{AtomicUsize, Ordering},
};

/// Does not collect any remainign items
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct NoCollect<S>(S);

impl<Q: Collection, S: Strategy<Q>> Strategy<Q> for NoCollect<S> {
    type Gambler = S::Gambler;

    fn choose_offer_arm(
        &self,
        state: &impl StorageBackend<<Self::Gambler as Hooked>::Stake>,
        gambler: &mut Self::Gambler,
    ) -> usize {
        self.0.choose_offer_arm(state, gambler)
    }

    fn choose_poll_arm(
        &self,
        choose_to: &impl StorageBackend<<Self::Gambler as Hooked>::Stake>,
        gambler: &mut Self::Gambler,
    ) -> usize {
        self.0.choose_poll_arm(choose_to, gambler)
    }

    fn fork_gambler(&self, gambler: &mut Self::Gambler) -> Self::Gambler {
        self.0.fork_gambler(gambler)
    }

    fn create_gambler(&self) -> Self::Gambler {
        self.0.create_gambler()
    }

    fn collect<'b, 'c>(
        &self,
        _state: &impl StorageBackend<<Self::Gambler as Hooked>::Stake>,
        _sub_collections: &'c impl StorageBackend<Q>,
        _input: <Q::PollSignature as Signature>::Input<'b>,
    ) -> Option<(<Q::PollSignature as Signature>::Output<'b, 'c>, usize)>
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
    backend: &'a B,
    _phantom: PhantomData<(&'a T, &'a K)>,
}

impl<'a, B, T, K> StorageView<'a, B, T, K> {
    fn new(backend: &'a B) -> Self {
        Self {
            backend,
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
        self.backend[index].project()
    }
}

impl<'b, B, T, K> StorageBackend<T> for StorageView<'b, B, T, K>
where
    B: StorageBackend<K>,
    K: View<'b, T>,
{
    type Rebind<U> = B::Rebind<U>;

    fn len(&self) -> usize {
        self.backend.len()
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a,
    {
        self.backend.iter().map(|i| i.project())
    }

    fn is_empty(&self) -> bool {
        self.backend.is_empty()
    }

    fn map_to_buffer<U>(&self, f: impl Fn(usize) -> U) -> Self::Rebind<U> {
        self.backend.map_to_buffer(f)
    }
}

/// Runs a double collect on a dequeu. This strategy promises empty-linearizability
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct DoubleCollect<S>(S);

#[allow(unnameable_types)]
pub struct DoubleCollectGambler<A> {
    gambler: A,
}

#[allow(unnameable_types)]
#[derive(Default, Debug)]
pub struct DoubleCollectState<S> {
    strategy: S,
    epoch: AtomicUsize,
}

impl<'a, S> View<'a, S> for DoubleCollectState<S> {
    fn project(&'a self) -> &'a S {
        &self.strategy
    }
}

impl<A: Hooked> Hooked for DoubleCollectGambler<A> {
    type Stake = CachePadded<DoubleCollectState<A::Stake>>;
}

impl<T: Hook> Hook for DoubleCollectState<T> {
    fn on_offer_succ(&self) {
        self.strategy.on_offer_succ();
    }

    fn on_poll_succ(&self) {
        self.strategy.on_poll_succ();
    }
}

impl<S: Strategy<Q>, Q: Collection> Strategy<Q> for DoubleCollect<S> {
    type Gambler = DoubleCollectGambler<S::Gambler>;

    fn choose_offer_arm(
        &self,
        state: &impl StorageBackend<<Self::Gambler as Hooked>::Stake>,
        gambler: &mut Self::Gambler,
    ) -> usize {
        let idx = self
            .0
            .choose_offer_arm(&StorageView::new(state), &mut gambler.gambler);
        state[idx].epoch.fetch_add(1, Ordering::Release);
        idx
    }

    fn choose_poll_arm(
        &self,
        choose_to: &impl StorageBackend<<Self::Gambler as Hooked>::Stake>,
        gambler: &mut Self::Gambler,
    ) -> usize {
        self.0
            .choose_poll_arm(&StorageView::new(choose_to), &mut gambler.gambler)
    }

    fn fork_gambler(&self, gambler: &mut Self::Gambler) -> Self::Gambler {
        DoubleCollectGambler {
            gambler: self.0.fork_gambler(&mut gambler.gambler),
        }
    }

    fn create_gambler(&self) -> Self::Gambler {
        DoubleCollectGambler {
            gambler: self.0.create_gambler(),
        }
    }

    fn collect<'b, 'c>(
        &self,
        state: &impl StorageBackend<<Self::Gambler as Hooked>::Stake>,
        sub_collections: &'c impl StorageBackend<Q>,
        input: <Q::PollSignature as Signature>::Input<'b>,
    ) -> Option<(<Q::PollSignature as Signature>::Output<'b, 'c>, usize)>
    where
        Q: 'c,
    {
        let mut versions = state.map_to_buffer(|_| None);

        'collect: loop {
            for (i, item) in state.iter().enumerate() {
                let epoch = item.epoch.load(Ordering::Acquire);
                if let Ok(item) = sub_collections[i].poll(input) {
                    return Some((item, i));
                }
                versions[i].replace(epoch);
            }

            for (stored_epoch, item) in versions.iter().zip(state.iter()) {
                let epoch = item.epoch.load(Ordering::Acquire);
                if stored_epoch.is_some_and(|e| e < epoch) {
                    continue 'collect;
                }
            }

            return None;
        }
    }
}
