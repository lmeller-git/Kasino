use rand::{RngExt, SeedableRng, rngs::SmallRng};

use crate::{
    Collection,
    storage::StorageBackend,
    strategy::{Hooked, InstrumentedState, NoPad, Strategy},
};

/// a random scheduler
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RandomAccess<R = SmallRng> {
    rng: R,
}

impl<S> Default for RandomAccess<S>
where
    S: SeedableRng,
{
    fn default() -> Self {
        Self {
            rng: S::seed_from_u64(Default::default()),
        }
    }
}

impl<Q: Collection, S: RngExt + SeedableRng> Strategy<Q> for RandomAccess<S> {
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
            rng: S::seed_from_u64(42),
        }
    }
}

impl<S> Hooked for RandomAccess<S> {
    type Stake = NoPad<InstrumentedState<()>>;
}
