use rand::{RngExt, SeedableRng, rngs::SmallRng};

use crate::schedule::{Hooked, Schedule};

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
    type Arm<'a> = Self;
    type State = ();

    fn choose_enq(&self, choose_to: usize, arm: &mut Self::Arm<'_>) -> usize {
        arm.rng.random_range(..choose_to)
    }

    fn choose_deq(&self, choose_to: usize, arm: &mut Self::Arm<'_>) -> usize {
        arm.rng.random_range(..choose_to)
    }

    fn fork_arm(&self, arm: &mut Self::Arm<'_>) -> Self::Arm<'_> {
        Self {
            rng: arm.rng.fork(),
        }
    }

    fn create_arm(&self) -> Self::Arm<'_> {
        Self {
            rng: SmallRng::seed_from_u64(42),
        }
    }
}

impl Hooked for RandomAccess<SmallRng> {}
