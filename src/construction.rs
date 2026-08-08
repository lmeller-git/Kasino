use core::marker::PhantomData;

#[cfg(feature = "alloc")]
use crate::storage::GrowingBackend;
use crate::{
    Collection,
    schedule::{Hooked, Schedule},
    storage::StorageBackend,
};

pub struct LopeCore<Q, S, B, C> {
    scheduler: S,
    sub_collections: B,
    collection_state: C,
    _p: PhantomData<Q>,
}

impl<
    Q: Collection + Default,
    S: Schedule<Q> + Default,
    B: StorageBackend<Q> + Default,
    C: StorageBackend<<S::Arm as Hooked>::State> + Default,
> LopeCore<Q, S, B, C>
{
    pub fn new() -> Self {
        Self {
            scheduler: S::default(),
            sub_collections: B::default(),
            collection_state: C::default(),
            _p: PhantomData,
        }
    }

    pub fn new_root(&self) -> LopeCoreArm<'_, Q, S, B, C> {
        LopeCoreArm {
            parent: self,
            arm: self.scheduler.create_arm(),
        }
    }
}

#[cfg(feature = "alloc")]
impl<
    Q: Collection,
    S: Schedule<Q> + Default,
    B: GrowingBackend<Q>,
    C: GrowingBackend<<S::Arm as Hooked>::State>,
> LopeCore<Q, S, B, C>
{
    pub fn add_queue(&self, q: Q) {
        self.sub_collections.push(q);
        self.collection_state
            .push(<S::Arm as Hooked>::State::default());
    }
}

pub struct LopeCoreArm<'a, Q, S: Schedule<Q>, B, C> {
    parent: &'a LopeCore<Q, S, B, C>,
    arm: S::Arm,
}

impl<
    'a,
    Q: Collection,
    S: Schedule<Q>,
    B: StorageBackend<Q>,
    C: StorageBackend<<S::Arm as Hooked>::State>,
> LopeCoreArm<'a, Q, S, B, C>
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
    Q: Collection,
    S: Schedule<Q> + Default,
    B: GrowingBackend<Q>,
    C: GrowingBackend<<S::Arm as Hooked>::State>,
> LopeCoreArm<'a, Q, S, B, C>
{
    pub fn add_queue(&self, q: Q) {
        self.parent.add_queue(q)
    }
}
