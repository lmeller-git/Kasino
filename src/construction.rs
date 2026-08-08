use core::marker::PhantomData;

use crate::{
    Collection,
    schedule::{Hooked, Schedule},
    storage::StorageBackend,
};
#[cfg(feature = "alloc")]
use crate::{NewSized, storage::GrowingBackend};

pub struct LopeCore<Q, S, B, C, const SUB_CAP: usize = 32> {
    scheduler: S,
    sub_collections: B,
    collection_state: C,
    _p: PhantomData<Q>,
}

impl<
    Q: Collection,
    S: Schedule<Q> + Default,
    B: StorageBackend<Q>,
    C: StorageBackend<<S::Arm as Hooked>::State>,
    const SUB_CAP: usize,
> LopeCore<Q, S, B, C, SUB_CAP>
{
    pub fn new_with(queues: B, states: C) -> Self {
        Self {
            scheduler: S::default(),
            sub_collections: queues,
            collection_state: states,
            _p: PhantomData,
        }
    }

    pub fn new_root(&self) -> LopeCoreArm<'_, Q, S, B, C, SUB_CAP> {
        LopeCoreArm {
            parent: self,
            arm: self.scheduler.create_arm(),
        }
    }
}

#[cfg(feature = "alloc")]
impl<
    Q: Collection + NewSized<SUB_CAP>,
    S: Schedule<Q> + Default,
    B: GrowingBackend<Q>,
    C: GrowingBackend<<S::Arm as Hooked>::State>,
    const SUB_CAP: usize,
> LopeCore<Q, S, B, C, SUB_CAP>
{
    pub fn add_queue(&self) {
        self.sub_collections
            .push(<Q as NewSized<SUB_CAP>>::with_capacity());
        self.collection_state
            .push(<S::Arm as Hooked>::State::default());
    }
}

pub struct LopeCoreArm<'a, Q, S: Schedule<Q>, B, C, const SUB_CAP: usize = 32> {
    parent: &'a LopeCore<Q, S, B, C, SUB_CAP>,
    arm: S::Arm,
}

impl<
    'a,
    Q: Collection,
    S: Schedule<Q>,
    B: StorageBackend<Q>,
    C: StorageBackend<<S::Arm as Hooked>::State>,
    const SUB_CAP: usize,
> LopeCoreArm<'a, Q, S, B, C, SUB_CAP>
{
    pub fn fork(&mut self) -> Self {
        Self {
            parent: self.parent,
            arm: S::fork_arm(&self.parent.scheduler, &mut self.arm),
        }
    }

    pub fn push(&mut self, item: Q::Item) -> Result<(), Q::Item> {
        let i = self
            .parent
            .scheduler
            .choose_enq(&self.parent.collection_state, &mut self.arm);
        self.parent.sub_collections[i].push(item)?;
        self.arm.on_enq(&self.parent.collection_state[i]);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<Q::Item> {
        let i = self
            .parent
            .scheduler
            .choose_enq(&self.parent.collection_state, &mut self.arm);
        let r = self.parent.sub_collections[i].pop();
        if r.is_some() {
            self.arm.on_deq(&self.parent.collection_state[i]);
        }
        // TODO: may want to do a double-collect pass here to grant empty-linearizability.
        // But this is dependant on schedule and may not always be possible (for example on random schedule).
        // Is also not strictly necessary and reduces performance
        r
    }

    pub fn len(&self) -> usize {
        self.parent.sub_collections.iter().map(|q| q.len()).sum()
    }

    pub fn cap(&self) -> usize {
        self.parent.sub_collections.iter().map(|q| q.cap()).sum()
    }
}

#[cfg(feature = "alloc")]
impl<
    'a,
    Q: Collection + NewSized<SUB_CAP>,
    S: Schedule<Q> + Default,
    B: GrowingBackend<Q>,
    C: GrowingBackend<<S::Arm as Hooked>::State>,
    const SUB_CAP: usize,
> LopeCoreArm<'a, Q, S, B, C, SUB_CAP>
{
    pub fn add_queue(&self) {
        self.parent.add_queue()
    }
}
