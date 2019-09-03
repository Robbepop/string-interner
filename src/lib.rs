#![cfg_attr(all(feature = "bench", test), feature(test))]

#![deny(missing_docs)]

//! A string interning data structure that was designed for minimal memory-overhead
//! and fast access to the underlying interned string contents.
//! 
//! Uses a similar interface as the string interner of the rust compiler.
//! 
//! Provides support to use all primitive types as symbols
//! 
//! Example usage:
//! 
//! ```
//! 	use string_interner::DefaultStringInterner;
//! 	let mut interner = DefaultStringInterner::default();
//! 	let name0 = interner.get_or_intern("Elephant");
//! 	let name1 = interner.get_or_intern("Tiger");
//! 	let name2 = interner.get_or_intern("Horse");
//! 	let name3 = interner.get_or_intern("Tiger");
//! 	let name4 = interner.get_or_intern("Tiger");
//! 	let name5 = interner.get_or_intern("Mouse");
//! 	let name6 = interner.get_or_intern("Horse");
//! 	let name7 = interner.get_or_intern("Tiger");
//! 	assert_eq!(name0, 0);
//! 	assert_eq!(name1, 1);
//! 	assert_eq!(name2, 2);
//! 	assert_eq!(name3, 1);
//! 	assert_eq!(name4, 1);
//! 	assert_eq!(name5, 3);
//! 	assert_eq!(name6, 2);
//! 	assert_eq!(name7, 1);
//! ```

#[cfg(all(feature = "bench", test))]
extern crate test;

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

use std::vec;
use std::slice;
use std::iter;
use std::marker;

use std::hash::{Hash, Hasher, BuildHasher};
use std::collections::HashMap;
use std::collections::hash_map::RandomState;

/// Represents indices into the `StringInterner`.
/// 
/// Values of this type shall be lightweight as the whole purpose
/// of interning values is to be able to store them efficiently in memory.
/// 
/// This trait allows definitions of custom `Symbol`s besides
/// the already supported unsigned integer primitives.
pub trait Symbol: Copy + Ord + Eq {
	/// Creates a symbol explicitely from a usize primitive type.
	/// 
	/// Defaults to simply using the standard From<usize> trait.
	fn from_usize(val: usize) -> Self;

	/// Creates a usize explicitely from this symbol.
	/// 
	/// Defaults to simply using the standard Into<usize> trait.
	fn to_usize(self) -> usize;
}

impl<T> Symbol for T where T: Copy + Ord + Eq + From<usize> + Into<usize> {
	#[inline]
	fn from_usize(val: usize) -> Self { val.into() }
	#[inline]
	fn to_usize(self) -> usize { self.into() }
}

/// Internal reference to str used only within the `StringInterner` itself
/// to encapsulate the unsafe behaviour of interor references.
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
		unsafe{ &*self.0 }
	}
}

impl<T> From<T> for InternalStrRef
	where T: AsRef<str>
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

/// Defaults to using usize as the underlying and internal
/// symbol data representation within this `StringInterner`.
pub type DefaultStringInterner = StringInterner<usize>;

/// Provides a bidirectional mapping between String stored within
/// the interner and indices.
/// The main purpose is to store every unique String only once and
/// make it possible to reference it via lightweight indices.
/// 
/// Compilers often use this for implementing a symbol table.
/// 
/// The main goal of this `StringInterner` is to store String
/// with as low memory overhead as possible.
#[derive(Debug, Eq)]
pub struct StringInterner<Sym, H = RandomState>
	where Sym: Symbol,
	      H  : BuildHasher
{
	map   : HashMap<InternalStrRef, Sym, H>,
	values: Vec<Box<str>>
}

impl<Sym, H> PartialEq for StringInterner<Sym, H>
	where Sym: Symbol,
	      H  : BuildHasher
{
	fn eq(&self, rhs: &Self) -> bool {
		self.len() == rhs.len() && self.values == rhs.values
	}
}

impl Default for StringInterner<usize, RandomState> {
	#[inline]
	fn default() -> Self {
		StringInterner::new()
	}
}

