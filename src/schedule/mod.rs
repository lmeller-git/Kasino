//! schedulers used in this crate

mod dcbo;
mod dra;
mod random;
mod round_robin;

pub use dcbo::DCBO;
pub use dra::DRA;
pub use random::RandomAccess;
pub use round_robin::RoundRobin;

use crate::storage::StorageBackend;

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
}

/// callbacks actuated by successful operations on Lope, which influence the schedulers next decision
pub trait Hooked {
    /// The type of state associated with this hook
    type State: Default;
    /// mutate the state on a succesfull enqueue
    fn on_enq(&mut self, _sub_state: &Self::State) {}
    /// mutatet the state on a succesfull dequeue
    fn on_deq(&mut self, _sub_state: &Self::State) {}
}
