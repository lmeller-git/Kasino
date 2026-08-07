mod dcbo;
mod dra;
mod random;
mod round_robin;

pub trait Schedule<T>: Default {
    type State: Default;
    type Arm<'a>: Hooked
    where
        Self: 'a;

    fn choose_enq(&self, choose_to: usize, arm: &mut Self::Arm<'_>) -> usize;
    fn choose_deq(&self, choose_to: usize, arm: &mut Self::Arm<'_>) -> usize;

    fn fork_arm(&self, arm: &mut Self::Arm<'_>) -> Self::Arm<'_>;
    fn create_arm(&self) -> Self::Arm<'_>;

    #[cfg(feature = "alloc")]
    fn grow(&self) {}
}

pub trait Hooked {
    fn on_enq(&mut self, choice: usize) {}
    fn on_deq(&mut self, choice: usize) {}
}
