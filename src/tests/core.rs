use crate::{
    InlineLope,
    schedule::{DCBO, DoubleCollect},
    tests::test_library::{
        LockedDeque,
        force_push,
        len,
        len_empty_full,
        linearizable,
        mpmc,
        mpmc_ring_buffer,
        mpsc,
        smoke,
        smoke_long,
        spsc,
    },
};

#[test]
fn smoke_impl() {
    let q: InlineLope<LockedDeque<u32>, DCBO, 2> = InlineLope::new();
    smoke(q.new_root());
}

#[test]
fn smoke_long_impl() {
    let q: InlineLope<LockedDeque<u32>, DCBO, 2> = InlineLope::new();
    smoke_long(q.new_root());
}

#[test]
fn force_push_impl() {
    let q: InlineLope<LockedDeque<u32>, DCBO, 2> = InlineLope::new();
    force_push(q.new_root());
}

#[test]
fn len_impl() {
    let q: InlineLope<LockedDeque<u32>, DCBO, 2> = InlineLope::new();
    len(q.new_root());
}

#[test]
fn len_empty_full_impl() {
    let q: InlineLope<LockedDeque<()>, DCBO, 2, 1> = InlineLope::new();
    len_empty_full(q.new_root());
}

#[test]
fn mpmc_impl() {
    let q: InlineLope<LockedDeque<u32>, DCBO, 2> = InlineLope::new();
    mpmc(q.new_root());
}

#[test]
fn mpmc_ring_buffer_impl() {
    let q: InlineLope<LockedDeque<u32>, DCBO, 2> = InlineLope::new();
    mpmc_ring_buffer(q.new_root());
}

#[test]
fn mpsc_impl() {
    let q: InlineLope<LockedDeque<u32>, DCBO, 2> = InlineLope::new();
    mpsc(q.new_root());
}

#[test]
fn spsc_impl() {
    let q: InlineLope<LockedDeque<u32>, DCBO, 2> = InlineLope::new();
    spsc(q.new_root());
}

#[test]
fn linearizable_impl() {
    let q: InlineLope<LockedDeque<u32>, DoubleCollect<DCBO>, 2> = InlineLope::new();
    linearizable(q.new_root());
}
