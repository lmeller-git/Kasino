use crate::schedule::{Hooked, Schedule};

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
    type Arm<'a>
        = RRArm
    where
        Self: 'a;
    type State = ();

    fn choose_enq(&self, choose_to: usize, arm: &mut Self::Arm<'_>) -> usize {
        arm.fetch_add() % choose_to
    }

    fn choose_deq(&self, choose_to: usize, arm: &mut Self::Arm<'_>) -> usize {
        arm.fetch_add() % choose_to
    }

    fn fork_arm(&self, arm: &mut Self::Arm<'_>) -> Self::Arm<'_> {
        *arm
    }

    fn create_arm(&self) -> Self::Arm<'_> {
        RRArm::default()
    }
}

impl Hooked for RRArm {}
