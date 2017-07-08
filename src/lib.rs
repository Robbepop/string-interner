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
//! 	use string_interner::StringInterner;
//! 	let mut interner = StringInterner::<usize>::new();
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

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

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
unsafe impl<Sym> Send for StringInterner<Sym> where Sym: Symbol + Send {}
unsafe impl<Sym> Sync for StringInterner<Sym> where Sym: Symbol + Sync {}


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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringInterner<Sym>
	where Sym: Symbol
{
	map   : HashMap<InternalStrRef, Sym>,
	values: Vec<Box<str>>
}

impl<S> Default for StringInterner<S>
	where S: Symbol
{
	fn default() -> Self {
		StringInterner::new()
	}
}

impl<Sym> StringInterner<Sym>
	where Sym: Symbol
{
	/// Creates a new empty `StringInterner`.
	/// 
	/// Used instead of Deriving from Default to not make internals depend on it.
	#[inline]
	pub fn new() -> Self {
		StringInterner{
			map   : HashMap::new(),
			values: Vec::new()
		}
	}

	/// Creates a new `StringInterner` with a given capacity.
	#[inline]
	pub fn with_capacity(cap: usize) -> Self {
		StringInterner{
			map   : HashMap::with_capacity(cap),
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
		let new_id : Sym            = self.make_symbol();
		let new_ref: InternalStrRef = new_val.as_ref().into();
		self.values.push(new_val.into().into_boxed_str());
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
}

/// Iterator over the pairs of symbols and interned string for a `StringInterner`.
pub struct Iter<'a, Sym>
	where Sym: Symbol + 'a
{
	interner: &'a StringInterner<Sym>,
	current : usize
}

impl<'a, Sym> Iter<'a, Sym>
	where Sym: Symbol + 'a
{
	/// Creates a new iterator for the given StringIterator over pairs of 
	/// symbols and their associated interned string.
	#[inline]
	fn new(interner: &'a StringInterner<Sym>) -> Self {
		Iter{
			interner: interner,
			current : 0
		}
	}
}

impl<'a, Sym> Iterator for Iter<'a, Sym>
	where Sym: Symbol + 'a
{
	type Item = (Sym, &'a str);

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		let sym = Sym::from_usize(self.current);
		match self.interner.resolve(sym) {
			Some(str) => {
				self.current += 1;
				Some((sym, str))
			},
			None => None
		}
	}

	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		use std::cmp::max;
		let rem_elems = max(0, self.interner.len() - self.current);
		(rem_elems, Some(rem_elems))
	}
}

/// Iterator over the interned strings for a `StringInterner`.
pub struct Values<'a, Sym>
	where Sym: Symbol + 'a
{
	iter: Iter<'a, Sym>
}

impl<'a, Sym> Values<'a, Sym>
	where Sym: Symbol + 'a
{
	/// Creates a new iterator for the given StringIterator over its interned strings.
	#[inline]
	fn new(interner: &'a StringInterner<Sym>) -> Self {
		Values{
			iter: interner.iter()
		}
	}
}

impl<'a, Sym> Iterator for Values<'a, Sym>
	where Sym: Symbol + 'a
{
	type Item = &'a str;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		match self.iter.next() {
			Some((_, string)) => Some(string),
			None              => None
		}
	}

	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		self.iter.size_hint()
	}
}

use std::vec;
use std::iter;
use std::marker;

impl<Sym> iter::IntoIterator for StringInterner<Sym>
	where Sym: Symbol
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

#[cfg(test)]
mod tests {
	use ::{DefaultStringInterner, InternalStrRef};

	fn make_dummy_interner() -> (DefaultStringInterner, [usize; 8]) {
		let mut interner = DefaultStringInterner::new();
		let name0 = interner.get_or_intern("foo");
		let name1 = interner.get_or_intern("bar");
		let name2 = interner.get_or_intern("baz");
		let name3 = interner.get_or_intern("foo");
		let name4 = interner.get_or_intern("rofl");
		let name5 = interner.get_or_intern("bar");
		let name6 = interner.get_or_intern("mao");
		let name7 = interner.get_or_intern("foo");
		(interner, [name0, name1, name2, name3, name4, name5, name6, name7])
	}

