//! Useful components and helper traits for implementing the [`Collection`] trait.

use core::marker::PhantomData;

use crate::{Collection, Signature};

/// A method that takes a `T` by value and returns it on an error.
pub struct ValueSig<T>(PhantomData<T>);

impl<T> Signature for ValueSig<T> {
    type Error<'input, 'arm>
        = T
    where
        Self: 'arm;
    type Input<'a> = T;
    type Output<'input, 'arm>
        = ()
    where
        Self: 'arm;
}

/// A method that returns a `T` on success.
pub struct PopSig<T>(PhantomData<T>);

impl<T> Signature for PopSig<T> {
    type Error<'input, 'arm>
        = ()
    where
        Self: 'arm;
    type Input<'a> = ();
    type Output<'input, 'arm>
        = T
    where
        Self: 'arm;
}

/// A method that takes no arguments and returns unit values.
pub struct Unit;

impl Signature for Unit {
    type Error<'input, 'arm>
        = ()
    where
        Self: 'arm;
    type Input<'a> = ();
    type Output<'input, 'arm>
        = ()
    where
        Self: 'arm;
}

/// A collection which one pushes items to and pops items from
pub trait PushPopCollection {
    /// The type of item stored in this collection
    type Item;

    /// Pushes an item to the collection.
    ///
    /// Returns the item on an error.
    fn push(&self, item: Self::Item) -> Result<(), Self::Item>;
    /// Tries to pop an item from the collection.
    fn pop(&self) -> Option<Self::Item>;
    /// The length ofthe collection.
    fn len(&self) -> usize;
    /// The capacity of the collection
    #[inline]
    fn capacity(&self) -> usize {
        usize::MAX
    }

    /// Is the collection empty?
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<Q> Collection for Q
where
    Q: PushPopCollection,
{
    type OfferSignature = ValueSig<Q::Item>;
    type PollSignature = PopSig<Q::Item>;

    #[inline]
    fn offer<'input, 'arm>(
        &'arm self,
        item: <Self::OfferSignature as Signature>::Input<'input>,
    ) -> Result<
        <Self::OfferSignature as Signature>::Output<'input, 'arm>,
        <Self::OfferSignature as Signature>::Error<'input, 'arm>,
    > {
        self.push(item)
    }

    #[inline]
    fn poll<'input, 'arm>(
        &'arm self,
        _input: <Self::PollSignature as Signature>::Input<'input>,
    ) -> Result<
        <Self::PollSignature as Signature>::Output<'input, 'arm>,
        <Self::PollSignature as Signature>::Error<'input, 'arm>,
    > {
        self.pop().ok_or(())
    }

    #[inline]
    fn len(&self) -> usize {
        PushPopCollection::len(self)
    }

    #[inline]
    fn capacity(&self) -> usize {
        PushPopCollection::capacity(self)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        PushPopCollection::is_empty(self)
    }
}
