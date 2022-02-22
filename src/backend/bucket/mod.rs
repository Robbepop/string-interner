#![cfg(feature = "backends")]

//! An interner backend that reduces memory allocations by using string buckets.
//!
//! # Note
//!
//! Implementation inspired by matklad's blog post that can be found here:
//! <https://matklad.github.io/2020/03/22/fast-simple-rust-interner.html>
//!
//! # Usage Hint
//!
//! Use when deallocations or copy overhead is costly or when
//! interning of static strings is especially common.
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
//! | Footprint   | **ok**   |
//! | Supports `get_or_intern_static` | **yes** |
//! | `Send` + `Sync` | **yes** |
//! | Contiguous  | **yes**  |

mod fixed;

pub use fixed::FixedContainer;

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
use len_trait::{
    Capacity,
    Empty,
    Len,
    WithCapacity,
};
use std::ptr::NonNull;

/// An interner backend that reduces memory allocations by using string buckets.
///
/// See the [module-level documentation](self) for more.
#[derive(Debug)]
pub struct BucketBackend<S = str, Sym = DefaultSymbol>
where
    S: ?Sized + Internable,
    S::Container: FixedContainer<S>,
{
    spans: Vec<NonNull<S>>,
    head: S::Container,
    full: Vec<S::Container>,
    _marker: PhantomData<fn() -> Sym>,
}

/// # Safety
///
/// The bucket backend requires a manual [`Send`] impl because it is self
/// referential. When cloning a bucket backend a deep clone is performed and
/// all references to itself are updated for the clone.
unsafe impl<S, Sym> Send for BucketBackend<S, Sym>
where
    Sym: Symbol,
    S: ?Sized + Internable,
    S::Container: FixedContainer<S>,
{
}

/// # Safety
///
/// The bucket backend requires a manual [`Send`] impl because it is self
/// referential. Those references won't escape its own scope and also
/// the bucket backend has no interior mutability.
unsafe impl<S, Sym> Sync for BucketBackend<S, Sym>
where
    Sym: Symbol,
    S: ?Sized + Internable,
    S::Container: FixedContainer<S>,
{
}

impl<S, Sym> Default for BucketBackend<S, Sym>
where
    S: ?Sized + Internable,
    S::Container: FixedContainer<S>,
{
    #[cfg_attr(feature = "inline-more", inline)]
    fn default() -> Self {
        Self {
            spans: Vec::new(),
            head: S::Container::default(),
            full: Vec::new(),
            _marker: Default::default(),
        }
    }
}

impl<S, Sym> Backend for BucketBackend<S, Sym>
where
    Sym: Symbol,
    S: ?Sized + Internable,
    S::Container: FixedContainer<S>,
{
    type Str = S;
    type Symbol = Sym;

    #[cfg_attr(feature = "inline-more", inline)]
    fn with_capacity(cap: usize) -> Self {
        Self {
            spans: Vec::with_capacity(cap),
            head: S::Container::with_capacity(cap),
            full: Vec::new(),
            _marker: Default::default(),
        }
    }

    #[inline]
    fn intern(&mut self, string: &S) -> Self::Symbol {
        // SAFETY: This is safe because we never hand out the returned
        //         interned string instance to the outside and only operate
        //         on it within this backend.
        let interned = unsafe { self.alloc(string) };
        self.push_span(interned)
    }

    #[cfg_attr(feature = "inline-more", inline)]
    fn intern_static(&mut self, string: &'static S) -> Self::Symbol {
        let interned = NonNull::from(string);
        self.push_span(interned)
    }

    fn shrink_to_fit(&mut self) {
        self.spans.shrink_to_fit();
        self.full.shrink_to_fit();
    }

    #[inline]
    fn resolve(&self, symbol: Self::Symbol) -> Option<&S> {
        // SAFETY: A `FixedContainer` cannot invalidate pointers to its interned
        //         strings, making its spans always valid.
        unsafe { self.spans.get(symbol.to_usize()).map(|p| p.as_ref()) }
    }

    #[inline]
    unsafe fn resolve_unchecked(&self, symbol: Self::Symbol) -> &S {
        // SAFETY: The function is marked unsafe so that the caller guarantees
        //         that required invariants are checked.
        unsafe { self.spans.get_unchecked(symbol.to_usize()).as_ref() }
    }
}

