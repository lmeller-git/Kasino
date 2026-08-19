use crate::{
    Collection,
    storage::StorageBackend,
    strategy::{Hooked, InstrumentedState, NoPad, Strategy},
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

impl<Q: Collection> Strategy<Q> for RoundRobin {
    type Gambler = RRArm;

    fn choose_offer_arm(
        &self,
        state: &impl StorageBackend<<Self::Gambler as Hooked>::Stake>,
        arm: &mut Self::Gambler,
    ) -> usize {
        arm.fetch_add() % state.len()
    }

    fn choose_poll_arm(
        &self,
        state: &impl StorageBackend<<Self::Gambler as Hooked>::Stake>,
        arm: &mut Self::Gambler,
    ) -> usize {
        arm.fetch_add() % state.len()
    }

    fn fork_gambler(&self, arm: &mut Self::Gambler) -> Self::Gambler {
        *arm
    }

    fn create_gambler(&self) -> Self::Gambler {
        RRArm::default()
    }
}

impl Hooked for RRArm {
    type Stake = NoPad<InstrumentedState<()>>;
}
