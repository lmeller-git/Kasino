use portable_atomic::AtomicUsize;
use rand::{RngExt, SeedableRng, rngs::SmallRng};

use crate::{
    schedule::{Hooked, Schedule},
    storage::StorageBackend,
    sync::atomic::Ordering,
};

#[derive(Default)]
pub struct State {
    enq: AtomicUsize,
    deq: AtomicUsize,
}

pub struct DRA<B> {
    state: B,
}

impl<B: Default> Default for DRA<B> {
    fn default() -> Self {
        Self {
            state: Default::default(),
        }
    }
}

pub struct DRAArm<'a, B, R> {
    parent: &'a DRA<B>,
    rng: R,
}

impl<B: Default + StorageBackend<State>, T> Schedule<T> for DRA<B> {
    type Arm<'a>
        = DRAArm<'a, B, SmallRng>
    where
        Self: 'a;
    type State = State;

    fn choose_enq(&self, _choose_to: usize, arm: &mut Self::Arm<'_>) -> usize {
        (0..2)
            .map(|_| arm.rng.random_range(..self.state.len()))
            .min_by_key(|&i| {
                self.state.as_slice()[i]
                    .enq
                    .load(Ordering::Relaxed)
                    .saturating_sub(self.state.as_slice()[i].deq.load(Ordering::Relaxed))
            })
            .unwrap()
    }

    fn choose_deq(&self, _choose_to: usize, arm: &mut Self::Arm<'_>) -> usize {
        (0..2)
            .map(|_| arm.rng.random_range(..self.state.len()))
            .max_by_key(|&i| {
                self.state.as_slice()[i]
                    .deq
                    .load(Ordering::Relaxed)
                    .saturating_sub(self.state.as_slice()[i].enq.load(Ordering::Relaxed))
            })
            .unwrap()
    }

    fn fork_arm(&self, arm: &mut Self::Arm<'_>) -> Self::Arm<'_> {
        DRAArm {
            parent: self,
            rng: arm.rng.fork(),
        }
    }

    fn create_arm(&self) -> Self::Arm<'_> {
        DRAArm {
            rng: SmallRng::seed_from_u64(42),
            parent: self,
        }
    }
}

impl<'a, B: StorageBackend<State>> Hooked for DRAArm<'a, B, SmallRng> {
    fn on_enq(&mut self, choice: usize) {
        self.parent.state.as_slice()[choice]
            .enq
            .fetch_add(1, Ordering::Release);
    }

    fn on_deq(&mut self, choice: usize) {
        self.parent.state.as_slice()[choice]
            .deq
            .fetch_add(1, Ordering::Release);
    }
}
