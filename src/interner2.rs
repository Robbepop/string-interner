use crate::{
    backend::Backend,
    compat::{
        DefaultHashBuilder,
        HashMap,
    },
    DefaultBackend,
    DefaultSymbol,
    InternalStr,
    InternedStr,
    Symbol,
};
use core::hash::BuildHasher;

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
            .map(|(id, str)| (InternedStr::new(str).into(), id.into()))
            .collect::<HashMap<_, S, H>>();
        Self { map, backend }
    }
}

impl<S, B> StringInterner<S, B>
where
    S: Symbol,
    B: Backend<S>,
{
    /// Creates a new empty `StringInterner`.
    #[inline]
    pub fn new() -> Self {
        StringInterner {
            map: HashMap::new(),
            backend: B::default(),
        }
    }

    /// Creates a new `StringInterner` with the given initial capacity.
    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        StringInterner {
            map: HashMap::new(),
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

    /// Interns the given string.
    ///
    /// Returns a symbol for resolution into the original string.
    #[inline]
    pub fn get_or_intern<T>(&mut self, string: &str) -> S {
        self.map.get(string).copied().unwrap_or_else(|| unsafe {
            let (interned_str, symbol) = self.backend.intern(string);
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
