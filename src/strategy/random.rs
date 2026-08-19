use rand::{RngExt, SeedableRng, rngs::SmallRng};

use crate::{
    Collection,
    storage::StorageBackend,
    strategy::{Hooked, InstrumentedState, NoPad, Strategy},
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

impl<Q: Collection> Strategy<Q> for RandomAccess<SmallRng> {
    type Gambler = Self;

    fn choose_offer_arm(
        &self,
        state: &impl StorageBackend<<Self::Gambler as Hooked>::Stake>,
        arm: &mut Self::Gambler,
    ) -> usize {
        arm.rng.random_range(..state.len())
    }

    fn choose_poll_arm(
        &self,
        state: &impl StorageBackend<<Self::Gambler as Hooked>::Stake>,
        arm: &mut Self::Gambler,
    ) -> usize {
        arm.rng.random_range(..state.len())
    }

    fn fork_gambler(&self, arm: &mut Self::Gambler) -> Self::Gambler {
        Self {
            rng: arm.rng.fork(),
        }
    }

    fn create_gambler(&self) -> Self::Gambler {
        Self {
            rng: SmallRng::seed_from_u64(42),
        }
    }
}

impl Hooked for RandomAccess<SmallRng> {
    type Stake = NoPad<InstrumentedState<()>>;
}
