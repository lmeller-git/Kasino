//! A construction that elastically relaxes a given collection.
//!
//! `Lope` aims to improve performance of concurrent datastcurtures by sharding operations into multiple subqueues.
//! This process introduces a relaxation of the wrapped datastruture, the specifics depending on the used scheduler.
//!
//! Multiple schedulers, ameneable to different kinds of datastructures and requirements are provided.
//!
//! Additionally an interface for defining custom schedulers is available.
//!
//! ## Usage
//!
//! ```rust
//! # use lope::{Collection, NewSized};
//! # use std::sync::Mutex;
//! # use std::collections::VecDeque;
//! #
//! # struct MyQueue<T> { deque: Mutex<VecDeque<T>>, cap: usize }
//! # impl<T> Collection for MyQueue<T> {
//! #     type Item = T;
//! #     fn push(&self, item: T) -> Result<(), T> {
//! #         let mut g = self.deque.lock().unwrap();
//! #         if g.len() >= self.cap { Err(item) } else { g.push_back(item); Ok(()) }
//! #     }
//! #     fn pop(&self) -> Option<T> { self.deque.lock().unwrap().pop_front() }
//! #     fn len(&self) -> usize { self.deque.lock().unwrap().len() }
//! #     fn cap(&self) -> usize { self.cap }
//! # }
//! # impl<T, const N: usize> NewSized<N> for MyQueue<T> {
//! #     fn with_capacity() -> Self { Self { deque: Mutex::new(VecDeque::with_capacity(N)), cap: N } }
//! # }
//! use lope::{InlineLope, schedule::DCBO};
//! // Create a Lope wrapping your datastructure
//! let container = InlineLope::<MyQueue<i32>, DCBO, 8>::new();
//! // Create a new owned handle to this container
//! let mut my_handle = container.new_root();
//! // fork this handle
//! let mut my_handle2 = my_handle.fork();
//!
//! // It implements all operations of a Collection
//! assert!(my_handle.push(42).is_ok());
//! assert!(my_handle2.push(10).is_ok());
//! _ = my_handle.pop();
//! ```
//!
//! ## Property preservation
//!
//! ### Progress Guarantees:
//!
//! - **Lock Freedom**: if the wrapped collection is lock-free, [`Lope`] is also lock-free.
//! - **Obstruction Freedom**: if the wrapped collection exposes obstruction-free methods, all corresponding operations on [`Lope`] are also obstruction-free.
//!
//! ### Ordering and Consistency Guarantees:
//!
//! - **Relaxed FIFO**: if the wrapped collection has FIFO ordering, [`Lope`] has **k-FIFO** ordering.
//! - **Linearizability**: if the wrapped collection is linearizable, all operations on [`Resizable`] are also linearizable with respect to its relaxed FIFO specification.
//!
//! ### Relaxation
//!
//! The rank error and delay are in general unbounded. However, the rank error and delay of some schedules is bounded with high probabilty.
//! The exact bounds here are differing across different schedulers.
//!
//! For more information refer to the schedulers documenation and the reference papers.
//!
//! ## Perfomance
//!
//! TODO
//!
//! ## Limitations
//!
//! - Currently an instantiated Lope cannot be resized. Its capacity is fixed at construction time.
//! - The capacity of each sub collection is fixed statically. The total capacity of Lope is constrained to a multiple of this.
//!
//! ## Platform Support
//!
//! All platforms supporting native atomic operations are supported.
//!
//! The feature `atomic-fallback` may be used, if no native atomic operations are available.
//!
//! ## Feature Flags
//!
//! - `std`: Enables `std` support.
//! - `atomic-fallback`:  Uses the `portable-atomic` fallback feature if native atomics are missing. It is discouraged to use this feature, as fallback atomics internally rely on locks.
//! - `default`: None
//!
//! ## Testing
//!
//! Currently testing is based on:
//!
//! - **Miri** - to validate pointer arithmetic and catch undefined behavior.
//! - **Loom and Shuttle** - to test for race conditions and non-blocking invariants.
//! - **ASan** - to check for memory corruption.
//!
//! ## References
//!
//! - Performance, Scalability, and Semantics of Concurrent FIFO Queues, Kirsch et al.
//! - Balanced Allocations over Efficient Queues: A Fast Relaxed FIFO Queue, Geijer et al.

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
