#![cfg_attr(all(feature = "bench", test), feature(test))]
#![doc(html_root_url = "https://docs.rs/crate/string-interner/0.7.0")]
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
//! 	.into_iter()
//! 	.collect::<DefaultStringInterner>();
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
//! 	.into_iter()
//! 	.collect::<DefaultStringInterner>();
//! for (sym, str) in interner {
//! 	// iteration code here!
//! }
//! ```

#[cfg(all(feature = "bench", test))]
extern crate test;

#[cfg(all(feature = "bench", test))]
#[macro_use]
extern crate lazy_static;

#[cfg(all(feature = "bench", test))]
extern crate fnv;

#[cfg(feature = "serde_support")]
extern crate serde;

#[cfg(all(feature = "serde_support", test))]
extern crate serde_json;

#[cfg(test)]
mod tests;

#[cfg(all(feature = "bench", test))]
mod benches;

#[cfg(feature = "serde_support")]
mod serde_impl;

use std::iter::FromIterator;
use std::{
	collections::{hash_map::RandomState, HashMap},
	hash::{BuildHasher, Hash, Hasher},
	iter, marker,
	num::NonZeroU32,
	slice, u32, vec,
};

/// Types implementing this trait are able to act as symbols for string interners.
///
/// Symbols are returned by `StringInterner::get_or_intern` and allow look-ups of the
/// original string contents with `StringInterner::resolve`.
///
/// # Note
///
/// Optimal symbols allow for efficient comparisons and have a small memory footprint.
pub trait Symbol: Copy + Ord + Eq {
	/// Creates a symbol from a `usize`.
	///
	/// # Note
	///
	/// Implementations panic if the operation cannot succeed.
	fn from_usize(val: usize) -> Self;

	/// Returns the `usize` representation of `self`.
	fn to_usize(self) -> usize;
}

/// Symbol type used by the `DefaultStringInterner`.
///
/// # Note
///
/// This special symbol type has a memory footprint of 32 bits
/// and allows for certain space optimizations such as using it within an option: `Option<Sym>`
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sym(NonZeroU32);

impl Symbol for Sym {
	/// Creates a `Sym` from the given `usize`.
	///
	/// # Panics
	///
	/// If the given `usize` is greater than `u32::MAX - 1`.
	fn from_usize(val: usize) -> Self {
		assert!(
			val < u32::MAX as usize,
			"Symbol value {} is too large and not supported by `string_interner::Sym` type",
			val
		);
		Sym(NonZeroU32::new((val + 1) as u32).unwrap_or_else(|| {
			unreachable!("Should never fail because `val + 1` is nonzero and `<= u32::MAX`")
		}))
	}

	fn to_usize(self) -> usize {
		(self.0.get() as usize) - 1
	}
}

impl Symbol for usize {
	fn from_usize(val: usize) -> Self {
		val
	}

	fn to_usize(self) -> usize {
		self
	}
}

/// Internal reference to `str` used only within the `StringInterner` itself
/// to encapsulate the unsafe behaviour of interior references.
#[derive(Debug, Copy, Clone, Eq)]
struct InternalStrRef(*const str);

impl InternalStrRef {
	/// Creates an InternalStrRef from a str.
	///
	/// This just wraps the str internally.
	fn from_str(val: &str) -> Self {
		InternalStrRef(val as *const str)
	}

	/// Reinterprets this InternalStrRef as a str.
	///
	/// This is "safe" as long as this InternalStrRef only
	/// refers to strs that outlive this instance or
	/// the instance that owns this InternalStrRef.
	/// This should hold true for `StringInterner`.
	///
	/// Does not allocate memory!
	fn as_str(&self) -> &str {
		unsafe { &*self.0 }
	}
}

impl<T> From<T> for InternalStrRef
where
	T: AsRef<str>,
{
	fn from(val: T) -> Self {
		InternalStrRef::from_str(val.as_ref())
	}
}

impl Hash for InternalStrRef {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.as_str().hash(state)
	}
}

impl PartialEq for InternalStrRef {
	fn eq(&self, other: &InternalStrRef) -> bool {
		self.as_str() == other.as_str()
	}
}

/// `StringInterner` that uses `Sym` as its underlying symbol type.
pub type DefaultStringInterner = StringInterner<Sym>;

