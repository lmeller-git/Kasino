use portable_atomic::AtomicUsize;
use rand::{RngExt, SeedableRng, rngs::SmallRng};

use crate::{
    schedule::{Hooked, Schedule},
    storage::StorageBackend,
    sync::atomic::Ordering,
};

#[allow(unnameable_types)]
#[derive(Default, Debug)]
pub struct DCBOState {
    enq: AtomicUsize,
    deq: AtomicUsize,
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct DCBO<const CHOOSE: usize = 2>;

#[allow(unnameable_types)]
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, Hash)]
pub struct DCBOArm<R> {
    rng: R,
}

impl<T, const CHOOSE: usize> Schedule<T> for DCBO<CHOOSE> {
    type Arm = DCBOArm<SmallRng>;

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
        DCBOArm {
            rng: arm.rng.fork(),
        }
    }

    fn create_arm(&self) -> Self::Arm {
        DCBOArm {
            rng: SmallRng::seed_from_u64(42),
        }
    }
}

impl Hooked for DCBOArm<SmallRng> {
    type State = DCBOState;

    fn on_enq(&mut self, sub_state: &Self::State) {
        sub_state.enq.fetch_add(1, Ordering::Relaxed);
    }

    fn on_deq(&mut self, sub_state: &Self::State) {
        sub_state.deq.fetch_add(1, Ordering::Relaxed);
    }
}
