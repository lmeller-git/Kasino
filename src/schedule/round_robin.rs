use crate::{
    schedule::{Hooked, Schedule},
    storage::StorageBackend,
};

#[derive(Default)]
pub struct RoundRobin {}

#[derive(Debug, Default, Clone, Copy)]
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

impl<T> Schedule<T> for RoundRobin {
    type Arm = RRArm;

    fn choose_enq(
        &self,
        state: &impl StorageBackend<<Self::Arm as Hooked>::State>,
        arm: &mut Self::Arm,
    ) -> usize {
        arm.fetch_add() % state.len()
    }

    fn choose_deq(
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
    type State = ();
}
