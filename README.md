[![Codecov](https://codecov.io/github/lmeller-git/lope/coverage.svg?branch=main)](https://codecov.io/gh/lmeller-git/lope)
![CI Test](https://github.com/lmeller-git/lope/actions/workflows/test.yml/badge.svg?branch=main)
![Safety Test](https://github.com/lmeller-git/lope/actions/workflows/safety.yml/badge.svg?branch=main)
![no_std Test](https://github.com/lmeller-git/lope/actions/workflows/nostd.yml/badge.svg?branch=main)
[![Crates.io](https://img.shields.io/crates/v/lope)](https://crates.io/crates/lope)
[![Docs.rs](https://docs.rs/lope/badge.svg)](https://docs.rs/lope)

# Lope


<!-- cargo-rdme start -->

A construction that elastically relaxes a given collection.

`Lope` aims to improve performance of concurrent datastcurtures by sharding operations into multiple subqueues.
This process introduces a relaxation of the wrapped datastruture, the specifics depending on the used scheduler.

Multiple schedulers, ameneable to different kinds of datastructures and requirements are provided.

Additionally an interface for defining custom schedulers is available.

### Usage

```rust
use lope::{InlineLope, schedule::DCBO};
// Create a Lope wrapping your datastructure
let container = InlineLope::<MyQueue<i32>, DCBO, 8>::new();
// Create a new owned handle to this container
let mut my_handle = container.new_root();
// fork this handle
let mut my_handle2 = my_handle.fork();

// It implements all operations of a Collection
assert!(my_handle.push(42).is_ok());
assert!(my_handle2.push(10).is_ok());
_ = my_handle.pop();
```

### Property preservation

#### Progress Guarantees:

- **Lock Freedom**: if the wrapped collection is lock-free, [`Lope`] is also lock-free.
- **Obstruction Freedom**: if the wrapped collection exposes obstruction-free methods, all corresponding operations on [`Lope`] are also obstruction-free.

#### Ordering and Consistency Guarantees:

- **Relaxed FIFO**: if the wrapped collection has FIFO ordering, [`Lope`] has **k-FIFO** ordering.
- **Linearizability**: if the wrapped collection is linearizable, all operations on [`Lope`] are also linearizable with respect to its relaxed FIFO specification.

#### Relaxation

The rank error and delay are in general unbounded. However, the rank error and delay of some schedules is bounded with high probabilty.
The exact bounds here are differing across different schedulers.

For more information refer to the schedulers documenation and the reference papers.

### Perfomance

TODO

### Limitations

- Currently an instantiated Lope cannot be resized. Its capacity is fixed at construction time.
- The capacity of each sub collection is fixed statically. The total capacity of Lope is constrained to a multiple of this.

### Platform Support

All platforms supporting native atomic operations are supported.

The feature `atomic-fallback` may be used, if no native atomic operations are available.

### Feature Flags

- `std`: Enables `std` support.
- `atomic-fallback`:  Uses the `portable-atomic` fallback feature if native atomics are missing. It is discouraged to use this feature, as fallback atomics internally rely on locks.
- `default`: None

### Testing

Currently testing is based on:

- **Miri** - to validate pointer arithmetic and catch undefined behavior.
- **Loom and Shuttle** - to test for race conditions and non-blocking invariants.
- **ASan** - to check for memory corruption.

### References

- Performance, Scalability, and Semantics of Concurrent FIFO Queues, Kirsch et al.
- Balanced Allocations over Efficient Queues: A Fast Relaxed FIFO Queue, Geijer et al.

<!-- cargo-rdme end -->

