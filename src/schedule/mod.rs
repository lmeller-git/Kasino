//! schedulers used in this crate

mod collect;
mod dcbo;
mod dra;
mod random;
mod round_robin;

use core::ops::{Deref, DerefMut};

use crossbeam_utils::CachePadded;
pub use dcbo::DCBO;
pub use dra::DRA;
pub use random::RandomAccess;
pub use round_robin::RoundRobin;

use crate::{
    Collection,
    storage::StorageBackend,
    sync::atomic::{AtomicUsize, Ordering},
};

/// A schedule that determins which arm to pull next
pub trait Schedule<T> {
    /// An owned hook to the schedule
    type Arm: Hooked;

    /// choose the next arm that we call push on
    fn choose_enq(
        &self,
        state: &impl StorageBackend<<Self::Arm as Hooked>::State>,
        arm: &mut Self::Arm,
    ) -> usize;
    /// choose the next arm that we call pop on
    fn choose_deq(
        &self,
        choose_to: &impl StorageBackend<<Self::Arm as Hooked>::State>,
        arm: &mut Self::Arm,
    ) -> usize;

    /// forks an owned hook into a new one
    fn fork_arm(&self, arm: &mut Self::Arm) -> Self::Arm;
    /// creates a new owned hook with default specs
    fn create_arm(&self) -> Self::Arm;

    /// Run a collect strategy on a queue to ensure we collect any remaingin item if one exists.
    fn collect<Q: Collection>(
        &self,
        _state: &impl StorageBackend<<Self::Arm as Hooked>::State>,
        sub_collections: &impl StorageBackend<Q>,
    ) -> Option<(Q::Item, usize)> {
        for (i, q) in sub_collections.iter().enumerate() {
            if let Some(item) = q.pop() {
                return Some((item, i));
            }
        }
        None
    }
}

/// a hook for a scheduler state
pub trait Hook {
    /// mutate the state on a succesfull enqueue
    fn on_enq(&self) {}
    /// mutatet the state on a succesfull dequeue
    fn on_deq(&self) {}
}

/// callbacks actuated by successful operations on Lope, which influence the schedulers next decision
pub trait Hooked {
    /// The type of state associated with this hook
    type State: Default + Hook;
    /// mutate the state on a succesfull enqueue
    fn on_enq(&mut self, sub_state: &Self::State) {
        sub_state.on_enq();
    }
    /// mutatet the state on a succesfull dequeue
    fn on_deq(&mut self, sub_state: &Self::State) {
        sub_state.on_deq();
    }
}

/// instrumented scheduler state
#[cfg(debug_assertions)]
pub type InstrumentedState<T> = DbgState<T>;
/// instrumented scheduler state
#[cfg(not(debug_assertions))]
pub type InstrumentedState<T> = T;

/// State useful for instrumenting various calls
#[cfg(debug_assertions)]
#[derive(Debug, Default)]
pub struct DbgState<T> {
    enq_count: AtomicUsize,
    deq_count: AtomicUsize,
    sched_state: T,
}

#[cfg(debug_assertions)]
impl<T> DbgState<T> {
    /// the enqueue count
    pub fn enq(&self) -> usize {
        self.enq_count.load(Ordering::Relaxed)
    }

    /// the dequeue count
    pub fn deq(&self) -> usize {
        self.deq_count.load(Ordering::Relaxed)
    }
}

#[cfg(debug_assertions)]
impl<T> Deref for DbgState<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.sched_state
    }
}

#[cfg(debug_assertions)]
impl<T> DerefMut for DbgState<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sched_state
    }
}

#[cfg(debug_assertions)]
impl<T> Clone for DbgState<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            enq_count: self.enq_count.load(Ordering::Relaxed).into(),
            deq_count: self.deq_count.load(Ordering::Relaxed).into(),
            sched_state: self.sched_state.clone(),
        }
    }
}

#[cfg(debug_assertions)]
impl<T> Hook for DbgState<T>
where
    T: Hook,
{
    fn on_enq(&self) {
        self.enq_count.fetch_add(1, Ordering::Relaxed);
        self.sched_state.on_enq();
    }

    fn on_deq(&self) {
        self.deq_count.fetch_add(1, Ordering::Relaxed);
        self.sched_state.on_deq();
    }
}

/// a state that stores enqueue and dequeue count
#[derive(Default, Debug)]
pub struct EDCount {
    enq: AtomicUsize,
    deq: AtomicUsize,
}

impl Clone for EDCount {
    fn clone(&self) -> Self {
        Self {
            enq: self.enq.load(Ordering::Relaxed).into(),
            deq: self.deq.load(Ordering::Relaxed).into(),
        }
    }
}

impl Hook for EDCount {
    fn on_enq(&self) {
        self.enq.fetch_add(1, Ordering::Relaxed);
    }

    fn on_deq(&self) {
        self.deq.fetch_add(1, Ordering::Relaxed);
    }
}

impl Hook for () {}

impl<T> Hook for CachePadded<T>
where
    T: Hook,
{
    fn on_enq(&self) {
        T::on_enq(self);
    }

    fn on_deq(&self) {
        T::on_deq(self);
    }
}

/// a transparent wrapper around a T
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoPad<T>(T);

impl<T> Deref for NoPad<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for NoPad<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Hook for NoPad<T>
where
    T: Hook,
{
    fn on_enq(&self) {
        self.0.on_enq();
    }

    fn on_deq(&self) {
        self.0.on_deq();
    }
}
