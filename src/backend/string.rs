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
/// - **Query:** Efficiency of interned string look-up given a symbol.
/// - **Memory:** The number of allocations and overall memory consumption.
///
/// Rating varies between **bad**, **ok** and **good**.
///
/// | Scenario | Rating |
/// |:---------|:------:|
/// | Fill     | **ok** |
/// | Query    | **ok** |
/// | Memory   | **good** |
/// | Supports `get_or_intern_static` | **no** |
#[derive(Debug, Clone)]
pub struct StringBackend<S> {
    spans: Vec<Span>,
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
        for (&lhs, &rhs) in self.spans.iter().zip(other.spans.iter()) {
            if self.span_to_str(lhs) != self.span_to_str(rhs) {
                return false
            }
        }
        true
    }
}

impl<S> Eq for StringBackend<S> where S: Symbol {}

impl<S> Default for StringBackend<S> {
    #[cfg_attr(feature = "inline-more", inline)]
    fn default() -> Self {
        Self {
            spans: Vec::default(),
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
        expect_valid_symbol(self.spans.len())
    }

    /// Returns the string associated to the span.
    fn span_to_str(&self, span: Span) -> &str {
        // SAFETY: We convert a `String` into its underlying bytes and then
        //         directly reinterpret it as `&str` again which is safe. Also
        //         nothing mutates the string in between since this is a `&self`
        //         method.
        unsafe {
            core::str::from_utf8_unchecked(
                &self.buffer.as_bytes()[(span.from as usize)..(span.to as usize)],
            )
        }
    }

    /// Returns the span for the given symbol if any.
    fn symbol_to_span(&self, symbol: S) -> Option<Span> {
        self.spans.get(symbol.to_usize()).copied()
    }

    /// Returns the span for the given symbol if any.
    unsafe fn symbol_to_span_unchecked(&self, symbol: S) -> Span {
        *self.spans.get_unchecked(symbol.to_usize())
    }

    /// Pushes the given span into the spans and returns its symbol.
    fn push_span(&mut self, span: Span) -> S {
        let symbol = self.next_symbol();
        self.spans.push(span);
        symbol
    }

    /// Pushes the given string into the buffer and returns its span.
    ///
    /// # Panics
    ///
    /// If the backend ran out of symbols.
    fn push_string(&mut self, string: &str) -> Span {
        let from = self.buffer.as_bytes().len();
        self.buffer.push_str(string);
        let to = self.buffer.as_bytes().len();
        Span {
            from: from.try_into().expect("ran out of symbols"),
            to: to.try_into().expect("ran out of symbols"),
        }
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
            spans: Vec::with_capacity(cap),
            buffer: String::with_capacity(cap * default_word_len),
            marker: Default::default(),
        }
    }

    #[inline]
    fn intern(&mut self, string: &str) -> S {
        let span = self.push_string(string);
        self.push_span(span)
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
    iter: Enumerate<slice::Iter<'a, Span>>,
}

impl<'a, S> Iter<'a, S> {
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn new(backend: &'a StringBackend<S>) -> Self {
        Self {
            backend,
            iter: backend.spans.iter().enumerate(),
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
            .map(|(id, &span)| (expect_valid_symbol(id), self.backend.span_to_str(span)))
    }
}
