use core::marker::PhantomData;

use crossbeam_utils::CachePadded;
use rand::{RngExt, SeedableRng, rngs::SmallRng};

use crate::{
    Collection,
    storage::StorageBackend,
    strategy::{EDCount, Hooked, InstrumentedState, Strategy},
    sync::atomic::Ordering,
};

/// A DCBO scheduler
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct DCBO<R = SmallRng, const CHOOSE: usize = 2>(PhantomData<R>);

impl<R, const CHOOSE: usize> Default for DCBO<R, CHOOSE> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[allow(unnameable_types)]
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, Hash)]
pub struct DCBOGambler<R = SmallRng> {
    rng: R,
}

impl<R: RngExt + SeedableRng, Q: Collection, const CHOOSE: usize> Strategy<Q> for DCBO<R, CHOOSE> {
    type Gambler = DCBOGambler<R>;

    fn choose_offer_arm(
        &self,
        state: &impl StorageBackend<<Self::Gambler as Hooked>::Stake>,
        gambler: &mut Self::Gambler,
    ) -> usize {
        (0..CHOOSE)
            .map(|_| gambler.rng.random_range(..state.len()))
            .min_by_key(|&i| state[i].offer_count.load(Ordering::Relaxed))
            .unwrap()
    }

    fn choose_poll_arm(
        &self,
        state: &impl StorageBackend<<Self::Gambler as Hooked>::Stake>,
        gambler: &mut Self::Gambler,
    ) -> usize {
        (0..CHOOSE)
            .map(|_| gambler.rng.random_range(..state.len()))
            .min_by_key(|&i| state[i].poll_count.load(Ordering::Relaxed))
            .unwrap()
    }

    fn fork_gambler(&self, gambler: &mut Self::Gambler) -> Self::Gambler {
        DCBOGambler {
            rng: gambler.rng.fork(),
        }
    }

    fn create_gambler(&self) -> Self::Gambler {
        DCBOGambler {
            rng: R::seed_from_u64(Default::default()),
        }
    }
}

impl<R> Hooked for DCBOGambler<R> {
    type Stake = CachePadded<InstrumentedState<EDCount>>;
}
