#![doc(html_root_url = "https://docs.rs/crate/string-interner/0.8.0")]
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

//! Caches strings efficiently, with minimal memory footprint and associates them with unique symbols.
//! These symbols allow constant time comparisons and look-ups to the underlying interned strings.
//!
//! ### Example: Interning & Symbols
//!
//! ```
//! use string_interner::StringInterner;
//!
//! let mut interner = StringInterner::default();
//! let sym0 = interner.get_or_intern("Elephant");
//! let sym1 = interner.get_or_intern("Tiger");
//! let sym2 = interner.get_or_intern("Horse");
//! let sym3 = interner.get_or_intern("Tiger");
//! assert_ne!(sym0, sym1);
//! assert_ne!(sym0, sym2);
//! assert_ne!(sym1, sym2);
//! assert_eq!(sym1, sym3); // same!
//! ```
//!
//! ### Example: Creation by `FromIterator`
//!
//! ```
//! # use string_interner::DefaultStringInterner;
//! let interner = vec!["Elephant", "Tiger", "Horse", "Tiger"]
//!     .into_iter()
//!     .collect::<DefaultStringInterner>();
//! ```
//!
//! ### Example: Look-up
//!
//! ```
//! # use string_interner::StringInterner;
//! let mut interner = StringInterner::default();
//! let sym = interner.get_or_intern("Banana");
//! assert_eq!(interner.resolve(sym), Some("Banana"));
//! ```
//!
//! ### Example: Iteration
//!
//! ```
//! # use string_interner::DefaultStringInterner;
//! let interner = vec!["Earth", "Water", "Fire", "Air"]
//!     .into_iter()
//!     .collect::<DefaultStringInterner>();
//! for (sym, str) in interner {
//!     // iteration code here!
//! }
//! ```

#[cfg(test)]
mod tests;

#[cfg(feature = "serde-1")]
mod serde_impl;

mod compat;
mod iter;
mod symbol;

use crate::compat::{
    Box,
    DefaultHashBuilder,
    HashMap,
    String,
    Vec,
};
pub use crate::{
    iter::{
        IntoIter,
        Iter,
        Values,
    },
    symbol::{
        DefaultSymbol,
        Symbol,
    },
};
use core::{
    hash::{
        BuildHasher,
        Hash,
        Hasher,
    },
    iter::{
        FromIterator,
        IntoIterator,
        Iterator,
    },
    pin::Pin,
    ptr::NonNull,
};

/// Internal reference to an interned `str`.
///
/// This is a self-referential from the interners string map
/// into the interner's actual vector of strings.
#[derive(Debug, Copy, Clone, Eq)]
struct PinnedStr(NonNull<str>);

impl PinnedStr {
    /// Creates a new `PinnedStr` from the given `str`.
    fn from_str(val: &str) -> Self {
        PinnedStr(NonNull::from(val))
    }

    /// Creates a new `PinnedStr` from the given pinned `str`.
    fn from_pin(pinned: Pin<&str>) -> Self {
        PinnedStr(NonNull::from(&*pinned))
    }

    /// Returns a shared reference to the underlying `str`.
    fn as_str(&self) -> &str {
        // SAFETY: This is safe since we only ever operate on interned `str`
        //         that are never moved around in memory to avoid danling
        //         references.
        unsafe { self.0.as_ref() }
    }
}

impl Hash for PinnedStr {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl PartialEq for PinnedStr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl core::borrow::Borrow<str> for PinnedStr {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

/// `StringInterner` that uses `Sym` as its underlying symbol type.
pub type DefaultStringInterner = StringInterner<DefaultSymbol>;

/// Caches strings efficiently, with minimal memory footprint and associates them with unique symbols.
/// These symbols allow constant time comparisons and look-ups to the underlying interned strings.
#[derive(Debug, Eq)]
pub struct StringInterner<S, H = DefaultHashBuilder>
where
    S: Symbol,
    H: BuildHasher,
{
    /// Mapping from the interned string spans to their symbols.
    map: HashMap<PinnedStr, S, H>,
    /// Spans for all the interned string pointing to either `buf` of `full`.
    spans: Vec<PinnedStr>,
    /// The accumulating buffer that is appended to `full` when ... full.
    buf: String,
    /// All filled string buffers.
    full: Vec<String>,
}

impl<S, H> PartialEq for StringInterner<S, H>
where
    S: Symbol,
    H: BuildHasher,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.len() == rhs.len() && self.spans == rhs.spans
    }
}

