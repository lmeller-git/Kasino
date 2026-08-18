use crate::{
    schedule::{Hooked, InstrumentedState, NoPad, Schedule},
    storage::StorageBackend,
};

/// a round robin scheduler
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, Hash)]
pub struct RoundRobin;

#[allow(unnameable_types)]
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, Hash)]
pub struct RRArm {
    cur: usize,
}

impl RRArm {
    fn fetch_add(&mut self) -> usize {
        let n = self.cur;
        self.cur += 1;
        n
    }
}

impl Schedule for RoundRobin {
    type Arm = RRArm;

    fn choose_offer_shard(
        &self,
        state: &impl StorageBackend<<Self::Arm as Hooked>::State>,
        arm: &mut Self::Arm,
    ) -> usize {
        arm.fetch_add() % state.len()
    }

    fn choose_poll_shard(
        &self,
        state: &impl StorageBackend<<Self::Arm as Hooked>::State>,
        arm: &mut Self::Arm,
    ) -> usize {
        arm.fetch_add() % state.len()
    }

    fn fork_arm(&self, arm: &mut Self::Arm) -> Self::Arm {
        *arm
    }

    fn create_arm(&self) -> Self::Arm {
        RRArm::default()
    }
}

impl Hooked for RRArm {
    type State = NoPad<InstrumentedState<()>>;
}
