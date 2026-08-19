use core::marker::PhantomData;

use crate::{
    Collection,
    Signature,
    storage::StorageBackend,
    strategy::{Hooked, Strategy},
};

pub(crate) const DEFAULT_QUEUE_CAP: usize = 32;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash)]
pub(crate) struct BanditCore<Q, S, B, C, const SUB_CAP: usize = DEFAULT_QUEUE_CAP> {
    scheduler: S,
    sub_collections: B,
    collection_state: C,
    _p: PhantomData<Q>,
}

impl<Q, S, B, C, const SUB_CAP: usize> BanditCore<Q, S, B, C, SUB_CAP>
where
    S: Default,
{
    pub(crate) fn new_with(queues: B, states: C) -> Self {
        Self {
            scheduler: S::default(),
            sub_collections: queues,
            collection_state: states,
            _p: PhantomData,
        }
    }
}

impl<Q, S, B, C, const SUB_CAP: usize> BanditCore<Q, S, B, C, SUB_CAP>
where
    B: StorageBackend<Q>,
{
    /// returns the number of subqueues
    pub(crate) fn arm_count(&self) -> usize {
        self.sub_collections.len()
    }
}

impl<Q, S, B, C, const SUB_CAP: usize> BanditCore<Q, S, B, C, SUB_CAP>
where
    S: Strategy<Q>,
    Q: Collection,
{
    pub(crate) fn buy_in(&self) -> BanditHandle<'_, Q, S, B, C, SUB_CAP> {
        BanditHandle {
            parent: self,
            arm: self.scheduler.create_gambler(),
        }
    }
}

/// An owned handle into the core collection. May be used for mutabel access of some fields
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct BanditHandle<
    'a,
    Q: Collection,
    S: Strategy<Q>,
    B,
    C,
    const SUB_CAP: usize = DEFAULT_QUEUE_CAP,
> {
    parent: &'a BanditCore<Q, S, B, C, SUB_CAP>,
    arm: S::Gambler,
}

