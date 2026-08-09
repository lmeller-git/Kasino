use crate::{
    InlineLope,
    schedule::DCBO,
    tests::test_library::{LockedDeque, mpmc, mpmc_ring_buffer, mpsc, spsc},
};

const RETRIES: usize = 1000;
const DEPTH: usize = 10;

#[test]
fn spsc_impl() {
    shuttle::check_pct(
        || {
            let q: InlineLope<LockedDeque<u32>, DCBO, 2> = InlineLope::new();
            spsc(q.new_root());
        },
        RETRIES,
        DEPTH,
    );
}

#[test]
fn mpsc_impl() {
    shuttle::check_pct(
        || {
            let q: InlineLope<LockedDeque<u32>, DCBO, 2> = InlineLope::new();
            mpsc(q.new_root());
        },
        RETRIES,
        DEPTH,
    );
}

#[test]
fn mpmc_impl() {
    shuttle::check_pct(
        || {
            let q: InlineLope<LockedDeque<u32>, DCBO, 2> = InlineLope::new();
            mpmc(q.new_root());
        },
        RETRIES,
        DEPTH,
    );
}

#[test]
fn mpmc_ring_buffer_impl() {
    shuttle::check_pct(
        || {
            let q: InlineLope<LockedDeque<u32>, DCBO, 2> = InlineLope::new();
            mpmc_ring_buffer(q.new_root());
        },
        RETRIES,
        DEPTH,
    );
}
