#![cfg(feature = "backends")]

use super::Backend;
use crate::{
    compat::{
        String,
        Vec,
    },
    symbol::expect_valid_symbol,
    Symbol,
};
use core::{
    convert::TryInto,
    iter::Enumerate,
    marker::PhantomData,
    slice,
};

/// An interner backend that appends all interned strings together.
///
/// # Note
///
/// Implementation inspired by [CAD97's](https://github.com/CAD97) research
/// project [`strena`](https://github.com/CAD97/strena).
///
/// # Usage
///
/// - **Fill:** Efficiency of filling an empty string interner.
/// - **Resolve:** Efficiency of interned string look-up given a symbol.
/// - **Allocations:** The number of allocations performed by the backend.
/// - **Footprint:** The total heap memory consumed by the backend.
///
/// Rating varies between **bad**, **ok** and **good**.
///
/// | Scenario    |  Rating  |
/// |:------------|:--------:|
/// | Fill        | **good** |
/// | Resolve     | **bad**   |
/// | Allocations | **good** |
/// | Footprint   | **good**   |
/// | Supports `get_or_intern_static` | **no** |
/// | `Send` + `Sync` | **yes** |
#[derive(Debug)]
pub struct StringBackend<S> {
    ends: Vec<u32>,
    buffer: String,
    marker: PhantomData<fn() -> S>,
}

/// Represents a `[from, to)` index into the `StringBackend` buffer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Span {
    from: u32,
    to: u32,
}

impl<S> PartialEq for StringBackend<S>
where
    S: Symbol,
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
    fn next_symbol(&self) -> S {
        expect_valid_symbol(self.ends.len())
    }

    /// Returns the string associated to the span.
    fn span_to_str(&self, span: Span) -> &str {
        // SAFETY: - We convert a `String` into its underlying bytes and then
        //           directly reinterpret it as `&str` again which is safe.
        //         - Nothing mutates the string in between since this is a `&self`
        //           method.
        //         - The spans we use for `(start..end]` ranges are always
        //           constructed in accordance to valid utf8 byte ranges.
        unsafe {
            core::str::from_utf8_unchecked(
                &self.buffer.as_bytes()[(span.from as usize)..(span.to as usize)],
            )
        }
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
        let to = *self.ends.get_unchecked(index);
        let from = self.ends.get(index.wrapping_sub(1)).copied().unwrap_or(0);
        Span { from, to }
    }

    /// Pushes the given string into the buffer and returns its span.
    ///
    /// # Panics
    ///
    /// If the backend ran out of symbols.
    fn push_string(&mut self, string: &str) -> S {
        self.buffer.push_str(string);
        let to = self
            .buffer
            .as_bytes()
            .len()
            .try_into()
            .expect("ran out of symbols");
        let symbol = self.next_symbol();
        self.ends.push(to);
        symbol
    }
}

impl<S> Backend<S> for StringBackend<S>
where
    S: Symbol,
{
    #[cfg_attr(feature = "inline-more", inline)]
    fn with_capacity(cap: usize) -> Self {
        // According to google the approx. word length is 5.
        let default_word_len = 5;
        Self {
            ends: Vec::with_capacity(cap),
            buffer: String::with_capacity(cap * default_word_len),
            marker: Default::default(),
        }
    }

    #[inline]
    fn intern(&mut self, string: &str) -> S {
        self.push_string(string)
    }

    #[inline]
    fn resolve(&self, symbol: S) -> Option<&str> {
        self.symbol_to_span(symbol)
            .map(|span| self.span_to_str(span))
    }

    #[inline]
    unsafe fn resolve_unchecked(&self, symbol: S) -> &str {
        self.span_to_str(self.symbol_to_span_unchecked(symbol))
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
        Self::IntoIter::new(self)
    }
}

pub struct Iter<'a, S> {
    backend: &'a StringBackend<S>,
    start: u32,
    ends: Enumerate<slice::Iter<'a, u32>>,
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