	#[test]
	fn internal_str_ref() {
		use std::mem;
		assert_eq!(mem::size_of::<InternalStrRef>(), mem::size_of::<&str>());

		let s0 = "Hello";
		let s1 = ", World!";
		let s2 = "Hello";
		let s3 = ", World!";
		let r0 = InternalStrRef::from_str(s0);
		let r1 = InternalStrRef::from_str(s1);
		let r2 = InternalStrRef::from_str(s2);
		let r3 = InternalStrRef::from_str(s3);
		assert_eq!(r0, r2);
		assert_eq!(r1, r3);
		assert_ne!(r0, r1);
		assert_ne!(r2, r3);

		use std::collections::hash_map::DefaultHasher;
		use std::hash::Hash;
		let mut sip = DefaultHasher::new();
		assert_eq!(r0.hash(&mut sip), s0.hash(&mut sip));
		assert_eq!(r1.hash(&mut sip), s1.hash(&mut sip));
		assert_eq!(r2.hash(&mut sip), s2.hash(&mut sip));
		assert_eq!(r3.hash(&mut sip), s3.hash(&mut sip));
	}

	#[test]
	fn intern_str() {
		let (_, names) = make_dummy_interner();
		assert_eq!(names[0], 0);
		assert_eq!(names[1], 1);
		assert_eq!(names[2], 2);
		assert_eq!(names[3], 0);
		assert_eq!(names[4], 3);
		assert_eq!(names[5], 1);
		assert_eq!(names[6], 4);
		assert_eq!(names[7], 0);
	}

	#[test]
	fn intern_string() {
		let mut interner = DefaultStringInterner::new();
		let name_0 = interner.get_or_intern("Hello".to_owned());
		let name_1 = interner.get_or_intern("World".to_owned());
		let name_2 = interner.get_or_intern("I am a String".to_owned());
		let name_3 = interner.get_or_intern("Foo".to_owned());
		let name_4 = interner.get_or_intern("Bar".to_owned());
		let name_5 = interner.get_or_intern("I am a String".to_owned());
		let name_6 = interner.get_or_intern("Next is empty".to_owned());
		let name_7 = interner.get_or_intern("".to_owned());
		let name_8 = interner.get_or_intern("I am a String".to_owned());
		let name_9 = interner.get_or_intern("I am a String".to_owned());
		let name10 = interner.get_or_intern("Foo".to_owned());

		assert_eq!(interner.len(), 7);

		assert_eq!(name_0, 0);
		assert_eq!(name_1, 1);
		assert_eq!(name_2, 2);
		assert_eq!(name_3, 3);
		assert_eq!(name_4, 4);
		assert_eq!(name_5, 2);
		assert_eq!(name_6, 5);
		assert_eq!(name_7, 6);
		assert_eq!(name_8, 2);
		assert_eq!(name_9, 2);
		assert_eq!(name10, 3);
	}

	#[test]
	fn len() {
		let (interner, _) = make_dummy_interner();
		assert_eq!(interner.len(), 5);	
	}

	#[test]
	fn get() {
		let (interner, _) = make_dummy_interner();
		assert_eq!(interner.resolve(0), Some("foo"));
		assert_eq!(interner.resolve(1), Some("bar"));
		assert_eq!(interner.resolve(2), Some("baz"));
		assert_eq!(interner.resolve(3), Some("rofl"));
		assert_eq!(interner.resolve(4), Some("mao"));
		assert_eq!(interner.resolve(5), None);
	}

	#[test]
	fn lookup_symbol() {
		let (interner, _) = make_dummy_interner();
		assert_eq!(interner.get("foo"),  Some(0));
		assert_eq!(interner.get("bar"),  Some(1));
		assert_eq!(interner.get("baz"),  Some(2));
		assert_eq!(interner.get("rofl"), Some(3));
		assert_eq!(interner.get("mao"),  Some(4));
		assert_eq!(interner.get("xD"),   None);
	}

	#[test]
	fn clear() {
		let (mut interner, _) = make_dummy_interner();
		assert_eq!(interner.len(), 5);
		interner.clear();
		assert_eq!(interner.len(), 0);
	}