impl<S, Sym> BucketBackend<S, Sym>
where
    Sym: Symbol,
    S: ?Sized + Internable,
    S::Container: FixedContainer<S>,
{
    /// Returns the next available symbol.
    fn next_symbol(&self) -> Sym {
        expect_valid_symbol(self.spans.len())
    }

    /// Pushes the given interned string into the spans and returns its symbol.
    fn push_span(&mut self, interned: NonNull<S>) -> Sym {
        let symbol = self.next_symbol();
        self.spans.push(interned);
        symbol
    }

    /// Interns a new string into the backend and returns a reference to it.
    unsafe fn alloc(&mut self, string: &S) -> NonNull<S> {
        let cap = self.head.capacity();
        if cap < self.head.len() + string.len() {
            let new_cap = (usize::max(cap, string.len()) + 1).next_power_of_two();
            let new_head = S::Container::with_capacity(new_cap);
            let old_head = core::mem::replace(&mut self.head, new_head);
            if !old_head.is_empty() {
                self.full.push(old_head);
            }
        }
        self.head
            .try_push_str(string)
            .expect("encountered invalid head capacity (2)")
    }
}

impl<S, Sym> Clone for BucketBackend<S, Sym>
where
    S: ?Sized + Internable,
    S::Container: FixedContainer<S>,
{
    fn clone(&self) -> Self {
        // For performance reasons we copy all cloned strings into a single cloned
        // head string leaving the cloned `full` empty.
        let new_head_cap =
            self.head.capacity() + self.full.iter().fold(0, |lhs, rhs| lhs + rhs.len());
        let mut head = S::Container::with_capacity(new_head_cap);
        let mut spans = Vec::with_capacity(self.spans.len());
        for span in &self.spans {
            // SAFETY: This is safe because a `FixedContainer` cannot invalidate pointers
            //         to its interned strings, making its references always valid.
            unsafe {
                let string = span.as_ref();
                let interned = head
                    .try_push_str(string)
                    .expect("encountered invalid head capacity");
                spans.push(interned);
            }
        }
        Self {
            spans,
            head,
            full: Vec::new(),
            _marker: Default::default(),
        }
    }
}

impl<S, Sym> Eq for BucketBackend<S, Sym>
where
    Sym: Symbol,
    S: ?Sized + Internable + Eq,
    S::Container: FixedContainer<S>,
{
}

impl<S, Sym> PartialEq for BucketBackend<S, Sym>
where
    Sym: Symbol,
    S: ?Sized + Internable + PartialEq,
    S::Container: FixedContainer<S>,
{
    #[cfg_attr(feature = "inline-more", inline)]
    fn eq(&self, other: &Self) -> bool {
        if self.spans.len() != other.spans.len() {
            return false
        }

        // SAFETY: A `FixedContainer` cannot invalidate pointers to its interned
        //         strings, making its spans always valid.
        unsafe {
            self.spans
                .iter()
                .zip(other.spans.iter())
                .all(|(x, y)| x.as_ref() == y.as_ref())
        }
    }
}

impl<'a, S, Sym> IntoIterator for &'a BucketBackend<S, Sym>
where
    Sym: Symbol,
    S: ?Sized + Internable,
    S::Container: FixedContainer<S>,
{
    type Item = (Sym, &'a S);
    type IntoIter = Iter<'a, S, Sym>;

    #[cfg_attr(feature = "inline-more", inline)]
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

/// Iterator for a [`BucketBackend`](crate::backend::bucket::BucketBackend)
/// that returns all of its interned strings.
pub struct Iter<'a, S, Sym>
where
    S: ?Sized + Internable,
    S::Container: FixedContainer<S>,
{
    iter: Enumerate<slice::Iter<'a, NonNull<S>>>,
    marker: PhantomData<fn(&S) -> Sym>,
}

impl<'a, S, Sym> Iter<'a, S, Sym>
where
    S: ?Sized + Internable,
    S::Container: FixedContainer<S>,
{
    #[cfg_attr(feature = "inline-more", inline)]
    pub(super) fn new(backend: &'a BucketBackend<S, Sym>) -> Self {
        Self {
            iter: backend.spans.iter().enumerate(),
            marker: Default::default(),
        }
    }
}

impl<'a, S, Sym> Iterator for Iter<'a, S, Sym>
where
    Sym: Symbol,
    S: ?Sized + Internable,
    S::Container: FixedContainer<S>,
{
    type Item = (Sym, &'a S);

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            self.iter
                .next()
                .map(|(id, interned)| (expect_valid_symbol(id), interned.as_ref()))
        }
    }
}
