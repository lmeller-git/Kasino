use portable_atomic::AtomicUsize;
use rand::{RngExt, SeedableRng, rngs::SmallRng};

use crate::{
    schedule::{Hooked, Schedule},
    storage::StorageBackend,
    sync::atomic::Ordering,
};

#[derive(Default)]
pub struct DCBOState {
    enq: AtomicUsize,
    deq: AtomicUsize,
}

pub struct DCBO<const CHOOSE: usize = 2> {}

impl<const CHOOSE: usize> Default for DCBO<CHOOSE> {
    fn default() -> Self {
        Self {}
    }
}

pub struct DCBOARM<R> {
    rng: R,
}

impl<T, const CHOOSE: usize> Schedule<T> for DCBO<CHOOSE> {
    type Arm = DCBOARM<SmallRng>;

    fn choose_enq(
        &self,
        state: &impl StorageBackend<<Self::Arm as Hooked>::State>,
        arm: &mut Self::Arm,
    ) -> usize {
        (0..CHOOSE)
            .map(|_| arm.rng.random_range(..state.len()))
            .min_by_key(|&i| state[i].enq.load(Ordering::Relaxed))
            .unwrap()
    }

    fn choose_deq(
        &self,
        state: &impl StorageBackend<<Self::Arm as Hooked>::State>,
        arm: &mut Self::Arm,
    ) -> usize {
        (0..CHOOSE)
            .map(|_| arm.rng.random_range(..state.len()))
            .max_by_key(|&i| state[i].deq.load(Ordering::Relaxed))
            .unwrap()
    }

    fn fork_arm(&self, arm: &mut Self::Arm) -> Self::Arm {
        DCBOARM {
            rng: arm.rng.fork(),
        }
    }

    fn create_arm(&self) -> Self::Arm {
        DCBOARM {
            rng: SmallRng::seed_from_u64(42),
        }
    }
}

impl Hooked for DCBOARM<SmallRng> {
    type State = DCBOState;

    fn on_enq(&mut self, sub_state: &Self::State) {
        sub_state.enq.fetch_add(1, Ordering::Relaxed);
    }

    fn on_deq(&mut self, sub_state: &Self::State) {
        sub_state.deq.fetch_add(1, Ordering::Relaxed);
    }
}
