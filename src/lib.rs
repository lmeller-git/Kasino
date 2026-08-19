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
//! # use kasino::{Collection, WithCapacity, Signature};
//! # use std::sync::Mutex;
//! # use std::collections::VecDeque;
//! # use std::marker::PhantomData;
//! # struct QueuePushSignature<T>(PhantomData<T>);
//! # impl<T> Signature for QueuePushSignature<T> {
//! #     type Input<'a> = T;
//! #     type Output<'io, 'arm> = () where Self: 'arm;
//! #     type Error<'io, 'arm> = T where Self: 'arm;
//! # }
//! # struct QueuePollSignature<T>(PhantomData<T>);
//! # impl<T> Signature for QueuePollSignature<T> {
//! #     type Input<'a> = ();
//! #     type Output<'io, 'arm> = T where Self: 'arm;
//! #     type Error<'io, 'arm> = () where Self: 'arm;
//! # }
//! #
//! # struct MyQueue<T> { deque: Mutex<VecDeque<T>>, cap: usize }
//! # impl<T> Collection for MyQueue<T> {
//! #     type PollSignature = QueuePollSignature<T>;
//! #     type OfferSignature = QueuePushSignature<T>;
//! #     fn offer<'io, 'arm>(
//! #         &'arm self,
//! #         item: <Self::OfferSignature as Signature>::Input<'io>,
//! #     ) -> Result<
//! #         <Self::OfferSignature as Signature>::Output<'io, 'arm>,
//! #         <Self::OfferSignature as Signature>::Error<'io, 'arm>,
//! #     > {
//! #         let mut g = self.deque.lock().unwrap();
//! #         if g.len() >= self.cap { Err(item) } else { g.push_back(item); Ok(()) }
//! #     }
//! #     fn poll<'io, 'arm>(
//! #         &'arm self,
//! #         input: <Self::PollSignature as Signature>::Input<'io>,
//! #     ) -> Result<
//! #         <Self::PollSignature as Signature>::Output<'io, 'arm>,
//! #         <Self::PollSignature as Signature>::Error<'io, 'arm>,
//! #     > {
//! #         self.deque.lock().unwrap().pop_front().ok_or(())
//! #     }
//! #     fn len(&self) -> usize { self.deque.lock().unwrap().len() }
//! #     fn cap(&self) -> usize { self.cap }
//! # }
//! # impl<T, const N: usize> WithCapacity<N> for MyQueue<T> {
//! #     fn with_capacity() -> Self { Self { deque: Mutex::new(VecDeque::with_capacity(N)), cap: N } }
//! # }
//! use kasino::{InlineBandit, strategy::DCBO};
//!
//! let bandit = InlineBandit::<MyQueue<i32>, DCBO, 8>::new();
//!
//! let mut handle = bandit.buy_in();
//! let mut handle2 = handle.fork();
//!
//! assert!(handle.offer(42).is_ok());
//! assert!(handle2.offer(10).is_ok());
//! assert!(handle.poll(()).is_ok());
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
//! - **Linearizability**: if the wrapped collection is linearizable, all operations on [`Lope`] are also linearizable with respect to its relaxed FIFO specification.
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
pub mod storage;
pub mod strategy;
mod sync;

#[cfg(test)]
mod tests;

#[cfg(feature = "alloc")]
pub use boxed::*;
pub use construction::BanditHandle;
pub use inline::*;

/// Description about the surface of a failable method
pub trait Signature {
    /// The input
    type Input<'a>;
    /// The successful output
    type Output<'io, 'arm>
    where
        Self: 'arm;
    /// the error
    type Error<'io, 'arm>
    where
        Self: 'arm;
}

/// A collection that supports `push` and `pop` operations.
///
/// the specification ordering of this collection may influence the rank error and delay of the sharded version.
pub trait Collection
where
    for<'a> <Self::PollSignature as Signature>::Input<'a>: Copy,
{
    /// The item stored in this collection.
    type OfferSignature: Signature;
    /// the contract of poll
    type PollSignature: Signature;

    /// pushes an item into the collection
    fn offer<'io, 'arm>(
        &'arm self,
        item: <Self::OfferSignature as Signature>::Input<'io>,
    ) -> Result<
        <Self::OfferSignature as Signature>::Output<'io, 'arm>,
        <Self::OfferSignature as Signature>::Error<'io, 'arm>,
    >;
    /// pops an item from the collection
    fn poll<'io, 'arm>(
        &'arm self,
        input: <Self::PollSignature as Signature>::Input<'io>,
    ) -> Result<
        <Self::PollSignature as Signature>::Output<'io, 'arm>,
        <Self::PollSignature as Signature>::Error<'io, 'arm>,
    >;
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
pub trait WithCapacity<const N: usize> {
    /// Constructs a new Collection with capacity N
    fn with_capacity() -> Self;
}