/// Caches strings efficiently, with minimal memory footprint and associates them with unique symbols.
/// These symbols allow constant time comparisons and look-ups to the underlying interned strings.
// # About the safety of `InternalStrRef`
//
// A string interner has `InternalStrRef`s and dereferences them, and the interner implements
// `Sync` and `Send` traits manually.
//
// These are safe when:
//
// * 1: whenever `InternalStrRef`s are dereferenced, referred strings should be alive, and
// * 2: strings referred by each `InternalStrRef` are not mutated from more than two threads at the
//   same time, and are not mutated when another read can happen.
//
// These conditions are satisfied, because:
//
// * 1-1: `InternalStrRef`s refer only the strings owned by the same interner, and are not being
//   taken outside,
// * 1-2: strings referred by `InternalStrRef`s are not moved to other place,
// * 1-3: once strings referred by `InternalStrRef`s are created, they would not be dropped until
//   the interner itself is dropped, and
// * 2-1: both `InternalStrRef`s and the strings they refer are read-only.
//
// ## 1-1: `InternalStrRef`s refer only the strings owned by the same interner, and are not being taken outside
//
// When an `InternalStrRef` is created at `StringInterner::intern()`, it refers also newly created
// string. The `InternalStrRef` is inserted to `self.map` and the string is `push()`ed to
// `self.values`, so they are owned by the same interner.
//
// `InternalStrRef`s are used as keys of the hashmap `self.map`, and they are not copied or moved
// outside of the owning interner.
// Map keys are used to look up symbols, but are not used outside the `HashMap` type once they are
// inserted to the hashmap.
//
// When the interner is cloned, the hashmap is not directly cloned but is newly created using newly
// cloned strings, so `InternalStrRef`s are not copied outside of the old interner to be cloned, and
// the `InternalStrRef`s of newly cloned interner refers the strings owned by the new intreners.
//
// Therefore, `InternalStrRef`s always refer the strings owned by the same interner as
// `InternalStrRef`s, and are not taken outside of the owning interner.
//
// ## 1-2: Strings referred by `InternalStrRef`s are not moved to other place
//
// Strings referred by `InternalStrRef`s are stored in `self.values`, that is `Vec<Box<str>>`.
// An `InternalStrRef` has a pointer to the string directly owned by `Box<str>`.
// `Box<str>` does not reallocate the string by any operation.
// Additionally, the `Box<str>` won't be dropped until the owning interner is dropped (condition
// 1-3).
// Therefore, the addresses of the strings referred by `InternalStrRef`s are not changed (until the
// owning interner is dropped).
//
// ## 1-3: Strings referred by `InternalStrRef`s would not be dropped until the interner is dropped
//
// Strings are stored in `self.values`, and once a string is added, the string will never be
// removed from the vector until the vector itself is dropped.
// Therefore, by condition 1-1 to 1-3, all `InternalStrRef`s remain valid while the owning interner
// exists.
//
// ## 2-1: Both `InternalStrRef`s and the strings they refer are read-only
//
// Once an `InternalStrRef` is inserted to `self.map` as a key, it is immutable (because the keys
// of the map would not be modified).
// The strings contained by `Box<str>` are also immutable because the interner mutates neither the
// box nor the strings.
// Therefore, `InternalStrRef`s and the strings they refer are read-only.
#[derive(Debug, Eq)]
pub struct StringInterner<S, H = RandomState>
where
	S: Symbol,
	H: BuildHasher,
{
	map: HashMap<InternalStrRef, S, H>,
	values: Vec<Box<str>>,
}

impl<S, H> PartialEq for StringInterner<S, H>
where
	S: Symbol,
	H: BuildHasher,
{
	fn eq(&self, rhs: &Self) -> bool {
		self.len() == rhs.len() && self.values == rhs.values
	}
}

impl Default for StringInterner<Sym, RandomState> {
	#[inline]
	fn default() -> Self {
		StringInterner::new()
	}
}

