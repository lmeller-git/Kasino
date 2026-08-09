//! TODO

#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![deny(missing_docs)]
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

#[cfg(test)]
mod tests;

#[cfg(feature = "alloc")]
pub use boxed::*;
pub use construction::LopeCoreArm;
pub use inline::*;

/// A collection that supports `push` and `pop` operations.
///
/// the specification ordering of this collection may influence the rank error and delay of the sharded version.
pub trait Collection {
    /// The item stored in this collection.
    type Item;

    /// pushes an item into the collection
    fn push(&self, item: Self::Item) -> Result<(), Self::Item>;
    /// pops an item from the collection
    fn pop(&self) -> Option<Self::Item>;
    /// the length of the collection
    fn len(&self) -> usize;
    /// the capacity of the collection
    fn cap(&self) -> usize;

    /// is the collection empty?
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A collection that may be created with a static initial capacity N
pub trait NewSized<const N: usize> {
    /// Constructs a new Collection with capacity N
    fn with_capacity() -> Self;
}
