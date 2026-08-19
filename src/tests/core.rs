use crate::{
    InlineBandit,
    strategy::{DCBO, DoubleCollect},
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
    let q: InlineBandit<LockedDeque<u32>, DCBO, 2> = InlineBandit::new();
    smoke(q.buy_in());
}

#[test]
fn smoke_long_impl() {
    let q: InlineBandit<LockedDeque<u32>, DCBO, 2> = InlineBandit::new();
    smoke_long(q.buy_in());
}

#[test]
fn force_push_impl() {
    let q: InlineBandit<LockedDeque<u32>, DCBO, 2> = InlineBandit::new();
    force_push(q.buy_in());
}

#[test]
fn len_impl() {
    let q: InlineBandit<LockedDeque<u32>, DCBO, 2> = InlineBandit::new();
    len(q.buy_in());
}

#[test]
fn len_empty_full_impl() {
    let q: InlineBandit<LockedDeque<()>, DCBO, 2, 1> = InlineBandit::new();
    len_empty_full(q.buy_in());
}

#[test]
fn mpmc_impl() {
    let q: InlineBandit<LockedDeque<u32>, DCBO, 2> = InlineBandit::new();
    mpmc(q.buy_in());
}

#[test]
fn mpmc_ring_buffer_impl() {
    let q: InlineBandit<LockedDeque<u32>, DCBO, 2> = InlineBandit::new();
    mpmc_ring_buffer(q.buy_in());
}

#[test]
fn mpsc_impl() {
    let q: InlineBandit<LockedDeque<u32>, DCBO, 2> = InlineBandit::new();
    mpsc(q.buy_in());
}

#[test]
fn spsc_impl() {
    let q: InlineBandit<LockedDeque<u32>, DCBO, 2> = InlineBandit::new();
    spsc(q.buy_in());
}

#[test]
fn linearizable_impl() {
    let q: InlineBandit<LockedDeque<u32>, DoubleCollect<DCBO>, 2> = InlineBandit::new();
    linearizable(q.buy_in());
}