impl<Sym, H> Clone for StringInterner<Sym, H>
	where Sym: Symbol,
	      H  : Clone + BuildHasher
{
    fn clone(&self) -> Self {
        let values = self.values.clone();
        let mut map: HashMap<InternalStrRef, _, H> = HashMap::with_capacity_and_hasher(values.len(), self.map.hasher().clone());
        map.extend(
            values.iter().enumerate().map(|(i,s)| (InternalStrRef::from_str(s), Sym::from_usize(i)))
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
unsafe impl<Sym, H> Send for StringInterner<Sym, H> where Sym: Symbol + Send, H: BuildHasher {}
unsafe impl<Sym, H> Sync for StringInterner<Sym, H> where Sym: Symbol + Sync, H: BuildHasher {}

impl<Sym> StringInterner<Sym>
	where Sym: Symbol
{
	/// Creates a new empty `StringInterner`.
	#[inline]
	pub fn new() -> StringInterner<Sym, RandomState> {
		StringInterner{
			map   : HashMap::new(),
			values: Vec::new()
		}
	}

	/// Creates a new `StringInterner` with the given initial capacity.
	#[inline]
	pub fn with_capacity(cap: usize) -> Self {
		StringInterner{
			map   : HashMap::with_capacity(cap),
			values: Vec::with_capacity(cap)
		}
	}

}

impl<Sym, H> StringInterner<Sym, H>
	where Sym: Symbol,
	      H  : BuildHasher
{
	/// Creates a new empty `StringInterner` with the given hasher.
	#[inline]
	pub fn with_hasher(hash_builder: H) -> StringInterner<Sym, H> {
		StringInterner{
			map   : HashMap::with_hasher(hash_builder),
			values: Vec::new()
		}
	}

	/// Creates a new empty `StringInterner` with the given initial capacity and the given hasher.
	#[inline]
	pub fn with_capacity_and_hasher(cap: usize, hash_builder: H) -> StringInterner<Sym, H> {
		StringInterner{
			map   : HashMap::with_hasher(hash_builder),
			values: Vec::with_capacity(cap)
		}
	}

	/// Interns the given value.
	/// 
	/// Returns a symbol to access it within this interner.
	/// 
	/// This either copies the contents of the string (e.g. for str)
	/// or moves them into this interner (e.g. for String).
	#[inline]
	pub fn get_or_intern<T>(&mut self, val: T) -> Sym
		where T: Into<String> + AsRef<str>
	{
		match self.map.get(&val.as_ref().into()) {
			Some(&sym) => sym,
			None       => self.intern(val)
		}
	}

	/// Interns the given value and ignores collissions.
	/// 
	/// Returns a symbol to access it within this interner.
	fn intern<T>(&mut self, new_val: T) -> Sym
		where T: Into<String> + AsRef<str>
	{
		let new_id: Sym = self.make_symbol();
		let new_boxed_val = new_val.into().into_boxed_str();
		let new_ref: InternalStrRef = new_boxed_val.as_ref().into();
		self.values.push(new_boxed_val);
		self.map.insert(new_ref, new_id);
		new_id
	}

	/// Creates a new symbol for the current state of the interner.
	fn make_symbol(&self) -> Sym {
		Sym::from_usize(self.len())
	}

	/// Returns a string slice to the string identified by the given symbol if available.
	/// Else, None is returned.
	#[inline]
	pub fn resolve(&self, symbol: Sym) -> Option<&str> {
		self.values
			.get(symbol.to_usize())
			.map(|boxed_str| boxed_str.as_ref())
	}

	/// Returns a string slice to the string identified by the given symbol,
	/// without doing bounds checking. So use it very carefully!
	#[inline]
	pub unsafe fn resolve_unchecked(&self, symbol: Sym) -> &str {
		self.values.get_unchecked(symbol.to_usize()).as_ref()
	}

	/// Returns the given string's symbol for this interner if existent.
	#[inline]
	pub fn get<T>(&self, val: T) -> Option<Sym>
		where T: AsRef<str>
	{
		self.map
			.get(&val.as_ref().into())
			.cloned()
	}

	/// Returns the number of uniquely stored Strings interned within this interner.
	#[inline]
	pub fn len(&self) -> usize {
		self.values.len()
	}

	/// Returns true if the string interner internes no elements.
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Returns an iterator over the interned strings.
	#[inline]
	pub fn iter(&self) -> Iter<Sym> {
		Iter::new(self)
	}

	/// Returns an iterator over all intern indices and their associated strings.
	#[inline]
	pub fn iter_values(&self) -> Values<Sym> {
		Values::new(self)
	}

	/// Removes all interned Strings from this interner.
	/// 
	/// This invalides all `Symbol` entities instantiated by it so far.
	#[inline]
	pub fn clear(&mut self) {
		self.map.clear();
		self.values.clear()
	}

	/// Shrinks the capacity of the interner as much as possible.
	pub fn shrink_to_fit(&mut self) {
		self.map.shrink_to_fit();
		self.values.shrink_to_fit();
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
		Sym: std::fmt::Debug,
		H: std::fmt::Debug,
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

/// Iterator over the pairs of symbols and interned string for a `StringInterner`.
pub struct Iter<'a, Sym> {
	iter: iter::Enumerate<slice::Iter<'a, Box<str>>>,
	mark: marker::PhantomData<Sym>
}

impl<'a, Sym> Iter<'a, Sym>
	where Sym: Symbol + 'a
{
	/// Creates a new iterator for the given StringIterator over pairs of 
	/// symbols and their associated interned string.
	#[inline]
	fn new<H>(interner: &'a StringInterner<Sym, H>) -> Self
		where H  : BuildHasher
	{
		Iter{iter: interner.values.iter().enumerate(), mark: marker::PhantomData}
	}
}

impl<'a, Sym> Iterator for Iter<'a, Sym>
	where Sym: Symbol + 'a
{
	type Item = (Sym, &'a str);

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		self.iter.next().map(|(num, boxed_str)| (Sym::from_usize(num), boxed_str.as_ref()))
	}

	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		self.iter.size_hint()
	}
}

/// Iterator over the interned strings for a `StringInterner`.
pub struct Values<'a, Sym>
	where Sym: Symbol + 'a
{
	iter: slice::Iter<'a, Box<str>>,
	mark: marker::PhantomData<Sym>
}

impl<'a, Sym> Values<'a, Sym>
	where Sym: Symbol + 'a
{
	/// Creates a new iterator for the given StringIterator over its interned strings.
	#[inline]
	fn new<H>(interner: &'a StringInterner<Sym, H>) -> Self
		where H  : BuildHasher
	{
		Values{
			iter: interner.values.iter(),
			mark: marker::PhantomData
		}
	}
}

impl<'a, Sym> Iterator for Values<'a, Sym>
	where Sym: Symbol + 'a
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

impl<Sym, H> iter::IntoIterator for StringInterner<Sym, H>
	where Sym: Symbol,
	      H  : BuildHasher
{
	type Item = (Sym, String);
	type IntoIter = IntoIter<Sym>;

	fn into_iter(self) -> Self::IntoIter {
		IntoIter{iter: self.values.into_iter().enumerate(), mark: marker::PhantomData}
	}
}

/// Iterator over the pairs of symbols and associated interned string when 
/// morphing a `StringInterner` into an iterator.
pub struct IntoIter<Sym>
	where Sym: Symbol
{
	iter: iter::Enumerate<vec::IntoIter<Box<str>>>,
	mark: marker::PhantomData<Sym>
}

impl<Sym> Iterator for IntoIter<Sym>
	where Sym: Symbol
{
	type Item = (Sym, String);

	fn next(&mut self) -> Option<Self::Item> {
		self.iter.next().map(|(num, boxed_str)| (Sym::from_usize(num), boxed_str.into_string()))
	}

	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		self.iter.size_hint()
	}
}
