#![cfg(feature = "backends")]

use super::Backend;
use crate::{symbol::expect_valid_symbol, DefaultSymbol, Error, Result, Symbol};
use alloc::{string::String, vec::Vec};
use core::{iter::Enumerate, marker::PhantomData, slice};

/// An interner backend that accumulates all interned string contents into one string.
///
/// # Note
///
/// Implementation inspired by [CAD97's](https://github.com/CAD97) research
/// project [`strena`](https://github.com/CAD97/strena).
///
/// # Usage Hint
///
/// Use this backend if runtime performance is what matters most to you.
///
/// # Usage
///
/// - **Fill:** Efficiency of filling an empty string interner.
/// - **Resolve:** Efficiency of interned string look-up given a symbol.
/// - **Allocations:** The number of allocations performed by the backend.
/// - **Footprint:** The total heap memory consumed by the backend.
/// - **Contiguous:** True if the returned symbols have contiguous values.
/// - **Iteration:** Efficiency of iterating over the interned strings.
///
/// Rating varies between **bad**, **ok**, **good** and **best**.
///
/// | Scenario    |  Rating  |
/// |:------------|:--------:|
/// | Fill        | **good** |
/// | Resolve     | **ok**   |
/// | Allocations | **good** |
/// | Footprint   | **good** |
/// | Supports `get_or_intern_static` | **no** |
/// | `Send` + `Sync` | **yes** |
/// | Contiguous  | **yes**  |
/// | Iteration   | **good** |
#[derive(Debug)]
pub struct StringBackend<S = DefaultSymbol> {
    ends: Vec<usize>,
    buffer: String,
    marker: PhantomData<fn() -> S>,
}

/// Represents a `[from, to)` index into the `StringBackend` buffer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Span {
    from: usize,
    to: usize,
}

impl<S> PartialEq for StringBackend<S>
where
    S: Symbol,
{
    fn eq(&self, other: &Self) -> bool {
        if self.ends.len() != other.ends.len() {
            return false;
        }
        for ((_, lhs), (_, rhs)) in self.into_iter().zip(other) {
            if lhs != rhs {
                return false;
            }
        }
        true
    }
}

impl<S> Eq for StringBackend<S> where S: Symbol {}

impl<S> Clone for StringBackend<S> {
    fn clone(&self) -> Self {
        Self {
            ends: self.ends.clone(),
            buffer: self.buffer.clone(),
            marker: Default::default(),
        }
    }
}

impl<S> Default for StringBackend<S> {
    #[cfg_attr(feature = "inline-more", inline)]
    fn default() -> Self {
        Self {
            ends: Vec::default(),
            buffer: String::default(),
            marker: Default::default(),
        }
    }
}

impl<S> StringBackend<S>
where
    S: Symbol,
{
    /// Returns the next available symbol.
    #[inline]
    fn try_next_symbol(&self) -> Result<S> {
        S::try_from_usize(self.ends.len()).ok_or(Error::OutOfSymbols)
    }

    /// Returns the string associated to the span.
    fn span_to_str(&self, span: Span) -> &str {
        // SAFETY: - We convert a `String` into its underlying bytes and then
        //           directly reinterpret it as `&str` again which is safe.
        //         - Nothing mutates the string in between since this is a `&self`
        //           method.
        //         - The spans we use for `(start..end]` ranges are always
        //           constructed in accordance to valid utf8 byte ranges.
        unsafe { core::str::from_utf8_unchecked(&self.buffer.as_bytes()[span.from..span.to]) }
    }

    /// Returns the span for the given symbol if any.
    fn symbol_to_span(&self, symbol: S) -> Option<Span> {
        let index = symbol.to_usize();
        self.ends.get(index).copied().map(|to| {
            let from = self.ends.get(index.wrapping_sub(1)).copied().unwrap_or(0);
            Span { from, to }
        })
    }

    /// Returns the span for the given symbol if any.
    unsafe fn symbol_to_span_unchecked(&self, symbol: S) -> Span {
        let index = symbol.to_usize();
        // SAFETY: The function is marked unsafe so that the caller guarantees
        //         that required invariants are checked.
        let to = unsafe { *self.ends.get_unchecked(index) };
        let from = self.ends.get(index.wrapping_sub(1)).copied().unwrap_or(0);
        Span { from, to }
    }

    /// Pushes the given string into the buffer and returns its span on success.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the backend ran out of symbols or memory.
    fn try_push_string(&mut self, string: &str) -> Result<S> {
        // reserve required capacity ahead of pushing a string
        self.buffer.try_reserve(string.len())?;
        // The following cannot panic, we already reserved enough capacity for a string.
        self.buffer.push_str(string);
        let to = self.buffer.as_bytes().len();
        let symbol = self.try_next_symbol()?;
        // FIXME: vec_push_within_capacity #100486, replace the following with:
        //
        // if let Err(value) = self.ends.push_within_capacity(to) {
        //     self.ends.try_reserve(1)?;
        //     // this cannot fail, the previous line either returned or added at least 1 free slot
        //     let _ = self.ends.push_within_capacity(value);
        // }
        self.ends.try_reserve(1)?;
        self.ends.push(to);

        Ok(symbol)
    }
}

// According to google the approx. word length is 5.
const DEFAULT_WORD_LEN: usize = 5;

impl<S> Backend for StringBackend<S>
where
    S: Symbol,
{
    type Symbol = S;
    type Iter<'a> = Iter<'a, S>
    where
        Self: 'a;

    #[cfg_attr(feature = "inline-more", inline)]
    fn with_capacity(cap: usize) -> Self {
        Self {
            ends: Vec::with_capacity(cap),
            buffer: String::with_capacity(cap * DEFAULT_WORD_LEN),
            marker: Default::default(),
        }
    }

    #[inline]
    fn try_intern(&mut self, string: &str) -> Result<Self::Symbol> {
        self.try_push_string(string)
    }

    #[inline]
    fn resolve(&self, symbol: Self::Symbol) -> Option<&str> {
        self.symbol_to_span(symbol)
            .map(|span| self.span_to_str(span))
    }

    fn try_reserve(&mut self, additional: usize) -> Result<()> {
        self.ends.try_reserve(additional)?;
        self.buffer.try_reserve(additional * DEFAULT_WORD_LEN)?;
        Ok(())
    }

    fn shrink_to_fit(&mut self) {
        self.ends.shrink_to_fit();
        self.buffer.shrink_to_fit();
    }

    #[inline]
    unsafe fn resolve_unchecked(&self, symbol: Self::Symbol) -> &str {
        // SAFETY: The function is marked unsafe so that the caller guarantees
        //         that required invariants are checked.
        unsafe { self.span_to_str(self.symbol_to_span_unchecked(symbol)) }
    }

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        Iter::new(self)
    }
}

impl<'a, S> IntoIterator for &'a StringBackend<S>
where
    S: Symbol,
{
    type Item = (S, &'a str);
    type IntoIter = Iter<'a, S>;

    #[cfg_attr(feature = "inline-more", inline)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct Iter<'a, S> {
    backend: &'a StringBackend<S>,
    start: usize,
    ends: Enumerate<slice::Iter<'a, usize>>,
}

impl<'a, S> Iter<'a, S> {
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn new(backend: &'a StringBackend<S>) -> Self {
        Self {
            backend,
            start: 0,
            ends: backend.ends.iter().enumerate(),
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