// Should be manually cloned.
// See <https://github.com/Robbepop/string-interner/issues/9>.
impl<S, H> Clone for StringInterner<S, H>
where
	S: Symbol,
	H: Clone + BuildHasher,
{
	fn clone(&self) -> Self {
		let values = self.values.clone();
		let mut map = HashMap::with_capacity_and_hasher(values.len(), self.map.hasher().clone());
		// Recreate `InternalStrRef` from the newly cloned `Box<str>`s.
		// Use `extend()` to avoid `H: Default` trait bound required by `FromIterator for HashMap`.
		map.extend(
			values
			.iter()
			.enumerate()
			.map(|(i, s)| (InternalStrRef::from_str(s), S::from_usize(i))),
		);
		Self { values, map }
	}
}

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
	pub fn new() -> StringInterner<S, RandomState> {
		StringInterner {
			map: HashMap::new(),
			values: Vec::new(),
		}
	}

	/// Creates a new `StringInterner` with the given initial capacity.
	#[inline]
	pub fn with_capacity(cap: usize) -> Self {
		StringInterner {
			map: HashMap::with_capacity(cap),
			values: Vec::with_capacity(cap),
		}
	}

	/// Returns the number of elements the `StringInterner` can hold without reallocating.
	#[inline]
	pub fn capacity(&self) -> usize {
		std::cmp::min(self.map.capacity(), self.values.capacity())
	}

	/// Reserves capacity for at least `additional` more elements to be interned into `self`.
	///
	/// The collection may reserve more space to avoid frequent allocations.
	/// After calling `reserve`, capacity will be greater than or equal to `self.len() + additional`.
	/// Does nothing if capacity is already sufficient.
	#[inline]
	pub fn reserve(&mut self, additional: usize) {
		self.map.reserve(additional);
		self.values.reserve(additional);
	}

	/// Checks whether the all `InternalStrRef`s refer the strigs owned by `self`.
	///
	/// For testing purpose only.
	///
	/// # Panics
	///
	/// Panics if the interner has wrong state. That is:
	///
	/// * when `InternalStrRef` refers the address which is not owned by the interner, or
	/// * when there are `Box<str>` not referred by any `InternalStrRef` owned by the interner.
	#[cfg(test)]
	pub(crate) fn assert_internal_str_refs_validity(&self)
	where
		S: std::fmt::Debug,
	{
		// Collect `InternalStrRef` pointers.
		let mut referred_ptrs = self
			.map
			.keys()
			.map(|s| s.0)
			.collect::<std::collections::HashSet<_>>();
		// Remove owned pointers.
		for (owned_str, owned_ptr) in self.values.iter().map(|v| (&**v, (&**v) as *const str)) {
			if !referred_ptrs.remove(&owned_ptr) {
				// `owned` is not in `referred_ptrs`.
				// It means the `Box<str>` is not found by `get()` and `get_or_intern()`.
				panic!(
					"String {:?} at {:?} is not registered to `map`: self={:#?}",
					owned_str, owned_ptr, self
				);
			}
		}
		if !referred_ptrs.is_empty() {
			// `self.map` has some dangling pointers.
			let values_ptrs = self
				.values
				.iter()
				.map(|v| (&**v, (&**v) as *const str))
				.collect::<Vec<_>>();
			panic!(
				"Dangling pointers found: pointers {:?} are not stored in `values`: \
				self={:#?}, values_ptrs = {:?}",
				referred_ptrs, self, values_ptrs
			);
		}
	}

	/// Returns the maximum capacity of the internal storages.
	///
	/// Storing `self.max_capacity() + 1` elements in total will cause all storages to be
	/// reallocated at least once.
	///
	/// For testing purpose only.
	#[cfg(test)]
	pub(crate) fn max_capacity(&self) -> usize {
		std::cmp::max(self.map.capacity(), self.values.capacity())
	}
}