impl Default for StringInterner<DefaultSymbol, DefaultHashBuilder> {
    #[inline]
    fn default() -> Self {
        StringInterner::new()
    }
}

// impl<S, H> Clone for StringInterner<S, H>
// where
//     S: Symbol,
//     H: Clone + BuildHasher,
// {
//     fn clone(&self) -> Self {
//         // We implement `Clone` manually for `StringInterner` to go around the
//         // issue of shallow closing the self-referential pinned strs.
//         // This was an issue with former implementations. Visit the following
//         // link for more information:
//         // https://github.com/Robbepop/string-interner/issues/9
//         let values = self.full.clone();
//         let mut map =
//             HashMap::with_capacity_and_hasher(values.len(), self.map.hasher().clone());
//         // Recreate `InternalStrRef` from the newly cloned `Box<str>`s.
//         // Use `extend()` to avoid `H: Default` trait bound required by `FromIterator for HashMap`.
//         map.extend(
//             values
//                 .iter()
//                 .enumerate()
//                 .map(|(i, s)| (PinnedStr::from_str(s), S::from_usize(i))),
//         );
//         Self { full: values, map }
//     }
// }

// About `Send` and `Sync` impls for `StringInterner`
// --------------------------------------------------
//
// tl;dr: Automation of Send+Sync impl was prevented by `InternalStrRef`
// being an unsafe abstraction and thus prevented Send+Sync default derivation.
//
// These implementations are safe due to the following reasons:
//  - `InternalStrRef` cannot be used outside `StringInterner`.
//  - Strings stored in `StringInterner` are not mutable.
//  - Iterator invalidation while growing the underlying `Vec<Box<str>>` is prevented by
//    using an additional indirection to store strings.
unsafe impl<S, H> Send for StringInterner<S, H>
where
    S: Symbol + Send,
    H: BuildHasher,
{
}
unsafe impl<S, H> Sync for StringInterner<S, H>
where
    S: Symbol + Sync,
    H: BuildHasher,
{
}

impl<S> StringInterner<S>
where
    S: Symbol,
{
    /// Creates a new empty `StringInterner`.
    #[inline]
    pub fn new() -> StringInterner<S, DefaultHashBuilder> {
        Self {
            map: HashMap::new(),
            spans: Vec::new(),
            buf: String::new(),
            full: Vec::new(),
        }
    }

    /// Creates a new `StringInterner` with the given initial capacity.
    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        let cap = cap.next_power_of_two();
        Self {
            map: HashMap::with_capacity(cap),
            spans: Vec::with_capacity(cap),
            buf: String::with_capacity(cap),
            full: Vec::new(),
        }
    }

    // /// Returns the number of strings the `StringInterner` can additionally hold without reallocating.
    // #[inline]
    // pub fn capacity(&self) -> usize {

    //     core::cmp::min(self.map.capacity(), self.full.capacity())
    // }

    // /// Reserves capacity for at least `additional` more elements to be interned into `self`.
    // ///
    // /// The collection may reserve more space to avoid frequent allocations.
    // /// After calling `reserve`, capacity will be greater than or equal to `self.len() + additional`.
    // /// Does nothing if capacity is already sufficient.
    // #[inline]
    // pub fn reserve(&mut self, additional: usize) {
    //     self.map.reserve(additional);
    //     self.full.reserve(additional);
    // }
}

