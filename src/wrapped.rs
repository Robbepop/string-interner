#![allow(missing_docs)]

use {Iter, StringInterner, Symbol, Values};
use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;
use std::ops::Deref;

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct PooledStr<'pool, Sym: Symbol + 'pool = usize, H: BuildHasher + 'pool = RandomState> {
	#[cfg_attr(feature = "serde_support",
	           serde(bound(serialize = "&'pool StringPool<Sym, H>: ::serde::Serialize",
	                       deserialize = "&'pool StringPool<Sym, H>: ::serde::Deserialize<'de>")))]
	pool: &'pool StringPool<Sym, H>,
	sym: Sym,
}

impl<'pool, Sym: Symbol + 'pool, H: BuildHasher + 'pool> Eq for PooledStr<'pool, Sym, H> {}
impl<'pool, Sym: Symbol + 'pool, H: BuildHasher + 'pool> PartialEq<Self> for PooledStr<'pool, Sym, H> {
	fn eq(&self, other: &Self) -> bool {
		self.sym == other.sym && ::std::ptr::eq(self.pool, other.pool)
	}
}

impl<'pool, Sym: Symbol + 'pool, H: BuildHasher + 'pool> Deref for PooledStr<'pool, Sym, H> {
	type Target = str;
	fn deref(&self) -> &str {
		// in the future, maybe use resolve_unchecked
		PooledStr::resolve(self)
	}
}

impl<'pool, Sym: Symbol + 'pool, H: BuildHasher + 'pool> PooledStr<'pool, Sym, H> {
	pub fn resolve(this: &Self) -> &str {
		this.pool.interner.resolve(this.sym)
			.expect("PooledStr exists without entry in StringPool")
	}

	pub unsafe fn resolve_unchecked(this: &Self) -> &str {
		this.pool.interner.resolve_unchecked(this.sym)
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct StringPool<Sym: Symbol = usize, H: BuildHasher = RandomState> {
	#[cfg_attr(feature = "serde_support",
	           serde(bound(serialize = "StringInterner<Sym, H>: ::serde::Serialize",
	                       deserialize = "StringInterner<Sym, H>: ::serde::Deserialize<'de>")))]
	interner: StringInterner<Sym, H>,
}

impl<Sym: Symbol> Default for StringPool<Sym> {
	fn default() -> Self {
		StringPool {
			interner: Default::default(),
		}
	}
}

impl<Sym: Symbol> StringPool<Sym> {
	pub fn new() -> Self {
		StringPool {
			interner: StringInterner::new(),
		}
	}

	pub fn with_capacity(cap: usize) -> Self {
		StringPool {
			interner: StringInterner::with_capacity(cap),
		}
	}
}

impl<Sym: Symbol, H: BuildHasher> StringPool<Sym, H> {
	pub fn with_hasher(hasher: H) -> Self {
		StringPool {
			interner: StringInterner::with_hasher(hasher),
		}
	}

	pub fn with_capacity_and_hasher(cap: usize, hasher: H) -> Self {
		StringPool {
			interner: StringInterner::with_capacity_and_hasher(cap, hasher),
		}
	}

	pub fn get_or_intern<T>(&mut self, val: T) -> PooledStr<Sym, H>
		where T: Into<String> + AsRef<str>
	{
		let sym = self.interner.get_or_intern(val);
		PooledStr {
			pool: self,
			sym,
		}
	}

	pub fn get<T>(&self, val: T) -> Option<PooledStr<Sym, H>>
		where T: AsRef<str>
	{
		self.interner.get(val).map(|sym| {
			PooledStr {
				pool: self,
				sym,
			}
		})
	}

	pub fn len(&self) -> usize {
		self.interner.len()
	}

	pub fn is_empty(&self) -> bool {
		self.interner.is_empty()
	}

	pub fn iter(&self) -> Iter<Sym> {
		self.interner.iter()
	}

	pub fn iter_values(&self) -> Values<Sym> {
		self.interner.iter_values()
	}

	pub fn shrink_to_fit(&mut self) {
		self.interner.shrink_to_fit()
	}
}
