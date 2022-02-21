#![cfg(feature = "backends")]

//! An interner backend that accumulates all interned string contents into one string.
//!
//! # Note
//!
//! Implementation inspired by [CAD97's](https://github.com/CAD97) research
//! project [`strena`](https://github.com/CAD97/strena).
//!
//! # Usage Hint
//!
//! Use this backend if runtime performance is what matters most to you.
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
//! | Fill        | **good** |
//! | Resolve     | **ok**   |
//! | Allocations | **good** |
//! | Footprint   | **good** |
//! | Supports `get_or_intern_static` | **no** |
//! | `Send` + `Sync` | **yes** |
//! | Contiguous  | **yes**  |

use len_trait::{
    CapacityMut,
    WithCapacity,
};

use super::{
    Backend,
    Internable,
};
use crate::{
    compat::Vec,
    symbol::expect_valid_symbol,
    DefaultSymbol,
    Symbol,
};
use core::{
    iter::Enumerate,
    marker::PhantomData,
    slice,
};

/// An interner backend that accumulates all interned string contents into one string.
///
/// See the [module-level documentation](self) for more.
#[derive(Debug)]
pub struct StringBackend<S = str, Sym = DefaultSymbol>
where
    S: ?Sized + Internable,
{
    ends: Vec<usize>,
    buffer: S::Container,
    marker: PhantomData<fn(&S) -> Sym>,
}

/// Represents a `[from, to)` index into the `StringBackend` buffer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Span {
    from: usize,
    to: usize,
}

impl<S, Sym> PartialEq for StringBackend<S, Sym>
where
    Sym: Symbol,
    S: ?Sized + Internable + PartialEq,
    S::Container: AsRef<S> + for<'e> Extend<&'e S>,
{
    fn eq(&self, other: &Self) -> bool {
        if self.ends.len() != other.ends.len() {
            return false
        }
        for ((_, lhs), (_, rhs)) in self.into_iter().zip(other) {
            if lhs != rhs {
                return false
            }
        }
        true
    }
}

impl<S, Sym> Eq for StringBackend<S, Sym>
where
    Sym: Symbol,
    S: ?Sized + Internable + Eq,
    S::Container: AsRef<S> + for<'e> Extend<&'e S>,
{
}

impl<S, Sym> Clone for StringBackend<S, Sym>
where
    S: ?Sized + Internable,
    S::Container: Clone,
{
    fn clone(&self) -> Self {
        Self {
            ends: self.ends.clone(),
            buffer: self.buffer.clone(),
            marker: Default::default(),
        }
    }
}

impl<S, Sym> Default for StringBackend<S, Sym>
where
    S: ?Sized + Internable,
    S::Container: Default,
{
    #[cfg_attr(feature = "inline-more", inline)]
    fn default() -> Self {
        Self {
            ends: Vec::default(),
            buffer: S::Container::default(),
            marker: Default::default(),
        }
    }
}

impl<S, Sym> StringBackend<S, Sym>
where
    Sym: Symbol,
    S: ?Sized + Internable,
{
    /// Returns the next available symbol.
    fn next_symbol(&self) -> Sym {
        expect_valid_symbol(self.ends.len())
    }

    /// Returns the string associated to the span.
    fn span_to_str(&self, span: Span) -> &S {
        S::from_slice(
            &self.buffer.as_ref().to_slice()[(span.from as usize)..(span.to as usize)],
        )
    }

    /// Returns the span for the given symbol if any.
    fn symbol_to_span(&self, symbol: Sym) -> Option<Span> {
        let index = symbol.to_usize();
        self.ends.get(index).copied().map(|to| {
            let from = self.ends.get(index.wrapping_sub(1)).copied().unwrap_or(0);
            Span { from, to }
        })
    }

    /// Returns the span for the given symbol if any.
    unsafe fn symbol_to_span_unchecked(&self, symbol: Sym) -> Span {
        let index = symbol.to_usize();
        // SAFETY: The function is marked unsafe so that the caller guarantees
        //         that required invariants are checked.
        let to = unsafe { *self.ends.get_unchecked(index) };
        let from = self.ends.get(index.wrapping_sub(1)).copied().unwrap_or(0);
        Span { from, to }
    }

    /// Pushes the given string into the buffer and returns its span.
    ///
    /// # Panics
    ///
    /// If the backend ran out of symbols.
    fn push_string(&mut self, string: &S) -> Sym {
        S::push_str(&mut self.buffer, string);
        let to = self.buffer.as_ref().to_slice().len();
        let symbol = self.next_symbol();
        self.ends.push(to);
        symbol
    }
}

impl<S, Sym> Backend for StringBackend<S, Sym>
where
    Sym: Symbol,
    S: ?Sized + Internable,
{
    type Str = S;
    type Symbol = Sym;

    #[cfg_attr(feature = "inline-more", inline)]
    fn with_capacity(cap: usize) -> Self {
        // According to google the approx. word length is 5.
        let default_word_len = 5;
        Self {
            ends: Vec::with_capacity(cap),
            buffer: <S::Container as WithCapacity>::with_capacity(cap * default_word_len),
            marker: Default::default(),
        }
    }

    #[inline]
    fn intern(&mut self, string: &S) -> Self::Symbol {
        self.push_string(string)
    }

    #[inline]
    fn resolve(&self, symbol: Self::Symbol) -> Option<&S> {
        self.symbol_to_span(symbol)
            .map(|span| self.span_to_str(span))
    }

    fn shrink_to_fit(&mut self) {
        self.ends.shrink_to_fit();
        self.buffer.shrink_to_fit();
    }

    #[inline]
    unsafe fn resolve_unchecked(&self, symbol: Self::Symbol) -> &S {
        // SAFETY: The function is marked unsafe so that the caller guarantees
        //         that required invariants are checked.
        unsafe { self.span_to_str(self.symbol_to_span_unchecked(symbol)) }
    }
}

impl<'a, S, Sym> IntoIterator for &'a StringBackend<S, Sym>
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

/// Iterator for a [`StringBackend`](crate::backend::string::StringBackend)
/// that returns all of its interned strings.
pub struct Iter<'a, S, Sym>
where
    S: ?Sized + Internable,
{
    backend: &'a StringBackend<S, Sym>,
    start: usize,
    ends: Enumerate<slice::Iter<'a, usize>>,
}

impl<'a, S, Sym> Iter<'a, S, Sym>
where
    S: ?Sized + Internable,
{
    #[cfg_attr(feature = "inline-more", inline)]
    pub(super) fn new(backend: &'a StringBackend<S, Sym>) -> Self {
        Self {
            backend,
            start: 0,
            ends: backend.ends.iter().enumerate(),
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
        self.ends.size_hint()
    }

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.ends.next().map(|(id, &to)| {
            let from = core::mem::replace(&mut self.start, to);
            (
                expect_valid_symbol(id),
                self.backend.span_to_str(Span { from, to }),
            )
        })
    }
}
