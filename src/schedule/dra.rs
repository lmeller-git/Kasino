use portable_atomic::AtomicUsize;
use rand::{RngExt, SeedableRng, rngs::SmallRng};

use crate::{
    schedule::{Hooked, Schedule},
    storage::StorageBackend,
    sync::atomic::Ordering,
};

#[derive(Default)]
pub struct DRAState {
    enq: AtomicUsize,
    deq: AtomicUsize,
}

pub struct DRA<const CHOOSE: usize = 2> {}

impl<const CHOOSE: usize> Default for DRA<CHOOSE> {
    fn default() -> Self {
        Self {}
    }
}

pub struct DRAArm<R> {
    rng: R,
}

impl<T, const CHOOSE: usize> Schedule<T> for DRA<CHOOSE> {
    type Arm = DRAArm<SmallRng>;

    fn choose_enq(
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

    fn choose_deq(
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
    type State = DRAState;

    fn on_enq(&mut self, sub_state: &Self::State) {
        sub_state.enq.fetch_add(1, Ordering::Release);
    }

    fn on_deq(&mut self, sub_state: &Self::State) {
        sub_state.deq.fetch_add(1, Ordering::Release);
    }
}
