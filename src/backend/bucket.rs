use super::Backend;
use crate::{
    backend::InternedStr,
    compat::{
        String,
        Vec,
    },
    symbol::expect_valid_symbol,
    Symbol,
};
use core::{
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
/// - **Query:** Efficiency of interned string look-up given a symbol.
/// - **Memory:** The number of allocations and overall memory consumption.
///
/// Rating varies between **bad**, **ok** and **good**.
///
/// | Scenario | Rating |
/// |:---------|:------:|
/// | Fill     | **good** |
/// | Query    | **ok** |
/// | Memory   | **good** |
/// | Supports `get_or_intern_static` | **yes** |
#[derive(Debug)]
pub struct BucketBackend<S> {
    spans: Vec<InternedStr>,
    head: String,
    full: Vec<String>,
    marker: PhantomData<fn() -> S>,
}

impl<S> Default for BucketBackend<S> {
    #[inline]
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
    #[inline]
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
        // SAFETY: This is safe because we never hand out the returned
        //         interned string instance to the outside and only operate
        //         on it within this backend.
        let interned = unsafe { self.alloc(string) };
        self.push_span(interned)
    }

    #[inline]
    fn intern_static(&mut self, string: &'static str) -> S {
        let interned = InternedStr::new(string);
        self.push_span(interned)
    }

    #[inline]
    fn resolve(&self, symbol: S) -> Option<&str> {
        self.spans
            .get(symbol.to_usize())
            .map(|interned| interned.as_str())
    }

    #[inline]
    unsafe fn resolve_unchecked(&self, symbol: S) -> &str {
        self.spans.get_unchecked(symbol.to_usize()).as_str()
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

    /// Pushes the given interned string into the spans and returns its symbol.
    fn push_span(&mut self, interned: InternedStr) -> S {
        let symbol = self.next_symbol();
        self.spans.push(interned.into());
        symbol
    }

    /// Interns a new string into the backend and returns a reference to it.
    unsafe fn alloc(&mut self, string: &str) -> InternedStr {
        let cap = self.head.capacity();
        if cap < self.head.len() + string.len() {
            let new_cap = (usize::max(cap, string.len()) + 1).next_power_of_two();
            let new_head = String::with_capacity(new_cap);
            let old_head = core::mem::replace(&mut self.head, new_head);
            self.full.push(old_head);
        }
        let interned = {
            let start = self.head.len();
            self.head.push_str(string);
            &self.head[start..start + string.len()]
        };
        InternedStr::new(interned)
    }
}

impl<S> Clone for BucketBackend<S>
where
    S: Symbol,
{
    fn clone(&self) -> Self {
        // For performance reasons we copy all cloned strings into a single cloned
        // head string leaving the cloned `full` empty.
        let new_head_cap = self.head.capacity()
            + self.full.iter().fold(0, |lhs, rhs| lhs + rhs.capacity());
        let mut head = String::with_capacity(new_head_cap);
        let mut spans = Vec::with_capacity(self.spans.len());
        for &span in &self.spans {
            let string = span.as_str();
            let start = head.len();
            head.push_str(string);
            let interned = InternedStr::new(&head[start..start + string.len()]);
            spans.push(interned.into());
        }
        Self {
            spans,
            head,
            full: Vec::new(),
            marker: Default::default(),
        }
    }
}

impl<S> Eq for BucketBackend<S> where S: Symbol {}

impl<S> PartialEq for BucketBackend<S>
where
    S: Symbol,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.spans == other.spans
    }
}

impl<'a, S> IntoIterator for &'a BucketBackend<S>
where
    S: Symbol,
{
    type Item = (S, &'a str);
    type IntoIter = Iter<'a, S>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

pub struct Iter<'a, S> {
    iter: Enumerate<slice::Iter<'a, InternedStr>>,
    symbol_marker: PhantomData<fn() -> S>,
}

impl<'a, S> Iter<'a, S> {
    #[inline]
    pub fn new(backend: &'a BucketBackend<S>) -> Self {
        Self {
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
        self.iter
            .next()
            .map(|(id, interned)| (expect_valid_symbol(id), interned.as_str()))
    }
}