impl<S, H> StringInterner<S, H>
where
    S: Symbol,
    H: BuildHasher,
{
    /// Creates a new empty `StringInterner` with the given hasher.
    #[inline]
    pub fn with_hasher(hash_builder: H) -> StringInterner<S, H> {
        Self {
            map: HashMap::with_hasher(hash_builder),
            spans: Vec::new(),
            buf: String::new(),
            full: Vec::new(),
        }
    }

    /// Creates a new empty `StringInterner` with the given initial capacity and the given hasher.
    #[inline]
    pub fn with_capacity_and_hasher(cap: usize, hash_builder: H) -> StringInterner<S, H> {
        let cap = cap.next_power_of_two();
        Self {
            map: HashMap::with_hasher(hash_builder),
            spans: Vec::with_capacity(cap),
            buf: String::with_capacity(cap),
            full: Vec::new(),
        }
    }

    /// Interns the given value.
    ///
    /// Returns a symbol to access it within this interner.
    ///
    /// This either copies the contents of the string (e.g. for str)
    /// or moves them into this interner (e.g. for String).
    #[inline]
    pub fn get_or_intern<T>(&mut self, val: T) -> S
    where
        T: Into<String> + AsRef<str>,
    {
        self.map
            .get(val.as_ref())
            .copied()
            .unwrap_or_else(|| self.intern(val))
    }

    /// Interns the given value and ignores collissions.
    ///
    /// Returns a symbol to access it within this interner.
    fn intern<T>(&mut self, new_val: T) -> S
    where
        T: Into<String> + AsRef<str>,
    {
        let new_str = new_val.as_ref();
        let pinned = self.alloc(new_str);
        let id = self.next_symbol();
        self.map.insert(pinned, id);
        self.spans.push(pinned);

        // debug_assert_eq!(self.get_or_intern(new_val), id);
        debug_assert_eq!(self.resolve(id), Some(new_str));

        id
    }

    fn alloc<T>(&mut self, new_val: T) -> PinnedStr
    where
        T: Into<String> + AsRef<str>,
    {
        let new_str = new_val.as_ref();
        let cap = self.buf.capacity();
        if cap < self.buf.len() + new_str.len() {
            let new_cap = (usize::max(cap, new_str.len()) + 1).next_power_of_two();
            let new_buf = String::with_capacity(new_cap);
            let old_buf = ::core::mem::replace(&mut self.buf, new_buf);
            self.full.push(old_buf);
        }
        let interned = {
            let start = self.buf.len();
            self.buf.push_str(new_str);
            &self.buf[start..]
        };
        PinnedStr::from_str(unsafe { &*(interned as *const str) })
    }

    /// Creates a new symbol for the current state of the interner.
    fn next_symbol(&self) -> S {
        S::from_usize(self.len())
    }

    /// Returns the string slice associated with the given symbol if available,
    /// otherwise returns `None`.
    #[inline]
    pub fn resolve(&self, symbol: S) -> Option<&str> {
        self.spans
            .get(symbol.to_usize())
            .map(|pinned| pinned.as_str())
    }

    /// Returns the string associated with the given symbol.
    ///
    /// # Note
    ///
    /// This does not check whether the given symbol has an associated string
    /// for the given string interner instance.
    ///
    /// # Safety
    ///
    /// This will result in undefined behaviour if the given symbol
    /// has no associated string for this interner instance.
    #[inline]
    pub unsafe fn resolve_unchecked(&self, symbol: S) -> &str {
        self.spans
            .get_unchecked(symbol.to_usize())
            .as_str()
    }

    /// Returns the symbol associated with the given string for this interner
    /// if existent, otherwise returns `None`.
    #[inline]
    pub fn get<T>(&self, val: T) -> Option<S>
    where
        T: AsRef<str>,
    {
        self.map.get(val.as_ref()).copied()
    }

    /// Returns the number of uniquely interned strings within this interner.
    #[inline]
    pub fn len(&self) -> usize {
        self.full.len()
    }

    /// Returns true if the string interner holds no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the interned strings.
    #[inline]
    pub fn iter(&self) -> Iter<S> {
        Iter::new(self)
    }

    /// Returns an iterator over all intern indices and their associated strings.
    #[inline]
    pub fn iter_values(&self) -> Values<S> {
        Values::new(self)
    }

    /// Shrinks the capacity of the interner as much as possible.
    pub fn shrink_to_fit(&mut self) {
        self.map.shrink_to_fit();
        self.full.shrink_to_fit();
    }
}

impl<T, S> FromIterator<T> for StringInterner<S>
where
    S: Symbol,
    T: Into<String> + AsRef<str>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let iter = iter.into_iter();
        let mut interner = StringInterner::with_capacity(iter.size_hint().0);
        interner.extend(iter);
        interner
    }
}

impl<T, S> Extend<T> for StringInterner<S>
where
    S: Symbol,
    T: Into<String> + AsRef<str>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for s in iter {
            self.get_or_intern(s);
        }
    }
}

impl<S, H> IntoIterator for StringInterner<S, H>
where
    S: Symbol,
    H: BuildHasher,
{
    type Item = (S, String);
    type IntoIter = IntoIter<S>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

impl<'a, S, H> IntoIterator for &'a StringInterner<S, H>
where
    S: Symbol,
    H: BuildHasher,
{
    type Item = (S, &'a str);
    type IntoIter = Iter<'a, S>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
