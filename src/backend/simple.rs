use super::Backend;
use crate::{
    compat::{
        Box,
        ToString,
        Vec,
    },
    symbol::expect_valid_symbol,
    Symbol,
};
use core::{
    iter::Enumerate,
    marker::PhantomData,
    pin::Pin,
    slice,
};

/// A simple backend that stores a separate allocation for every interned string.
///
/// Use this if you can afford many small allocations and if you want to have
/// especially decent performance for look-ups when the string interner is
/// already filled to some extend.
///
/// # Usage
///
/// - **Fill:** Efficiency of filling an empty string interner.
/// - **Query:** Efficiency of interned string look-up given a symbol.
/// - **Memory:** The number of allocations and overall memory consumption.
///
/// Rating varies between **bad**, **ok** and **good**.
///
/// | Scenario | Rating |
/// |:---------|:------:|
/// | Fill     | **bad** |
/// | Query    | **good** |
/// | Memory   | **bad:** many small allocations |
/// | Supports `get_or_intern_static` | **no** |
#[derive(Debug)]
pub struct SimpleBackend<S> {
    strings: Vec<Pin<Box<str>>>,
    symbol_marker: PhantomData<fn() -> S>,
}

impl<S> Default for SimpleBackend<S> {
    #[cfg_attr(feature = "inline-more", inline)]
    fn default() -> Self {
        Self {
            strings: Vec::new(),
            symbol_marker: Default::default(),
        }
    }
}

impl<S> Backend<S> for SimpleBackend<S>
where
    S: Symbol,
{
    #[cfg_attr(feature = "inline-more", inline)]
    fn with_capacity(cap: usize) -> Self {
        Self {
            strings: Vec::with_capacity(cap),
            symbol_marker: Default::default(),
        }
    }

    #[inline]
    fn intern(&mut self, string: &str) -> S {
        let symbol = expect_valid_symbol(self.strings.len());
        let str = Pin::new(string.to_string().into_boxed_str());
        self.strings.push(str);
        symbol
    }

    #[inline]
    fn resolve(&self, symbol: S) -> Option<&str> {
        self.strings.get(symbol.to_usize()).map(|pinned| &**pinned)
    }

    #[inline]
    unsafe fn resolve_unchecked(&self, symbol: S) -> &str {
        self.strings.get_unchecked(symbol.to_usize())
    }
}

impl<S> Clone for SimpleBackend<S>
where
    S: Symbol,
{
    #[cfg_attr(feature = "inline-more", inline)]
    fn clone(&self) -> Self {
        Self {
            strings: self.strings.clone(),
            symbol_marker: Default::default(),
        }
    }
}

impl<S> Eq for SimpleBackend<S> where S: Symbol {}

impl<S> PartialEq for SimpleBackend<S>
where
    S: Symbol,
{
    #[cfg_attr(feature = "inline-more", inline)]
    fn eq(&self, other: &Self) -> bool {
        self.strings == other.strings
    }
}

impl<'a, S> IntoIterator for &'a SimpleBackend<S>
where
    S: Symbol,
{
    type Item = (S, &'a str);
    type IntoIter = Iter<'a, S>;

    #[cfg_attr(feature = "inline-more", inline)]
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

pub struct Iter<'a, S> {
    iter: Enumerate<slice::Iter<'a, Pin<Box<str>>>>,
    symbol_marker: PhantomData<fn() -> S>,
}

impl<'a, S> Iter<'a, S> {
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn new(backend: &'a SimpleBackend<S>) -> Self {
        Self {
            iter: backend.strings.iter().enumerate(),
            symbol_marker: Default::default(),
        }
    }
}

impl<'a, S> Iterator for Iter<'a, S>
where
    S: Symbol,
{
    type Item = (S, &'a str);

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(id, pinned)| (expect_valid_symbol(id), &**pinned))
    }
}
