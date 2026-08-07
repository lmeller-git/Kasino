use core::marker::PhantomData;

use crate::{
    Collection,
    schedule::{Hooked, Schedule},
    storage::StorageBackend,
};

pub struct LopeCore<Q, S, B> {
    scheduler: S,
    sub_collections: B,
    _p: PhantomData<Q>,
}

impl<Q: Collection + Default, S: Schedule<Q> + Default, B: StorageBackend<Q> + Default>
    LopeCore<Q, S, B>
{
    pub fn new() -> Self {
        Self {
            scheduler: S::default(),
            sub_collections: B::default(),
            _p: PhantomData,
        }
    }

    pub fn new_root(&self) -> LopeCoreArm<'_, Q, S, B> {
        LopeCoreArm {
            parent: self,
            arm: self.scheduler.create_arm(),
        }
    }
}

pub struct LopeCoreArm<'a, Q, S: Schedule<Q>, B> {
    parent: &'a LopeCore<Q, S, B>,
    arm: S::Arm<'a>,
}

impl<'a, Q: Collection, S: Schedule<Q>, B: StorageBackend<Q>> LopeCoreArm<'a, Q, S, B> {
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
            .choose_enq(self.parent.sub_collections.len(), &mut self.arm);
        self.parent.sub_collections.as_slice()[i].push(item)?;
        self.arm.on_deq(i);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<Q::Item> {
        let i = self
            .parent
            .scheduler
            .choose_enq(self.parent.sub_collections.len(), &mut self.arm);
        let r = self.parent.sub_collections.as_slice()[i].pop();
        if r.is_some() {
            self.arm.on_deq(i);
        }
        // TODO: may want to do a double-collect pass here to grant empty-linearizability.
        // But this is dependant on schedule and may not always be possible (for example on random schedule).
        // Is also not strictly necessary and reduces performance
        r
    }

    pub fn len(&self) -> usize {
        self.parent
            .sub_collections
            .as_slice()
            .iter()
            .map(|q| q.len())
            .sum()
    }

    pub fn cap(&self) -> usize {
        self.parent
            .sub_collections
            .as_slice()
            .iter()
            .map(|q| q.cap())
            .sum()
    }
}