	#[test]
	fn string_interner_iter_values() {
		let (interner, _) = make_dummy_interner();
		let mut it = interner.iter_values();
		assert_eq!(it.next(), Some("foo"));
		assert_eq!(it.next(), Some("bar"));
		assert_eq!(it.next(), Some("baz"));
		assert_eq!(it.next(), Some("rofl"));
		assert_eq!(it.next(), Some("mao"));
		assert_eq!(it.next(), None);
	}

	#[test]
	fn string_interner_iter() {
		let (interner, _) = make_dummy_interner();
		let mut it = interner.iter();
		assert_eq!(it.next(), Some((0, "foo")));
		assert_eq!(it.next(), Some((1, "bar")));
		assert_eq!(it.next(), Some((2, "baz")));
		assert_eq!(it.next(), Some((3, "rofl")));
		assert_eq!(it.next(), Some((4, "mao")));
		assert_eq!(it.next(), None);
	}
}

#[cfg(all(feature = "bench", test))]
mod bench {
	use super::*;
    use test::{Bencher, black_box};

	fn read_file_to_string(path: &str) -> String {
		use std::io::prelude::*;
		use std::fs::File;

		let mut f = File::open(path).expect("bench file not found");
		let mut s = String::new();

		f.read_to_string(&mut s).expect("encountered problems writing bench file to string");
		s
	}

	fn read_default_test() -> String {
		read_file_to_string("bench/input.txt")
	}

	fn empty_setup<'a>(input: &'a str) -> (Vec<&'a str>, DefaultStringInterner) {
		let lines = input.split_whitespace().collect::<Vec<&'a str>>();
		let interner = DefaultStringInterner::with_capacity(lines.len());
		(lines, interner)
	}

	fn filled_setup<'a>(input: &'a str) -> (Vec<usize>, DefaultStringInterner) {
		let (lines, mut interner) = empty_setup(&input);
		let symbols = lines.iter().map(|&line| interner.get_or_intern(line)).collect::<Vec<_>>();
		(symbols, interner)
	}

	#[bench]
	fn bench_get_or_intern_unique(bencher: &mut Bencher) {
		let input = read_default_test();
		let (lines, mut interner) = empty_setup(&input);
		bencher.iter(|| {
			for &line in lines.iter() {
				black_box(interner.get_or_intern(line));
			}
			interner.clear();
		});
	}

	#[bench]
	fn bench_resolve(bencher: &mut Bencher) {
		let input = read_default_test();
		let (symbols, interner) = filled_setup(&input);
		bencher.iter(|| {
			for &sym in symbols.iter() {
				black_box(interner.resolve(sym));
			}
		});
	}

	#[bench]
	fn bench_resolve_unchecked(bencher: &mut Bencher) {
		let input = read_default_test();
		let (symbols, interner) = filled_setup(&input);
		bencher.iter(|| {
			for &sym in symbols.iter() {
				unsafe{ black_box(interner.resolve_unchecked(sym)) };
			}
		});
	}

	#[bench]
	fn bench_iter(bencher: &mut Bencher) {
		let input = read_default_test();
		let (_, interner) = filled_setup(&input);
		bencher.iter(|| {
			for (sym, strref) in interner.iter() {
				black_box((sym, strref));
			}
		})
	}

	#[bench]
	fn bench_values_iter(bencher: &mut Bencher) {
		let input = read_default_test();
		let (_, interner) = filled_setup(&input);
		bencher.iter(|| {
			for strref in interner.iter_values() {
				black_box(strref);
			}
		})
	}

	/// Mainly needed to approximate the `into_iterator` test below.
	#[bench]
	fn bench_clone(bencher: &mut Bencher) {
		let input = read_default_test();
		let (_, interner) = filled_setup(&input);
		bencher.iter(|| {
			black_box(interner.clone());
		})
	}

	#[bench]
	fn bench_into_iterator(bencher: &mut Bencher) {
		let input = read_default_test();
		let (_, interner) = filled_setup(&input);
		bencher.iter(|| {
			for (sym, string) in interner.clone() {
				black_box((sym, string));
			}
		})
	}
}
