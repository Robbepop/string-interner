use crate::{
    backend::{
        Backend,
        InternedStr,
    },
    compat::{
        DefaultHashBuilder,
        HashMap,
    },
    DefaultBackend,
    DefaultSymbol,
    InternalStr,
    Symbol,
};
use core::{
    hash::BuildHasher,
    iter::FromIterator,
};

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
#[derive(Debug)]
pub struct StringInterner<S = DefaultSymbol, B = DefaultBackend, H = DefaultHashBuilder>
where
    S: Symbol,
    B: Backend<S>,
    H: BuildHasher,
{
    map: HashMap<InternalStr, S, H>,
    backend: B,
}

impl Default for StringInterner<DefaultSymbol, DefaultBackend, DefaultHashBuilder> {
    #[inline]
    fn default() -> Self {
        StringInterner::new()
    }
}

impl<S, B, H> Clone for StringInterner<S, B, H>
where
    S: Symbol,
    B: Backend<S> + Clone,
    for<'a> &'a B: IntoIterator<Item = (S, &'a str)>,
    H: BuildHasher + Default,
{
    fn clone(&self) -> Self {
        // We implement `Clone` manually for `StringInterner` to go around the
        // issue of shallow closing the self-referential pinned strs.
        // This was an issue with former implementations. Visit the following
        // link for more information:
        // https://github.com/Robbepop/string-interner/issues/9
        let backend = self.backend.clone();
        let map = backend
            .into_iter()
            .map(|(id, str)| (InternedStr::new(str).into(), id))
            .collect::<HashMap<_, S, H>>();
        Self { map, backend }
    }
}

impl<S, B, H> PartialEq for StringInterner<S, B, H>
where
    S: Symbol,
    B: Backend<S> + PartialEq,
    H: BuildHasher,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.len() == rhs.len() && self.backend == rhs.backend
    }
}

impl<S, B, H> Eq for StringInterner<S, B, H>
where
    S: Symbol,
    B: Backend<S> + Eq,
    H: BuildHasher,
{
}

impl<S, B, H> StringInterner<S, B, H>
where
    S: Symbol,
    B: Backend<S>,
    H: BuildHasher + Default,
{
    /// Creates a new empty `StringInterner`.
    #[inline]
    pub fn new() -> Self {
        Self {
            map: HashMap::default(),
            backend: B::default(),
        }
    }

    /// Creates a new `StringInterner` with the given initial capacity.
    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            map: HashMap::default(),
            backend: B::with_capacity(cap),
        }
    }
}

impl<S, B, H> StringInterner<S, B, H>
where
    S: Symbol,
    B: Backend<S>,
    H: BuildHasher,
{
    /// Creates a new empty `StringInterner` with the given hasher.
    #[inline]
    pub fn with_hasher(hash_builder: H) -> Self {
        StringInterner {
            map: HashMap::with_hasher(hash_builder),
            backend: B::default(),
        }
    }

    /// Creates a new empty `StringInterner` with the given initial capacity and the given hasher.
    #[inline]
    pub fn with_capacity_and_hasher(cap: usize, hash_builder: H) -> Self {
        StringInterner {
            map: HashMap::with_hasher(hash_builder),
            backend: B::with_capacity(cap),
        }
    }

    /// Returns the number of strings interned by the interner.
    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if the string interner has no interned strings.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the symbol for the given string if any.
    ///
    /// Can be used to query if a string has already been interned without interning.
    #[inline]
    pub fn get<T>(&self, string: T) -> Option<S>
    where
        T: AsRef<str>,
    {
        self.map.get(string.as_ref()).copied()
    }

    /// Interns the given string.
    ///
    /// Returns a symbol for resolution into the original string.
    #[inline]
    pub fn get_or_intern<T>(&mut self, string: T) -> S
    where
        T: AsRef<str>,
    {
        let str = string.as_ref();
        self.map.get(str).copied().unwrap_or_else(|| unsafe {
            let (interned_str, symbol) = self.backend.intern(str);
            self.map.insert(interned_str.into(), symbol);
            symbol
        })
    }

    /// Returns the string for the given symbol if any.
    #[inline]
    pub fn resolve(&self, symbol: S) -> Option<&str> {
        self.backend.resolve(symbol)
    }
}

unsafe impl<S, B, H> Send for StringInterner<S, B, H>
where
    S: Symbol + Send,
    B: Backend<S> + Send,
    H: BuildHasher,
{
}

unsafe impl<S, B, H> Sync for StringInterner<S, B, H>
where
    S: Symbol + Sync,
    B: Backend<S> + Sync,
    H: BuildHasher,
{
}

impl<'a, S, B, H, T> FromIterator<T> for StringInterner<S, B, H>
where
    S: Symbol,
    B: Backend<S>,
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

impl<'a, S, B, H, T> Extend<T> for StringInterner<S, B, H>
where
    S: Symbol,
    B: Backend<S>,
    H: BuildHasher,
    T: AsRef<str>,
{
    #[inline]
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for s in iter {
            self.get_or_intern(s.as_ref());
        }
    }
}

impl<'a, S, B, H> IntoIterator for &'a StringInterner<S, B, H>
where
    S: Symbol,
    B: Backend<S>,
    &'a B: IntoIterator<Item = (S, &'a str)>,
    H: BuildHasher,
{
    type Item = (S, &'a str);
    type IntoIter = <&'a B as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.backend.into_iter()
    }
}
