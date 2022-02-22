#![cfg(feature = "backends")]

//! A simple backend that stores a separate allocation for every interned string.
//!
//! Use this if you can afford many small allocations and if you want to have
//! especially decent performance for look-ups when the string interner is
//! already filled to some extend.
//!
//! # Usage Hint
//!
//! Never actually use this interner backend since it only acts as a trivial baseline.
//!
//! # Usage
//!
//! - **Fill:** Efficiency of filling an empty string interner.
//! - **Resolve:** Efficiency of interned string look-up given a symbol.
//! - **Allocations:** The number of allocations performed by the backend.
//! - **Footprint:** The total heap memory consumed by the backend.
//! - **Contiguous:** True if the returned symbols have contiguous values.
//!
//! Rating varies between **bad**, **ok**, **good** and **best**.
//!
//! | Scenario    |  Rating  |
//! |:------------|:--------:|
//! | Fill        | **bad**  |
//! | Resolve     | **good** |
//! | Allocations | **bad**  |
//! | Footprint   | **bad**  |
//! | Supports `get_or_intern_static` | **no** |
//! | `Send` + `Sync` | **yes** |
//! | Contiguous  | **yes**  |

use super::{
    Backend,
    Internable,
};
use crate::{
    compat::{
        Box,
        Vec,
    },
    symbol::expect_valid_symbol,
    DefaultSymbol,
    Symbol,
};
use core::{
    iter::Enumerate,
    marker::PhantomData,
    slice,
};

/// A simple backend that stores a separate allocation for every interned string.
///
/// See the [module-level documentation](self) for more.
#[derive(Debug)]
pub struct SimpleBackend<S = str, Sym = DefaultSymbol>
where
    S: ?Sized + Internable,
{
    strings: Vec<Box<S>>,
    symbol_marker: PhantomData<fn() -> Sym>,
}

impl<S, Sym> Default for SimpleBackend<S, Sym>
where
    S: ?Sized + Internable,
{
    #[cfg_attr(feature = "inline-more", inline)]
    fn default() -> Self {
        Self {
            strings: Vec::new(),
            symbol_marker: Default::default(),
        }
    }
}

impl<S, Sym> Backend for SimpleBackend<S, Sym>
where
    Sym: Symbol,
    S: ?Sized + Internable,
{
    type Str = S;
    type Symbol = Sym;

    #[cfg_attr(feature = "inline-more", inline)]
    fn with_capacity(cap: usize) -> Self {
        Self {
            strings: Vec::with_capacity(cap),
            symbol_marker: Default::default(),
        }
    }

    #[inline]
    fn intern(&mut self, string: &S) -> Self::Symbol
where {
        let symbol = expect_valid_symbol(self.strings.len());
        self.strings.push(string.to_boxed());
        symbol
    }

    fn shrink_to_fit(&mut self) {
        self.strings.shrink_to_fit()
    }

    #[inline]
    fn resolve(&self, symbol: Self::Symbol) -> Option<&S> {
        self.strings.get(symbol.to_usize()).map(|pinned| &**pinned)
    }

    #[inline]
    unsafe fn resolve_unchecked(&self, symbol: Self::Symbol) -> &S {
        // SAFETY: The function is marked unsafe so that the caller guarantees
        //         that required invariants are checked.
        unsafe { self.strings.get_unchecked(symbol.to_usize()) }
    }
}

impl<S, Sym> Clone for SimpleBackend<S, Sym>
where
    S: ?Sized + Internable,
{
    #[cfg_attr(feature = "inline-more", inline)]
    fn clone(&self) -> Self {
        Self {
            strings: self.strings.iter().map(|s| s.as_ref().to_boxed()).collect(),
            symbol_marker: Default::default(),
        }
    }
}

impl<S, Sym> Eq for SimpleBackend<S, Sym>
where
    Sym: Symbol,
    S: ?Sized + Internable + Eq,
{
}

impl<S, Sym> PartialEq for SimpleBackend<S, Sym>
where
    Sym: Symbol,
    S: ?Sized + Internable + PartialEq,
{
    #[cfg_attr(feature = "inline-more", inline)]
    fn eq(&self, other: &Self) -> bool {
        self.strings == other.strings
    }
}

impl<'a, S, Sym> IntoIterator for &'a SimpleBackend<S, Sym>
where
    Sym: Symbol,
    S: ?Sized + Internable,
{
    type Item = (Sym, &'a S);
    type IntoIter = Iter<'a, S, Sym>;

    #[cfg_attr(feature = "inline-more", inline)]
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

/// Iterator for a [`SimpleBackend`](crate::backend::simple::SimpleBackend)
/// that returns all of its interned strings.
pub struct Iter<'a, S, Sym>
where
    S: ?Sized + Internable,
{
    iter: Enumerate<slice::Iter<'a, Box<S>>>,
    symbol_marker: PhantomData<fn() -> Sym>,
}

impl<'a, S, Sym> Iter<'a, S, Sym>
where
    S: ?Sized + Internable,
{
    #[cfg_attr(feature = "inline-more", inline)]
    pub(super) fn new(backend: &'a SimpleBackend<S, Sym>) -> Self {
        Self {
            iter: backend.strings.iter().enumerate(),
            symbol_marker: Default::default(),
        }
    }
}

impl<'a, S, Sym> Iterator for Iter<'a, S, Sym>
where
    Sym: Symbol,
    S: ?Sized + Internable,
{
    type Item = (Sym, &'a S);

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
