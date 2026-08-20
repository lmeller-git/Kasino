use core::marker::PhantomData;

use crossbeam_utils::CachePadded;
use rand::{RngExt, SeedableRng, rngs::SmallRng};

use crate::{
    Collection,
    storage::StorageBackend,
    strategy::{EDCount, Hooked, Strategy},
};

/// A DCBO scheduler
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct DCBO<const CHOOSE: usize = 2, R = SmallRng>(PhantomData<R>);

impl<R, const CHOOSE: usize> Default for DCBO<CHOOSE, R> {
    #[inline]
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[expect(unnameable_types)]
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, Hash)]
pub struct DCBOGambler<R = SmallRng> {
    rng: R,
}

impl<R: RngExt + SeedableRng, Q: Collection, const CHOOSE: usize> Strategy<Q> for DCBO<CHOOSE, R> {
    type Gambler = DCBOGambler<R>;

    #[inline]
    fn choose_offer_arm(
        &self,
        state: &impl StorageBackend<<Self::Gambler as Hooked>::Stake>,
        gambler: &mut Self::Gambler,
    ) -> usize {
        (0..CHOOSE)
            .map(|_| gambler.rng.random_range(..state.len()))
            .min_by_key(|&i| state[i].offer_count())
            .unwrap()
    }

    #[inline]
    fn choose_poll_arm(
        &self,
        state: &impl StorageBackend<<Self::Gambler as Hooked>::Stake>,
        gambler: &mut Self::Gambler,
    ) -> usize {
        (0..CHOOSE)
            .map(|_| gambler.rng.random_range(..state.len()))
            .min_by_key(|&i| state[i].poll_count())
            .unwrap()
    }

    #[inline]
    fn fork_gambler(&self, gambler: &mut Self::Gambler) -> Self::Gambler {
        DCBOGambler {
            rng: gambler.rng.fork(),
        }
    }

    #[inline]
    fn create_gambler(&self) -> Self::Gambler {
        DCBOGambler {
            rng: R::seed_from_u64(Default::default()),
        }
    }
}

impl<R> Hooked for DCBOGambler<R> {
    type Stake = CachePadded<EDCount>;
}
