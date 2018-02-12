use {Iter, StringInterner, Symbol};
use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;
use std::ops::Deref;
use std::mem;

/// A reference to an interned string pooled in a `StringPool`.
#[derive(Copy, Clone, Debug)]
pub struct PooledStr<'pool, Sym: Symbol + 'pool = usize, H: BuildHasher + 'pool = RandomState> {
	pool: &'pool StringInterner<Sym, H>,
	sym: Sym,
}

impl<'pool, Sym: Symbol + 'pool, H: BuildHasher + 'pool> PooledStr<'pool, Sym, H> {
	/// Create a new PooledStr.
	fn new(pool: &'pool StringInterner<Sym, H>, sym: Sym) -> Self {
		PooledStr { pool, sym }
	}
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
		PooledStr::resolve(self)
	}
}

impl<'pool, Sym: Symbol + 'pool, H: BuildHasher + 'pool> PooledStr<'pool, Sym, H> {
	/// Resolves this reference to the interned string slice.
	///
	/// `PooledStr` dereferences directly to the slice, so prefer `&*pooled`.
	pub fn resolve(this: &Self) -> &str {
		unsafe { this.pool.resolve_unchecked(this.sym) }
	}
}

/// A pool for interning strings. The interned strings are given out
/// as `PooledStr` references rather than just as an opaque index.
// # Safety
// - `interner` _MUST_ be append-only for `PooledStr` to never contain a bad symbol.
// - `interner` _MUST_ outlive all loaned `PooledStr`.
#[derive(Debug, Eq, PartialEq)]
pub struct StringPool<'a, Sym: Symbol + 'a = usize, H: BuildHasher + 'a = RandomState> {
	interner: &'a mut StringInterner<Sym, H>,
}

impl<'a, Sym: Symbol, H: BuildHasher> StringPool<'a, Sym, H> {
	/// Creates a new `StringPool` backed by a given interner.
	pub fn new(interner: &'a mut StringInterner<Sym, H>) -> Self {
		StringPool { interner }
	}

	/// Interns the given value.
	///
	/// Returns a `PooledStr` reference to the interned string.
	///
	/// This either copies the contents of the string (e.g. for str)
	/// or moves them into this interner (e.g. for String).
	pub fn get_or_intern<T>(&mut self, val: T) -> PooledStr<'a, Sym, H>
		where T: Into<String> + AsRef<str>
	{
		let sym = self.interner.get_or_intern(val);
		unsafe { PooledStr::new(mem::transmute(&self.interner), sym) }
	}

	// The transmute is required to lengthen the lifetime of the interner borrow.
	// The lifetime chosen ties each `PooledStr` to the mutable borrow of the backing Interner.
	// This keeps the `PooledStr` from extending the borrow of the pool itself, rendering it useless
	// and keeps the borrow of the backing interner alive until all `PooledStr` are dead.

	/// Returns the given string's pooled reference if existent.
	pub fn get<T>(&self, val: T) -> Option<PooledStr<'a, Sym, H>>
		where T: AsRef<str>
	{
		self.interner.get(val).map(|sym| {
			unsafe { PooledStr::new(mem::transmute(&self.interner), sym) }
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
	use StringInterner;

    #[test]
    fn basic_usage() {
	    let mut interner = StringInterner::default();
        let mut pool = StringPool::new(&mut interner);
	    let a1 = pool.get_or_intern("a");
	    let a2 = pool.get("a").unwrap();
	    assert_eq!(a1, a2);
    }
}
