use crate::{
    Collection,
    storage::StorageBackend,
    strategy::{Hooked, InstrumentedState, NoPad, Strategy},
};

/// a round robin scheduler
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, Hash)]
pub struct RoundRobin;

#[expect(unnameable_types)]
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, Hash)]
pub struct RoundRobinGambler {
    cur: usize,
}

impl RoundRobinGambler {
    fn fetch_add(&mut self) -> usize {
        let n = self.cur;
        self.cur += 1;
        n
    }
}

impl<Q: Collection> Strategy<Q> for RoundRobin {
    type Gambler = RoundRobinGambler;

    #[inline]
    fn choose_offer_arm(
        &self,
        state: &impl StorageBackend<<Self::Gambler as Hooked>::Stake>,
        gambler: &mut Self::Gambler,
    ) -> usize {
        gambler.fetch_add() % state.len()
    }

    #[inline]
    fn choose_poll_arm(
        &self,
        state: &impl StorageBackend<<Self::Gambler as Hooked>::Stake>,
        gambler: &mut Self::Gambler,
    ) -> usize {
        gambler.fetch_add() % state.len()
    }

    #[inline]
    fn fork_gambler(&self, gambler: &mut Self::Gambler) -> Self::Gambler {
        *gambler
    }

    #[inline]
    fn create_gambler(&self) -> Self::Gambler {
        RoundRobinGambler::default()
    }
}

impl Hooked for RoundRobinGambler {
    type Stake = NoPad<InstrumentedState<()>>;
}
