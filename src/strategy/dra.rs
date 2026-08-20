use core::marker::PhantomData;

use crossbeam_utils::CachePadded;
use rand::{RngExt, SeedableRng, rngs::SmallRng};

use crate::{
    Collection,
    storage::StorageBackend,
    strategy::{EDCount, Hooked, Strategy},
    sync::atomic::Ordering,
};

/// A DRA scheduler
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct DRA<const CHOOSE: usize = 2, R = SmallRng>(PhantomData<R>);

impl<R, const CHOOSE: usize> Default for DRA<CHOOSE, R> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[expect(unnameable_types)]
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, Hash)]
pub struct DRAGambler<R = SmallRng> {
    rng: R,
}

impl<R: RngExt + SeedableRng, Q: Collection, const CHOOSE: usize> Strategy<Q> for DRA<CHOOSE, R> {
    type Gambler = DRAGambler<R>;

    fn choose_offer_arm(
        &self,
        state: &impl StorageBackend<<Self::Gambler as Hooked>::Stake>,
        gambler: &mut Self::Gambler,
    ) -> usize {
        (0..CHOOSE)
            .map(|_| gambler.rng.random_range(..state.len()))
            .min_by_key(|&i| {
                state[i]
                    .offer_count
                    .load(Ordering::Relaxed)
                    .saturating_sub(state[i].poll_count())
            })
            .unwrap()
    }

    fn choose_poll_arm(
        &self,
        state: &impl StorageBackend<<Self::Gambler as Hooked>::Stake>,
        gambler: &mut Self::Gambler,
    ) -> usize {
        (0..CHOOSE)
            .map(|_| gambler.rng.random_range(..state.len()))
            .max_by_key(|&i| {
                state[i]
                    .poll_count
                    .load(Ordering::Relaxed)
                    .saturating_sub(state[i].offer_count())
            })
            .unwrap()
    }

    fn fork_gambler(&self, gambler: &mut Self::Gambler) -> Self::Gambler {
        DRAGambler {
            rng: gambler.rng.fork(),
        }
    }

    fn create_gambler(&self) -> Self::Gambler {
        DRAGambler {
            rng: R::seed_from_u64(Default::default()),
        }
    }
}

impl<R> Hooked for DRAGambler<R> {
    type Stake = CachePadded<EDCount>;
}
