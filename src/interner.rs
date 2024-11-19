use crate::{backend::Backend, Symbol};
use core::{
    fmt,
    fmt::{Debug, Formatter},
    hash::{BuildHasher, Hash, Hasher},
    iter::FromIterator,
};
use hashbrown::{DefaultHashBuilder, HashMap};

/// Creates the `u64` hash value for the given value using the given hash builder.
fn make_hash<T>(builder: &impl BuildHasher, value: &T) -> u64
where
    T: ?Sized + Hash,
{
    let state = &mut builder.build_hasher();
    value.hash(state);
    state.finish()
}

/// Data structure to intern and resolve strings.
///
/// Caches strings efficiently, with minimal memory footprint and associates them with unique symbols.
/// These symbols allow constant time comparisons and look-ups to the underlying interned strings.
///
/// The following API covers the main functionality:
///
/// - [`StringInterner::get_or_intern`]: To intern a new string.
///     - This maps from `string` type to `symbol` type.
/// - [`StringInterner::resolve`]: To resolve your already interned strings.
///     - This maps from `symbol` type to `string` type.
pub struct StringInterner<'i, B, H = DefaultHashBuilder>
where
    B: Backend<'i>,
{
    dedup: HashMap<<B as Backend<'i>>::Symbol, (), ()>,
    hasher: H,
    backend: B,
}

impl<'i, B, H> Debug for StringInterner<'i, B, H>
where
    B: Backend<'i> + Debug,
    <B as Backend<'i>>::Symbol: Symbol + Debug,
    H: BuildHasher,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("StringInterner")
            .field("dedup", &self.dedup)
            .field("backend", &self.backend)
            .finish()
    }
}

#[cfg(feature = "backends")]
impl<'i> Default for StringInterner<'i, crate::DefaultBackend<'i>> {
    #[cfg_attr(feature = "inline-more", inline)]
    fn default() -> Self {
        StringInterner::new()
    }
}

impl<'i, B, H> Clone for StringInterner<'i, B, H>
where
    B: Backend<'i> + Clone,
    <B as Backend<'i>>::Symbol: Symbol,
    H: BuildHasher + Clone,
{
    fn clone(&self) -> Self {
        Self {
            dedup: self.dedup.clone(),
            hasher: self.hasher.clone(),
            backend: self.backend.clone(),
        }
    }
}

impl<'i, B, H> PartialEq for StringInterner<'i, B, H>
where
    B: Backend<'i> + PartialEq,
    <B as Backend<'i>>::Symbol: Symbol,
    H: BuildHasher,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.len() == rhs.len() && self.backend == rhs.backend
    }
}

impl<'i, B, H> Eq for StringInterner<'i, B, H>
where
    B: Backend<'i> + Eq,
    <B as Backend<'i>>::Symbol: Symbol,
    H: BuildHasher,
{
}

impl<'i, B, H> StringInterner<'i, B, H>
where
    B: Backend<'i>,
    <B as Backend<'i>>::Symbol: Symbol,
    H: BuildHasher + Default,
{
    /// Creates a new empty `StringInterner`.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn new() -> Self {
        Self {
            dedup: HashMap::default(),
            hasher: Default::default(),
            backend: B::default(),
        }
    }

    /// Creates a new `StringInterner` with the given initial capacity.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            dedup: HashMap::with_capacity_and_hasher(cap, ()),
            hasher: Default::default(),
            backend: B::with_capacity(cap),
        }
    }
}

