#![cfg(feature = "backends")]

use super::Backend;
use crate::{
    compat::{Box, ToString, Vec},
    symbol::expect_valid_symbol,
    DefaultSymbol, Symbol,
};
use core::{iter::Enumerate, marker::PhantomData, slice};

/// A simple backend that stores a separate allocation for every interned string.
///
/// Use this if you can afford many small allocations and if you want to have
/// especially decent performance for look-ups when the string interner is
/// already filled to some extend.
///
/// # Usage Hint
///
/// Never actually use this interner backend since it only acts as a trivial baseline.
///
/// # Usage
///
/// - **Fill:** Efficiency of filling an empty string interner.
/// - **Resolve:** Efficiency of interned string look-up given a symbol.
/// - **Allocations:** The number of allocations performed by the backend.
/// - **Footprint:** The total heap memory consumed by the backend.
/// - **Contiguous:** True if the returned symbols have contiguous values.
///
/// Rating varies between **bad**, **ok**, **good** and **best**.
///
/// | Scenario    |  Rating  |
/// |:------------|:--------:|
/// | Fill        | **bad**  |
/// | Resolve     | **good** |
/// | Allocations | **bad**  |
/// | Footprint   | **bad**  |
/// | Supports `get_or_intern_static` | **no** |
/// | `Send` + `Sync` | **yes** |
/// | Contiguous  | **yes**  |
#[derive(Debug)]
pub struct SimpleBackend<S = DefaultSymbol> {
    strings: Vec<Box<str>>,
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

impl<S> Backend for SimpleBackend<S>
where
    S: Symbol,
{
    type Symbol = S;

    #[cfg_attr(feature = "inline-more", inline)]
    fn with_capacity(cap: usize) -> Self {
        Self {
            strings: Vec::with_capacity(cap),
            symbol_marker: Default::default(),
        }
    }

    #[inline]
    fn intern(&mut self, string: &str) -> Self::Symbol {
        let symbol = expect_valid_symbol(self.strings.len());
        let str = string.to_string().into_boxed_str();
        self.strings.push(str);
        symbol
    }

    fn shrink_to_fit(&mut self) {
        self.strings.shrink_to_fit()
    }

    #[inline]
    fn resolve(&self, symbol: Self::Symbol) -> Option<&str> {
        self.strings.get(symbol.to_usize()).map(|pinned| &**pinned)
    }

    #[inline]
    unsafe fn resolve_unchecked(&self, symbol: Self::Symbol) -> &str {
        // SAFETY: The function is marked unsafe so that the caller guarantees
        //         that required invariants are checked.
        unsafe { self.strings.get_unchecked(symbol.to_usize()) }
    }
}

impl<S> Clone for SimpleBackend<S> {
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
    iter: Enumerate<slice::Iter<'a, Box<str>>>,
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