impl<'a, Q, S, B, C, const SUB_CAP: usize> BanditHandle<'a, Q, S, B, C, SUB_CAP>
where
    Q: Collection,
    S: Strategy<Q>,
    B: StorageBackend<Q>,
    C: StorageBackend<<S::Gambler as Hooked>::Stake>,
{
    /// fork this handle into a new one
    pub fn fork(&mut self) -> Self {
        Self {
            parent: self.parent,
            arm: S::fork_gambler(&self.parent.scheduler, &mut self.arm),
        }
    }

    /// push an item into one of the underlying subcollections
    pub fn offer<'b, 'c>(
        &'c mut self,
        item: <Q::OfferSignature as Signature>::Input<'b>,
    ) -> Result<
        <Q::OfferSignature as Signature>::Output<'b, 'c>,
        <Q::OfferSignature as Signature>::Error<'b, 'c>,
    > {
        let i = self
            .parent
            .scheduler
            .choose_offer_arm(&self.parent.collection_state, &mut self.arm);
        match self.parent.sub_collections[i].offer(item) {
            Ok(r) => {
                self.arm.on_offer_succ(&self.parent.collection_state[i]);
                Ok(r)
            }
            Err(e) => {
                self.arm.on_offer_fail(&self.parent.collection_state[i]);
                Err(e)
            }
        }
    }

    /// pop and item from one of the underlying subcollections
    pub fn poll<'b, 'c>(
        &'c mut self,
        input: <Q::PollSignature as Signature>::Input<'b>,
    ) -> Result<
        <Q::PollSignature as Signature>::Output<'b, 'c>,
        <Q::PollSignature as Signature>::Error<'b, 'c>,
    > {
        let i = self
            .parent
            .scheduler
            .choose_poll_arm(&self.parent.collection_state, &mut self.arm);
        match self.parent.sub_collections[i].poll(input) {
            Ok(r) => {
                self.arm.on_poll_succ(&self.parent.collection_state[i]);
                Ok(r)
            }
            Err(e) => {
                self.arm.on_poll_fail(&self.parent.collection_state[i]);
                let r = self.parent.scheduler.collect(
                    &self.parent.collection_state,
                    &self.parent.sub_collections,
                    input,
                );
                if let Some((r, state)) = r {
                    self.arm.on_poll_succ(&self.parent.collection_state[state]);
                    Ok(r)
                } else {
                    Err(e)
                }
            }
        }
    }

    /// pops an item and returns the associated state
    #[allow(clippy::type_complexity)]
    pub fn poll_with_info<'b, 'c>(
        &'c mut self,
        input: <Q::PollSignature as Signature>::Input<'b>,
    ) -> (
        Result<
            <Q::PollSignature as Signature>::Output<'b, 'c>,
            <Q::PollSignature as Signature>::Error<'b, 'c>,
        >,
        <S::Gambler as Hooked>::Stake,
    )
    where
        <S::Gambler as Hooked>::Stake: Clone,
    {
        let i = self
            .parent
            .scheduler
            .choose_poll_arm(&self.parent.collection_state, &mut self.arm);
        match self.parent.sub_collections[i].poll(input) {
            Ok(r) => {
                self.arm.on_poll_succ(&self.parent.collection_state[i]);
                (Ok(r), self.parent.collection_state[i].clone())
            }
            Err(e) => {
                self.arm.on_poll_fail(&self.parent.collection_state[i]);
                let r = self.parent.scheduler.collect(
                    &self.parent.collection_state,
                    &self.parent.sub_collections,
                    input,
                );
                if let Some((r, state)) = r {
                    self.arm.on_poll_succ(&self.parent.collection_state[state]);
                    (Ok(r), self.parent.collection_state[state].clone())
                } else {
                    (Err(e), self.parent.collection_state[i].clone())
                }
            }
        }
    }

    /// state of the scheduler/queues
    pub fn state(&self) -> impl Iterator<Item = &<S::Gambler as Hooked>::Stake> {
        self.parent.collection_state.iter()
    }

    /// the total len of all active subcollections
    pub fn len(&self) -> usize {
        self.parent.sub_collections.iter().map(|q| q.len()).sum()
    }

    /// the total capacity of all active subcollections
    pub fn cap(&self) -> usize {
        self.parent.sub_collections.iter().map(|q| q.cap()).sum()
    }

    /// are all active subcollections empty?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a, Q, S, B, C, const SUB_CAP: usize> BanditHandle<'a, Q, S, B, C, SUB_CAP>
where
    B: StorageBackend<Q>,
    S: Strategy<Q>,
    Q: Collection,
{
    /// returns the number of subqueues
    pub fn nbr_subqueues(&self) -> usize {
        self.parent.arm_count()
    }
}

#[cfg(test)]
impl<'a, Q, S, B, C, const SUB_CAP: usize> BanditHandle<'a, Q, S, B, C, SUB_CAP>
where
    Q: Collection,
    S: Strategy<Q>,
    B: StorageBackend<Q>,
    C: StorageBackend<<S::Gambler as Hooked>::Stake>,
{
    /// polls and returns the index of the shard from which we polled
    #[allow(unused)]
    #[allow(clippy::type_complexity)]
    pub fn poll_with_idx<'b, 'c>(
        &'c mut self,
        input: <Q::PollSignature as Signature>::Input<'b>,
    ) -> Result<
        (<Q::PollSignature as Signature>::Output<'b, 'c>, usize),
        <Q::PollSignature as Signature>::Error<'b, 'c>,
    > {
        let i = self
            .parent
            .scheduler
            .choose_poll_arm(&self.parent.collection_state, &mut self.arm);
        match self.parent.sub_collections[i].poll(input) {
            Ok(r) => {
                self.arm.on_poll_succ(&self.parent.collection_state[i]);
                Ok((r, i))
            }
            Err(e) => {
                self.arm.on_poll_fail(&self.parent.collection_state[i]);
                let r = self.parent.scheduler.collect(
                    &self.parent.collection_state,
                    &self.parent.sub_collections,
                    input,
                );
                if let Some((r, state)) = r {
                    self.arm.on_poll_succ(&self.parent.collection_state[state]);
                    Ok((r, state))
                } else {
                    Err(e)
                }
            }
        }
    }
}
