use crossbeam_utils::CachePadded;
use rand::{RngExt, SeedableRng, rngs::SmallRng};

use crate::{
    schedule::{EDCount, Hooked, InstrumentedState, Schedule},
    storage::StorageBackend,
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

impl<const CHOOSE: usize> Schedule for DRA<CHOOSE> {
    type Arm = DRAArm<SmallRng>;

    fn choose_offer_shard(
        &self,
        state: &impl StorageBackend<<Self::Arm as Hooked>::State>,
        arm: &mut Self::Arm,
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

    fn choose_poll_shard(
        &self,
        state: &impl StorageBackend<<Self::Arm as Hooked>::State>,
        arm: &mut Self::Arm,
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

    fn fork_arm(&self, arm: &mut Self::Arm) -> Self::Arm {
        DRAArm {
            rng: arm.rng.fork(),
        }
    }

    fn create_arm(&self) -> Self::Arm {
        DRAArm {
            rng: SmallRng::seed_from_u64(42),
        }
    }
}

impl Hooked for DRAArm<SmallRng> {
    type State = CachePadded<InstrumentedState<EDCount>>;
}