impl<S, H> StringInterner<S, H>
where
	S: Symbol,
	H: BuildHasher,
{
	/// Creates a new empty `StringInterner` with the given hasher.
	#[inline]
	pub fn with_hasher(hash_builder: H) -> StringInterner<S, H> {
		StringInterner {
			map: HashMap::with_hasher(hash_builder),
			values: Vec::new(),
		}
	}

	/// Creates a new empty `StringInterner` with the given initial capacity and the given hasher.
	#[inline]
	pub fn with_capacity_and_hasher(cap: usize, hash_builder: H) -> StringInterner<S, H> {
		StringInterner {
			map: HashMap::with_hasher(hash_builder),
			values: Vec::with_capacity(cap),
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
		match self.map.get(&val.as_ref().into()) {
			Some(&sym) => sym,
			None => self.intern(val),
		}
	}

	/// Interns the given value and ignores collissions.
	///
	/// Returns a symbol to access it within this interner.
	fn intern<T>(&mut self, new_val: T) -> S
	where
		T: Into<String> + AsRef<str>,
	{
		let new_id: S = self.make_symbol();
		let new_boxed_val = new_val.into().into_boxed_str();
		let new_ref: InternalStrRef = new_boxed_val.as_ref().into();
		self.values.push(new_boxed_val);
		self.map.insert(new_ref, new_id);
		new_id
	}

	/// Creates a new symbol for the current state of the interner.
	fn make_symbol(&self) -> S {
		S::from_usize(self.len())
	}

	/// Returns the string slice associated with the given symbol if available,
	/// otherwise returns `None`.
	#[inline]
	pub fn resolve(&self, symbol: S) -> Option<&str> {
		self.values
			.get(symbol.to_usize())
			.map(|boxed_str| boxed_str.as_ref())
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
	/// had no associated string for this interner instance.
	#[inline]
	pub unsafe fn resolve_unchecked(&self, symbol: S) -> &str {
		self.values.get_unchecked(symbol.to_usize()).as_ref()
	}

	/// Returns the symbol associated with the given string for this interner
	/// if existent, otherwise returns `None`.
	#[inline]
	pub fn get<T>(&self, val: T) -> Option<S>
	where
		T: AsRef<str>,
	{
		self.map.get(&val.as_ref().into()).cloned()
	}

	/// Returns the number of uniquely interned strings within this interner.
	#[inline]
	pub fn len(&self) -> usize {
		self.values.len()
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
		self.values.shrink_to_fit();
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

impl<T, S> std::iter::Extend<T> for StringInterner<S>
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

/// Iterator over the pairs of associated symbols and interned strings for a `StringInterner`.
pub struct Iter<'a, S> {
	iter: iter::Enumerate<slice::Iter<'a, Box<str>>>,
	mark: marker::PhantomData<S>,
}

impl<'a, S> Iter<'a, S>
where
	S: Symbol + 'a,
{
	/// Creates a new iterator for the given StringIterator over pairs of
	/// symbols and their associated interned string.
	#[inline]
	fn new<H>(interner: &'a StringInterner<S, H>) -> Self
	where
		H: BuildHasher,
	{
		Iter {
			iter: interner.values.iter().enumerate(),
			mark: marker::PhantomData,
		}
	}
}

impl<'a, S> Iterator for Iter<'a, S>
where
	S: Symbol + 'a,
{
	type Item = (S, &'a str);

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		self.iter
			.next()
			.map(|(num, boxed_str)| (S::from_usize(num), boxed_str.as_ref()))
	}

	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		self.iter.size_hint()
	}
}

/// Iterator over the interned strings of a `StringInterner`.
pub struct Values<'a, S>
where
	S: Symbol + 'a,
{
	iter: slice::Iter<'a, Box<str>>,
	mark: marker::PhantomData<S>,
}

impl<'a, S> Values<'a, S>
where
	S: Symbol + 'a,
{
	/// Creates a new iterator for the given StringIterator over its interned strings.
	#[inline]
	fn new<H>(interner: &'a StringInterner<S, H>) -> Self
	where
		H: BuildHasher,
	{
		Values {
			iter: interner.values.iter(),
			mark: marker::PhantomData,
		}
	}
}

impl<'a, S> Iterator for Values<'a, S>
where
	S: Symbol + 'a,
{
	type Item = &'a str;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		self.iter.next().map(|boxed_str| boxed_str.as_ref())
	}

	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		self.iter.size_hint()
	}
}

impl<S, H> iter::IntoIterator for StringInterner<S, H>
where
	S: Symbol,
	H: BuildHasher,
{
	type Item = (S, String);
	type IntoIter = IntoIter<S>;

	fn into_iter(self) -> Self::IntoIter {
		IntoIter {
			iter: self.values.into_iter().enumerate(),
			mark: marker::PhantomData,
		}
	}
}

/// Iterator over the pairs of associated symbol and strings.
///
/// Consumes the `StringInterner` upon usage.
pub struct IntoIter<S>
where
	S: Symbol,
{
	iter: iter::Enumerate<vec::IntoIter<Box<str>>>,
	mark: marker::PhantomData<S>,
}

impl<S> Iterator for IntoIter<S>
where
	S: Symbol,
{
	type Item = (S, String);

	fn next(&mut self) -> Option<Self::Item> {
		self.iter
			.next()
			.map(|(num, boxed_str)| (S::from_usize(num), boxed_str.into_string()))
	}

	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		self.iter.size_hint()
	}
}