impl<'i, B, H> StringInterner<'i, B, H>
where
    B: Backend<'i>,
    <B as Backend<'i>>::Symbol: Symbol,
    H: BuildHasher,
{
    /// Creates a new empty `StringInterner` with the given hasher.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn with_hasher(hash_builder: H) -> Self {
        StringInterner {
            dedup: HashMap::default(),
            hasher: hash_builder,
            backend: B::default(),
        }
    }

    /// Creates a new empty `StringInterner` with the given initial capacity and the given hasher.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn with_capacity_and_hasher(cap: usize, hash_builder: H) -> Self {
        StringInterner {
            dedup: HashMap::with_capacity_and_hasher(cap, ()),
            hasher: hash_builder,
            backend: B::with_capacity(cap),
        }
    }

    /// Returns the number of strings interned by the interner.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn len(&self) -> usize {
        self.dedup.len()
    }

    /// Returns `true` if the string interner has no interned strings.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the symbol for the given string if any.
    ///
    /// Can be used to query if a string has already been interned without interning.
    #[inline]
    pub fn get<T>(&self, string: T) -> Option<<B as Backend<'i>>::Symbol>
    where
        T: AsRef<str>,
    {
        let string = string.as_ref();
        let Self {
            dedup,
            hasher,
            backend,
        } = self;
        let hash = make_hash(hasher, string);
        dedup
            .raw_entry()
            .from_hash(hash, |symbol| {
                // SAFETY: This is safe because we only operate on symbols that
                //         we receive from our backend making them valid.
                string == unsafe { backend.resolve_unchecked(*symbol) }.as_ref()
            })
            .map(|(&symbol, &())| symbol)
    }

    /// Interns the given string.
    ///
    /// This is used as backend by [`get_or_intern`][1] and [`get_or_intern_static`][2].
    ///
    /// [1]: [`StringInterner::get_or_intern`]
    /// [2]: [`StringInterner::get_or_intern_static`]
    #[cfg_attr(feature = "inline-more", inline)]
    fn get_or_intern_using<T>(
        &mut self,
        string: T,
        intern_fn: fn(&mut B, T) -> <B as Backend<'i>>::Symbol,
    ) -> <B as Backend<'i>>::Symbol
    where
        T: Copy + Hash + AsRef<str> + for<'a> PartialEq<&'a str>,
    {
        let Self {
            dedup,
            hasher,
            backend,
        } = self;
        let hash = make_hash(hasher, string.as_ref());
        let entry = dedup.raw_entry_mut().from_hash(hash, |symbol| {
            // SAFETY: This is safe because we only operate on symbols that
            //         we receive from our backend making them valid.
            string == unsafe { backend.resolve_unchecked(*symbol) }.as_ref()
        });
        use hashbrown::hash_map::RawEntryMut;
        let (&mut symbol, &mut ()) = match entry {
            RawEntryMut::Occupied(occupied) => occupied.into_key_value(),
            RawEntryMut::Vacant(vacant) => {
                let symbol = intern_fn(backend, string);
                vacant.insert_with_hasher(hash, symbol, (), |symbol| {
                    // SAFETY: This is safe because we only operate on symbols that
                    //         we receive from our backend making them valid.
                    let string = unsafe { backend.resolve_unchecked(*symbol) };
                    make_hash(hasher, string.as_ref())
                })
            }
        };
        symbol
    }

    /// Interns the given string.
    ///
    /// Returns a symbol for resolution into the original string.
    ///
    /// # Panics
    ///
    /// If the interner already interns the maximum number of strings possible
    /// by the chosen symbol type.
    #[inline]
    pub fn get_or_intern<T>(&mut self, string: T) -> <B as Backend<'i>>::Symbol
    where
        T: AsRef<str>,
    {
        self.get_or_intern_using(string.as_ref(), B::intern)
    }

    /// Interns the given `'static` string.
    ///
    /// Returns a symbol for resolution into the original string.
    ///
    /// # Note
    ///
    /// This is more efficient than [`StringInterner::get_or_intern`] since it might
    /// avoid some memory allocations if the backends supports this.
    ///
    /// # Panics
    ///
    /// If the interner already interns the maximum number of strings possible
    /// by the chosen symbol type.
    #[inline]
    pub fn get_or_intern_static(&mut self, string: &'static str) -> <B as Backend<'i>>::Symbol {
        self.get_or_intern_using(string, B::intern_static)
    }

    /// Shrink backend capacity to fit the interned strings exactly.
    pub fn shrink_to_fit(&mut self) {
        self.backend.shrink_to_fit()
    }

    /// Returns the string for the given `symbol`` if any.
    #[inline]
    pub fn resolve(&self, symbol: <B as Backend<'i>>::Symbol) -> Option<<B as Backend<'i>>::Access<'_>> {
        self.backend.resolve(symbol)
    }

    /// Returns the string for the given `symbol` without performing any checks.
    ///
    /// # Safety
    ///
    /// It is the caller's responsibility to provide this method with `symbol`s
    /// that are valid for the [`StringInterner`].
    #[inline]
    pub unsafe fn resolve_unchecked(&self, symbol: <B as Backend<'i>>::Symbol) -> <B as Backend<'i>>::Access<'_> {
        unsafe { self.backend.resolve_unchecked(symbol) }
    }

    /// Returns an iterator that yields all interned strings and their symbols.
    #[inline]
    pub fn iter(&self) -> <B as Backend<'i>>::Iter<'_> {
        self.backend.iter()
    }
}

impl<'i, B, H, T> FromIterator<T> for StringInterner<'i, B, H>
where
    B: Backend<'i>,
    <B as Backend<'i>>::Symbol: Symbol,
    H: BuildHasher + Default,
    T: AsRef<str>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let iter = iter.into_iter();
        let (capacity, _) = iter.size_hint();
        let mut interner = Self::with_capacity(capacity);
        interner.extend(iter);
        interner
    }
}

impl<'i, B, H, T> Extend<T> for StringInterner<'i, B, H>
where
    B: Backend<'i>,
    <B as Backend<'i>>::Symbol: Symbol,
    H: BuildHasher,
    T: AsRef<str>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for s in iter {
            self.get_or_intern(s.as_ref());
        }
    }
}

impl<'i, 'l, B, H> IntoIterator for &'l StringInterner<'i, B, H>
where
    B: Backend<'i>,
    <B as Backend<'i>>::Symbol: Symbol,
    &'l B: IntoIterator<Item = (<B as Backend<'i>>::Symbol, &'l str)>,
    H: BuildHasher,
{
    type Item = (<B as Backend<'i>>::Symbol, &'l str);
    type IntoIter = <&'l B as IntoIterator>::IntoIter;

    #[cfg_attr(feature = "inline-more", inline)]
    fn into_iter(self) -> Self::IntoIter {
        self.backend.into_iter()
    }
}
