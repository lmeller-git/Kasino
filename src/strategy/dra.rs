use crossbeam_utils::CachePadded;
use rand::{RngExt, SeedableRng, rngs::SmallRng};

use crate::{
    Collection,
    storage::StorageBackend,
    strategy::{EDCount, Hooked, InstrumentedState, Strategy},
    sync::atomic::Ordering,
};

/// A DRA scheduler
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct DRA<const CHOOSE: usize = 2>;

#[allow(unnameable_types)]
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, Hash)]
pub struct DRAArm<R> {
    rng: R,
}

impl<Q: Collection, const CHOOSE: usize> Strategy<Q> for DRA<CHOOSE> {
    type Gambler = DRAArm<SmallRng>;

    fn choose_offer_arm(
        &self,
        state: &impl StorageBackend<<Self::Gambler as Hooked>::Stake>,
        arm: &mut Self::Gambler,
    ) -> usize {
        (0..CHOOSE)
            .map(|_| arm.rng.random_range(..state.len()))
            .min_by_key(|&i| {
                state[i]
                    .enq
                    .load(Ordering::Relaxed)
                    .saturating_sub(state[i].deq.load(Ordering::Relaxed))
            })
            .unwrap()
    }

    fn choose_poll_arm(
        &self,
        state: &impl StorageBackend<<Self::Gambler as Hooked>::Stake>,
        arm: &mut Self::Gambler,
    ) -> usize {
        (0..CHOOSE)
            .map(|_| arm.rng.random_range(..state.len()))
            .max_by_key(|&i| {
                state[i]
                    .deq
                    .load(Ordering::Relaxed)
                    .saturating_sub(state[i].enq.load(Ordering::Relaxed))
            })
            .unwrap()
    }

    fn fork_gambler(&self, arm: &mut Self::Gambler) -> Self::Gambler {
        DRAArm {
            rng: arm.rng.fork(),
        }
    }

    fn create_gambler(&self) -> Self::Gambler {
        DRAArm {
            rng: SmallRng::seed_from_u64(42),
        }
    }
}

impl Hooked for DRAArm<SmallRng> {
    type Stake = CachePadded<InstrumentedState<EDCount>>;
}
