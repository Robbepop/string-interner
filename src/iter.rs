//! Iterator implementations for the [`StringInterner`].

use crate::{
    compat::{
        Box,
        String,
    },
    symbol::expect_valid_symbol,
    StringInterner,
    Symbol,
};
use core::{
    hash::BuildHasher,
    iter::{
        Enumerate,
        IntoIterator,
        Iterator,
    },
    marker,
    pin::Pin,
    slice,
};

/// Iterator over the pairs of associated symbols and interned strings for a `StringInterner`.
pub struct Iter<'a, S> {
    iter: Enumerate<slice::Iter<'a, Pin<Box<str>>>>,
    mark: marker::PhantomData<S>,
}

impl<'a, S> Iter<'a, S>
where
    S: Symbol + 'a,
{
    /// Creates a new iterator for the given StringIterator over pairs of
    /// symbols and their associated interned string.
    #[inline]
    pub(crate) fn new<H>(interner: &'a StringInterner<S, H>) -> Self
    where
        H: BuildHasher,
    {
        Self {
            iter: interner.values.iter().enumerate(),
            mark: marker::PhantomData,
        }
    }
}

impl<'a, S> Iterator for Iter<'a, S>
where
    S: Symbol + 'a,
{
    type Item = (S, &'a str);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(num, boxed_str)| {
            (expect_valid_symbol::<S>(num), boxed_str.as_ref().get_ref())
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
/// Iterator over the interned strings of a `StringInterner`.
pub struct Values<'a, S>
where
    S: Symbol + 'a,
{
    iter: slice::Iter<'a, Pin<Box<str>>>,
    mark: marker::PhantomData<S>,
}

impl<'a, S> Values<'a, S>
where
    S: Symbol + 'a,
{
    /// Creates a new iterator for the given StringIterator over its interned strings.
    #[inline]
    pub(crate) fn new<H>(interner: &'a StringInterner<S, H>) -> Self
    where
        H: BuildHasher,
    {
        Self {
            iter: interner.values.iter(),
            mark: marker::PhantomData,
        }
    }
}

impl<'a, S> Iterator for Values<'a, S>
where
    S: Symbol + 'a,
{
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|boxed_str| boxed_str.as_ref().get_ref())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// Iterator over the pairs of associated symbol and strings.
///
/// Consumes the `StringInterner` upon usage.
pub struct IntoIter<S>
where
    S: Symbol,
{
    iter: Enumerate<crate::compat::vec::IntoIter<Pin<Box<str>>>>,
    mark: marker::PhantomData<S>,
}

impl<S> IntoIter<S>
where
    S: Symbol,
{
    /// Creates a new iterator for the given StringIterator over pairs of
    /// symbols and their associated interned string.
    #[inline]
    pub(crate) fn new<H>(interner: StringInterner<S, H>) -> Self
    where
        H: BuildHasher,
    {
        Self {
            iter: interner.values.into_iter().enumerate(),
            mark: marker::PhantomData,
        }
    }
}

impl<S> Iterator for IntoIter<S>
where
    S: Symbol,
{
    type Item = (S, String);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(num, boxed_str)| {
            (
                expect_valid_symbol::<S>(num),
                Pin::into_inner(boxed_str).into_string(),
            )
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
