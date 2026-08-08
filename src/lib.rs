//! TODO

#![cfg_attr(not(any(feature = "std", test)), no_std)]
// #![deny(missing_docs)]
#![deny(clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
#![warn(unsafe_op_in_unsafe_fn)]

#[cfg(any(feature = "std", test))]
extern crate std;

#[allow(unused_extern_crates)]
#[cfg(any(feature = "alloc", test))]
extern crate alloc;

#[cfg(feature = "alloc")]
mod boxed;
mod construction;
mod inline;
pub mod schedule;
pub mod storage;
mod sync;

#[cfg(feature = "alloc")]
pub use boxed::*;
pub use construction::LopeCoreArm;
pub use inline::*;

pub trait Collection {
    type Item;

    fn push(&self, item: Self::Item) -> Result<(), Self::Item>;
    fn pop(&self) -> Option<Self::Item>;
    fn len(&self) -> usize;
    fn cap(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait NewSized<const N: usize> {
    fn with_capacity() -> Self;
}
