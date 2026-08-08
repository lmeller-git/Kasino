mod dcbo;
mod dra;
mod random;
mod round_robin;

pub use dcbo::*;
pub use dra::*;
pub use random::*;
pub use round_robin::*;

use crate::storage::StorageBackend;

pub trait Schedule<T> {
    type Arm: Hooked;

    fn choose_enq(
        &self,
        state: &impl StorageBackend<<Self::Arm as Hooked>::State>,
        arm: &mut Self::Arm,
    ) -> usize;
    fn choose_deq(
        &self,
        choose_to: &impl StorageBackend<<Self::Arm as Hooked>::State>,
        arm: &mut Self::Arm,
    ) -> usize;

    fn fork_arm(&self, arm: &mut Self::Arm) -> Self::Arm;
    fn create_arm(&self) -> Self::Arm;
}

pub trait Hooked {
    type State: Default;
    fn on_enq(&mut self, _sub_state: &Self::State) {}
    fn on_deq(&mut self, _sub_state: &Self::State) {}
}
