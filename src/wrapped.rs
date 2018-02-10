use {Iter, StringInterner, Symbol};
use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;
use std::ops::Deref;

/// A reference to an interned string pooled in a `StringPool`.
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
	/// Resolves this reference to the interned string slice.
	///
	/// `PooledStr` dereferences directly to the slice, so prefer `&*pooled`.
	pub fn resolve(this: &Self) -> &str {
		this.pool.interner.resolve(this.sym)
			.expect("PooledStr exists without entry in StringPool")
	}

	/// Resolves this reference without doing bounds checking.
	///
	/// A `PooledStr` should not be able to exist without being valid,
	/// but the regular `resolve` does the check and panic if this isn't true.
	///
	/// `PooledStr` dereferences directly to the slice, so prefer `&*pooled`.
	pub unsafe fn resolve_unchecked(this: &Self) -> &str {
		this.pool.interner.resolve_unchecked(this.sym)
	}
}

/// A pool for interning strings. The interned strings are given out
/// as `PooledStr` references rather than just as an opaque index.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct StringPool<Sym: Symbol = usize, H: BuildHasher = RandomState> {
	#[cfg_attr(feature = "serde_support",
	           serde(bound(serialize = "StringInterner<Sym, H>: ::serde::Serialize",
	                       deserialize = "StringInterner<Sym, H>: ::serde::Deserialize<'de>")))]
	interner: StringInterner<Sym, H>,
}

impl<Sym: Symbol> Default for StringPool<Sym>
	where StringInterner<Sym>: Default
{
	fn default() -> Self {
		StringPool {
			interner: Default::default(),
		}
	}
}

impl<Sym: Symbol> StringPool<Sym> {
	/// Creates a new empty `StringPool`.
	pub fn new() -> Self {
		StringPool {
			interner: StringInterner::new(),
		}
	}

	/// Creates a new `StringPool` with the given initial capacity.
	pub fn with_capacity(cap: usize) -> Self {
		StringPool {
			interner: StringInterner::with_capacity(cap),
		}
	}
}

impl<Sym: Symbol, H: BuildHasher> StringPool<Sym, H> {
	/// Creates a new empty `StringPool` with the given hasher.
	pub fn with_hasher(hasher: H) -> Self {
		StringPool {
			interner: StringInterner::with_hasher(hasher),
		}
	}

	/// Creates a new empty `StringPool` with the given initial capacity and the given hasher.
	pub fn with_capacity_and_hasher(cap: usize, hasher: H) -> Self {
		StringPool {
			interner: StringInterner::with_capacity_and_hasher(cap, hasher),
		}
	}

	/// Interns the given value.
	///
	/// Returns a `PooledStr` reference to the interned string.
	///
	/// This either copies the contents of the string (e.g. for str)
	/// or moves them into this interner (e.g. for String).
	pub fn get_or_intern<T>(&mut self, val: T) -> PooledStr<Sym, H>
		where T: Into<String> + AsRef<str>
	{
		let sym = self.interner.get_or_intern(val);
		PooledStr {
			pool: self,
			sym,
		}
	}

	/// Returns the given string's pooled reference if existent.
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

	/// Returns the number of uniquely stored Strings interned within this interner.
	pub fn len(&self) -> usize {
		self.interner.len()
	}

	/// Returns true if the string interner internes no elements.
	pub fn is_empty(&self) -> bool {
		self.interner.is_empty()
	}

	/// Returns an iterator over the interned strings.
	pub fn iter(&self) -> Iter<Sym> {
		self.interner.iter()
	}

	/// Shrinks the capacity of the interner as much as possible.
	pub fn shrink_to_fit(&mut self) {
		self.interner.shrink_to_fit()
	}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_usage() {
        let mut pool = StringPool::default();
	    let a1 = pool.get_or_intern("a");
	    let a2 = pool.get("a").unwrap();
	    assert_eq!(a1, a2);
    }
}
