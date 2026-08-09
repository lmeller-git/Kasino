use core::marker::PhantomData;

use crate::{
    Collection,
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
    S: Schedule<Q>,
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
pub struct LopeCoreArm<'a, Q, S: Schedule<Q>, B, C, const SUB_CAP: usize = DEFAULT_QUEUE_CAP> {
    parent: &'a LopeCore<Q, S, B, C, SUB_CAP>,
    arm: S::Arm,
}

impl<'a, Q, S, B, C, const SUB_CAP: usize> LopeCoreArm<'a, Q, S, B, C, SUB_CAP>
where
    Q: Collection,
    S: Schedule<Q>,
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
    pub fn push(&mut self, item: Q::Item) -> Result<(), Q::Item> {
        let i = self
            .parent
            .scheduler
            .choose_enq(&self.parent.collection_state, &mut self.arm);
        self.parent.sub_collections[i].push(item)?;
        self.arm.on_enq(&self.parent.collection_state[i]);
        Ok(())
    }

    /// pop and item from one of the underlying subcollections
    pub fn pop(&mut self) -> Option<Q::Item> {
        let i = self
            .parent
            .scheduler
            .choose_deq(&self.parent.collection_state, &mut self.arm);
        if let Some(item) = self.parent.sub_collections[i].pop() {
            self.arm.on_deq(&self.parent.collection_state[i]);
            return Some(item);
        }

        // TODO: may want to do a double-collect pass here to grant empty-linearizability.
        // But this is dependant on schedule and may not always be possible (for example on random schedule).
        // Is also not strictly necessary and reduces performance
        for (i, q) in self.parent.sub_collections.iter().enumerate() {
            if let Some(item) = q.pop() {
                self.arm.on_deq(&self.parent.collection_state[i]);
                return Some(item);
            }
        }

        None
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

#[cfg(test)]
impl<'a, Q, S, B, C, const SUB_CAP: usize> LopeCoreArm<'a, Q, S, B, C, SUB_CAP>
where
    Q: Collection,
    S: Schedule<Q>,
    B: StorageBackend<Q>,
    C: StorageBackend<<S::Arm as Hooked>::State>,
{
    #[allow(unused)]
    pub(crate) fn pop_with_idx(&mut self) -> Option<(Q::Item, usize)> {
        let i = self
            .parent
            .scheduler
            .choose_enq(&self.parent.collection_state, &mut self.arm);
        if let Some(item) = self.parent.sub_collections[i].pop() {
            self.arm.on_deq(&self.parent.collection_state[i]);
            return Some((item, i));
        }
        // TODO: may want to do a double-collect pass here to grant empty-linearizability.
        // But this is dependant on schedule and may not always be possible (for example on random schedule).
        // Is also not strictly necessary and reduces performance
        for (i, q) in self.parent.sub_collections.iter().enumerate() {
            if let Some(item) = q.pop() {
                self.arm.on_deq(&self.parent.collection_state[i]);
                return Some((item, i));
            }
        }

        None
    }
}
