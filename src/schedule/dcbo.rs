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

pub struct DCBO<B, const CHOOSE: usize = 2> {
    state: B,
}

impl<B: Default, const CHOOSE: usize> Default for DCBO<B, CHOOSE> {
    fn default() -> Self {
        Self {
            state: Default::default(),
        }
    }
}

pub struct DCBOARM<'a, B, R, const CHOOSE: usize = 2> {
    parent: &'a DCBO<B, CHOOSE>,
    rng: R,
}

impl<B: Default + StorageBackend<State>, T, const CHOOSE: usize> Schedule<T> for DCBO<B, CHOOSE> {
    type Arm<'a>
        = DCBOARM<'a, B, SmallRng, CHOOSE>
    where
        Self: 'a;
    type State = State;

    fn choose_enq(&self, _choose_to: usize, arm: &mut Self::Arm<'_>) -> usize {
        (0..CHOOSE)
            .map(|_| arm.rng.random_range(..self.state.len()))
            .min_by_key(|&i| self.state.as_slice()[i].enq.load(Ordering::Relaxed))
            .unwrap()
    }

    fn choose_deq(&self, _choose_to: usize, arm: &mut Self::Arm<'_>) -> usize {
        (0..CHOOSE)
            .map(|_| arm.rng.random_range(..self.state.len()))
            .max_by_key(|&i| self.state.as_slice()[i].deq.load(Ordering::Relaxed))
            .unwrap()
    }

    fn fork_arm(&self, arm: &mut Self::Arm<'_>) -> Self::Arm<'_> {
        DCBOARM {
            parent: self,
            rng: arm.rng.fork(),
        }
    }

    fn create_arm(&self) -> Self::Arm<'_> {
        DCBOARM {
            rng: SmallRng::seed_from_u64(42),
            parent: self,
        }
    }
}

impl<'a, B: StorageBackend<State>, const CHOOSE: usize> Hooked
    for DCBOARM<'a, B, SmallRng, CHOOSE>
{
    fn on_enq(&mut self, choice: usize) {
        self.parent.state.as_slice()[choice]
            .enq
            .fetch_add(1, Ordering::Relaxed);
    }

    fn on_deq(&mut self, choice: usize) {
        self.parent.state.as_slice()[choice]
            .deq
            .fetch_add(1, Ordering::Relaxed);
    }
}
