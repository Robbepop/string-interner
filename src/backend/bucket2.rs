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

/// An interner backend that reduces memory allocations by using string buckets.
///
/// # Note
///
/// Implementation inspired by matklad's blog post that can be found here:
/// https://matklad.github.io/2020/03/22/fast-simple-rust-interner.html
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
/// | Resolve     | **ok**   |
/// | Allocations | **good** |
/// | Footprint   | **ok**   |
/// | Supports `get_or_intern_static` | **yes** |
/// | `Send` + `Sync` | **yes** |
#[derive(Debug)]
pub struct BucketBackend<S> {
    spans: Vec<InternedSpan>,
    head: String,
    full: Vec<String>,
    marker: PhantomData<fn() -> S>,
}

/// Denotes a single interned string.
///
/// # Note
///
/// In order to reconstruct a string from this information two look-ups are
/// necessary since we only store the `end` position and not the `start`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InternedSpan {
    /// The bucket ID of the interned string.
    bucket_id: BucketId,
    /// The start index of the string within the bucket.
    start: u32,
    /// The end index of the string within the bucket.
    end: u32,
}

/// The identifier of a bucket.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BucketId {
    value: u32,
}

impl BucketId {
    /// Returns the `u32` identifier.
    pub fn get(self) -> u32 {
        self.value
    }
}

impl<S> Default for BucketBackend<S> {
    #[cfg_attr(feature = "inline-more", inline)]
    fn default() -> Self {
        Self {
            spans: Vec::new(),
            head: String::new(),
            full: Vec::new(),
            marker: Default::default(),
        }
    }
}

impl<S> Backend<S> for BucketBackend<S>
where
    S: Symbol,
{
    #[cfg_attr(feature = "inline-more", inline)]
    fn with_capacity(cap: usize) -> Self {
        Self {
            spans: Vec::with_capacity(cap),
            head: String::with_capacity(cap),
            full: Vec::new(),
            marker: Default::default(),
        }
    }

    #[inline]
    fn intern(&mut self, string: &str) -> S {
        let span = self.alloc(string);
        let symbol = self.push_span(span);
        symbol
    }

    #[inline]
    fn resolve(&self, symbol: S) -> Option<&str> {
        self.symbol_to_string(symbol)
    }

    #[inline]
    unsafe fn resolve_unchecked(&self, symbol: S) -> &str {
        self.symbol_to_string_unchecked(symbol)
    }
}

impl<S> BucketBackend<S>
where
    S: Symbol,
{
    /// Returns the next available symbol.
    fn next_symbol(&self) -> S {
        expect_valid_symbol(self.spans.len())
    }

    /// Returns the bucket ID of the current head bucket.
    fn head_bucket_id(&self) -> BucketId {
        BucketId {
            value: self.full.len() as u32,
        }
    }

    /// Returns the string associated with the given symbol if any.
    fn symbol_to_string(&self, symbol: S) -> Option<&str> {
        let span = self.spans.get(symbol.to_usize()).copied()?;
        self.span_to_string(span)
    }

    /// Returns the string associated with the given span if any.
    fn span_to_string(&self, span: InternedSpan) -> Option<&str> {
        let start = span.start as usize;
        let end = span.end as usize;
        let string = unsafe {
            core::str::from_utf8_unchecked(
                &self.bucket_id_to_bucket(span.bucket_id).as_bytes()[start..end],
            )
        };
        Some(string)
    }

    /// Returns the string associated with the given symbol if any.
    unsafe fn symbol_to_string_unchecked(&self, symbol: S) -> &str {
        let span = self.spans.get_unchecked(symbol.to_usize());
        let start = span.start as usize;
        let end = span.end as usize;
        core::str::from_utf8_unchecked(
            &self.bucket_id_to_bucket(span.bucket_id).as_bytes()[start..end],
        )
    }

    /// Returns the bucket for the given bucket ID.
    fn bucket_id_to_bucket(&self, bucket_id: BucketId) -> &str {
        debug_assert!(bucket_id.get() as usize <= self.full.len());
        let bucket_id = bucket_id.get() as usize;
        self.full.get(bucket_id).unwrap_or_else(|| &self.head)
    }

    /// Pushes the given interned span into the spans and returns its symbol.
    fn push_span(&mut self, span: InternedSpan) -> S {
        let symbol = self.next_symbol();
        self.spans.push(span);
        symbol
    }

    /// Interns a new string into the backend and returns a reference to it.
    fn alloc(&mut self, string: &str) -> InternedSpan {
        let cap = self.head.capacity();
        if cap < self.head.len() + string.len() {
            let new_cap = (usize::max(cap, string.len()) + 1).next_power_of_two();
            let new_head = String::with_capacity(new_cap);
            let old_head = core::mem::replace(&mut self.head, new_head);
            self.full.push(old_head);
        }
        let (start, end) = {
            let start = self.head.len() as u32;
            let len: u32 = string
                .len()
                .try_into()
                .expect("encountered too big string literal");
            let end = start + len;
            self.head.push_str(string);
            (start, end)
        };
        InternedSpan {
            bucket_id: self.head_bucket_id(),
            start,
            end,
        }
    }
}

impl<S> Clone for BucketBackend<S>
where
    S: Symbol,
{
    fn clone(&self) -> Self {
        Self {
            spans: self.spans.clone(),
            head: self.head.clone(),
            full: self.full.clone(),
            marker: Default::default(),
        }
    }
}

impl<S> Eq for BucketBackend<S> where S: Symbol {}

impl<S> PartialEq for BucketBackend<S>
where
    S: Symbol,
{
    #[cfg_attr(feature = "inline-more", inline)]
    fn eq(&self, other: &Self) -> bool {
        let self_symbols = self
            .spans
            .iter()
            .enumerate()
            .map(|(index, _)| expect_valid_symbol(index))
            .map(|symbol| {
                self.symbol_to_string(symbol)
                    .expect("encountered invalid symbol")
            });
        let other_symbols = other
            .spans
            .iter()
            .enumerate()
            .map(|(index, _)| expect_valid_symbol(index))
            .map(|symbol| {
                other
                    .symbol_to_string(symbol)
                    .expect("encountered invalid symbol")
            });
        for (lhs, rhs) in self_symbols.zip(other_symbols) {
            if lhs != rhs {
                return false
            }
        }
        true
    }
}

impl<'a, S> IntoIterator for &'a BucketBackend<S>
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
    backend: &'a BucketBackend<S>,
    iter: Enumerate<slice::Iter<'a, InternedSpan>>,
    symbol_marker: PhantomData<fn() -> S>,
}

impl<'a, S> Iter<'a, S> {
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn new(backend: &'a BucketBackend<S>) -> Self {
        Self {
            backend,
            iter: backend.spans.iter().enumerate(),
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
        self.iter.next().map(|(id, &span)| {
            let symbol = expect_valid_symbol(id);
            let string = self
                .backend
                .span_to_string(span)
                .expect("encountered invalid span");
            (symbol, string)
        })
    }
}
