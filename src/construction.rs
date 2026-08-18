use core::marker::PhantomData;

use crate::{
    Collection,
    IODescription,
    schedule::{Hooked, Schedule},
    storage::StorageBackend,
};

pub(crate) const DEFAULT_QUEUE_CAP: usize = 32;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash)]
pub(crate) struct LopeCore<Q, S, B, C, const SUB_CAP: usize = DEFAULT_QUEUE_CAP> {
    scheduler: S,
    sub_collections: B,
    collection_state: C,
    _p: PhantomData<Q>,
}

impl<Q, S, B, C, const SUB_CAP: usize> LopeCore<Q, S, B, C, SUB_CAP>
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

impl<Q, S, B, C, const SUB_CAP: usize> LopeCore<Q, S, B, C, SUB_CAP>
where
    B: StorageBackend<Q>,
{
    /// returns the number of subqueues
    pub(crate) fn nbr_subqueues(&self) -> usize {
        self.sub_collections.len()
    }
}

impl<Q, S, B, C, const SUB_CAP: usize> LopeCore<Q, S, B, C, SUB_CAP>
where
    S: Schedule,
{
    pub(crate) fn new_root(&self) -> LopeCoreArm<'_, Q, S, B, C, SUB_CAP> {
        LopeCoreArm {
            parent: self,
            arm: self.scheduler.create_arm(),
        }
    }
}

/// An owned handle into the core collection. May be used for mutabel access of some fields
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct LopeCoreArm<'a, Q, S: Schedule, B, C, const SUB_CAP: usize = DEFAULT_QUEUE_CAP> {
    parent: &'a LopeCore<Q, S, B, C, SUB_CAP>,
    arm: S::Arm,
}

impl<'a, Q, S, B, C, const SUB_CAP: usize> LopeCoreArm<'a, Q, S, B, C, SUB_CAP>
where
    Q: Collection,
    S: Schedule,
    B: StorageBackend<Q>,
    C: StorageBackend<<S::Arm as Hooked>::State>,
{
    /// fork this handle into a new one
    pub fn fork(&mut self) -> Self {
        Self {
            parent: self.parent,
            arm: S::fork_arm(&self.parent.scheduler, &mut self.arm),
        }
    }

    /// push an item into one of the underlying subcollections
    pub fn offer(
        &mut self,
        item: <Q::OfferIO as IODescription>::Input,
    ) -> Result<<Q::OfferIO as IODescription>::Output, <Q::OfferIO as IODescription>::Error> {
        let i = self
            .parent
            .scheduler
            .choose_offer_shard(&self.parent.collection_state, &mut self.arm);
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
    pub fn poll(
        &mut self,
        input: <Q::PollIO as IODescription>::Input,
    ) -> Result<<Q::PollIO as IODescription>::Output, <Q::PollIO as IODescription>::Error> {
        let i = self
            .parent
            .scheduler
            .choose_poll_shard(&self.parent.collection_state, &mut self.arm);
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
    pub fn poll_with_info(
        &mut self,
        input: <Q::PollIO as IODescription>::Input,
    ) -> (
        Result<<Q::PollIO as IODescription>::Output, <Q::PollIO as IODescription>::Error>,
        <S::Arm as Hooked>::State,
    )
    where
        <S::Arm as Hooked>::State: Clone,
    {
        let i = self
            .parent
            .scheduler
            .choose_poll_shard(&self.parent.collection_state, &mut self.arm);
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
    pub fn state(&self) -> impl Iterator<Item = &<S::Arm as Hooked>::State> {
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

impl<'a, Q, S, B, C, const SUB_CAP: usize> LopeCoreArm<'a, Q, S, B, C, SUB_CAP>
where
    B: StorageBackend<Q>,
    S: Schedule,
{
    /// returns the number of subqueues
    pub fn nbr_subqueues(&self) -> usize {
        self.parent.nbr_subqueues()
    }
}

#[cfg(test)]
impl<'a, Q, S, B, C, const SUB_CAP: usize> LopeCoreArm<'a, Q, S, B, C, SUB_CAP>
where
    Q: Collection,
    S: Schedule,
    B: StorageBackend<Q>,
    C: StorageBackend<<S::Arm as Hooked>::State>,
{
    /// polls and returns the index of the shard from which we polled
    #[allow(unused)]
    #[allow(clippy::type_complexity)]
    pub fn poll_with_idx(
        &mut self,
        input: <Q::PollIO as IODescription>::Input,
    ) -> Result<(<Q::PollIO as IODescription>::Output, usize), <Q::PollIO as IODescription>::Error>
    {
        let i = self
            .parent
            .scheduler
            .choose_poll_shard(&self.parent.collection_state, &mut self.arm);
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
