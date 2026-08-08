use rand::{RngExt, SeedableRng, rngs::SmallRng};

use crate::{
    schedule::{Hooked, Schedule},
    storage::StorageBackend,
};

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

impl<T> Schedule<T> for RandomAccess<SmallRng> {
    type Arm = Self;

    fn choose_enq(
        &self,
        state: &impl StorageBackend<<Self::Arm as Hooked>::State>,
        arm: &mut Self::Arm,
    ) -> usize {
        arm.rng.random_range(..state.len())
    }

    fn choose_deq(
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
    type State = ();
}
