use rand::{RngExt, SeedableRng, rngs::SmallRng};

use crate::{
    Collection,
    schedule::{Hooked, InstrumentedState, NoPad, Schedule},
    storage::StorageBackend,
};

/// a random scheduler
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RandomAccess<R> {
    rng: R,
}

impl Default for RandomAccess<SmallRng> {
    fn default() -> Self {
        Self {
            rng: SmallRng::seed_from_u64(42),
        }
    }
}

impl<Q: Collection> Schedule<Q> for RandomAccess<SmallRng> {
    type Arm = Self;

    fn choose_offer_shard(
        &self,
        state: &impl StorageBackend<<Self::Arm as Hooked>::State>,
        arm: &mut Self::Arm,
    ) -> usize {
        arm.rng.random_range(..state.len())
    }

    fn choose_poll_shard(
        &self,
        state: &impl StorageBackend<<Self::Arm as Hooked>::State>,
        arm: &mut Self::Arm,
    ) -> usize {
        arm.rng.random_range(..state.len())
    }

    fn fork_arm(&self, arm: &mut Self::Arm) -> Self::Arm {
        Self {
            rng: arm.rng.fork(),
        }
    }

    fn create_arm(&self) -> Self::Arm {
        Self {
            rng: SmallRng::seed_from_u64(42),
        }
    }
}

impl Hooked for RandomAccess<SmallRng> {
    type State = NoPad<InstrumentedState<()>>;
}
