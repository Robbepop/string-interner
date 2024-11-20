#![cfg(feature = "backends")]

use super::{Backend, PhantomBackend};
use crate::{symbol::expect_valid_symbol, DefaultSymbol, Symbol};
use alloc::{string::String, vec::Vec};
use core::{iter::Enumerate, slice};

/// An interner backend that concatenates all interned string contents into one large
/// buffer and keeps track of string bounds in a separate [`Vec`].
/// 
/// Implementation is inspired by [CAD97's](https://github.com/CAD97)
/// [`strena`](https://github.com/CAD97/strena) crate.
///
/// ## Trade-offs
/// - **Advantages:**
///   - Separated length tracking allows fast iteration.
/// - **Disadvantages:**
///   - Many insertions separated by external allocations can cause the buffer to drift
///     far away (in memory) from `Vec` storing string ends, which impedes performance of
///     all interning operations.
///   - Resolving a symbol requires two heap lookups because data and length are stored in
///     separate containers.
///
/// ## Use Cases
/// This backend is good for storing fewer large strings and for general use.
///
/// Refer to the [comparison table][crate::_docs::comparison_table] for comparison with
/// other backends.
#[derive(Debug)]
pub struct StringBackend<'i, S: Symbol = DefaultSymbol> {
    ends: Vec<usize>,
    buffer: String,
    marker: PhantomBackend<'i, Self>,
}

/// Represents a `[from, to)` index into the `StringBackend` buffer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Span {
    from: usize,
    to: usize,
}

impl<'i, S> PartialEq for StringBackend<'i, S>
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

impl<'i, S> Eq for StringBackend<'i, S> where S: Symbol {}

impl<'i, S: Symbol> Clone for StringBackend<'i, S> {
    fn clone(&self) -> Self {
        Self {
            ends: self.ends.clone(),
            buffer: self.buffer.clone(),
            marker: Default::default(),
        }
    }
}

impl<'i, S: Symbol> Default for StringBackend<'i, S> {
    #[cfg_attr(feature = "inline-more", inline)]
    fn default() -> Self {
        Self {
            ends: Vec::default(),
            buffer: String::default(),
            marker: Default::default(),
        }
    }
}

impl<'i, S> StringBackend<'i, S>
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

    /// Pushes the given string into the buffer and returns its span.
    ///
    /// # Panics
    ///
    /// If the backend ran out of symbols.
    fn push_string(&mut self, string: &str) -> S {
        self.buffer.push_str(string);
        let to = self.buffer.len();
        let symbol = self.next_symbol();
        self.ends.push(to);
        symbol
    }
}

impl<'i, S> Backend<'i> for StringBackend<'i, S>
where
    S: Symbol,
{
    type Access<'l> = &'l str where Self: 'l;

    type Symbol = S;
    type Iter<'l>
        = Iter<'i, 'l, S>
    where
        Self: 'l;

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
    fn intern(&mut self, string: &str) -> Self::Symbol {
        self.push_string(string)
    }

    #[inline]
    fn resolve(&self, symbol: Self::Symbol) -> Option<&str> {
        self.symbol_to_span(symbol)
            .map(|span| self.span_to_str(span))
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

impl<'i, 'l, S> IntoIterator for &'l StringBackend<'i, S>
where
    S: Symbol + 'l,
{
    type Item = (S, &'l str);
    type IntoIter = Iter<'i, 'l, S>;

    #[cfg_attr(feature = "inline-more", inline)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct Iter<'i, 'l, S: Symbol> {
    backend: &'l StringBackend<'i, S>,
    start: usize,
    ends: Enumerate<slice::Iter<'l, usize>>,
}

impl<'i, 'l, S: Symbol> Iter<'i, 'l, S> {
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn new(backend: &'l StringBackend<'i, S>) -> Self {
        Self {
            backend,
            start: 0,
            ends: backend.ends.iter().enumerate(),
        }
    }
}

impl<'i, 'l, S> Iterator for Iter<'i, 'l, S>
where
    S: Symbol,
{
    type Item = (S, &'l str);

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
