//! Backends for the [`StringInterner`](`crate::StringInterner`).
//!
//! The backend is the method or strategy that handles the actual interning.
//! There are trade-offs for the different kinds of backends. A user should
//! find the backend that suits their use case best.

mod bucket;
mod buffer;
mod string;

#[cfg(feature = "backends")]
pub use self::{bucket::BucketBackend, buffer::BufferBackend, string::StringBackend};
use crate::Symbol;

/// The default backend recommended for general use.
#[cfg(feature = "backends")]
pub type DefaultBackend<'i> = StringBackend<'i, crate::DefaultSymbol>;

/// [`PhantomData`][std::marker::PhantomData] wrapper that describes how a [`Backend`]
/// implementor uses lifetime `'i` and [`B::Symbol`][Backend::Symbol].
#[allow(type_alias_bounds)] // included for clarity
type PhantomBackend<'i, B: Backend<'i>> = std::marker::PhantomData<
    // 'i is invariant,        Symbol is covariant + Send + Sync
    (core::cell::Cell<&'i ()>, fn() -> <B as Backend<'i>>::Symbol)
>;

/// Types implementing this trait may act as backends for the string interner.
///
/// The job of a backend is to actually store, manage and organize the interned
/// strings. Different backends have different trade-offs. Users should pick
/// their backend with hinsight of their personal use-case.
pub trait Backend<'i>: Default {
    /// The symbol used by the string interner backend.
    type Symbol: Symbol;

    /// Describes the lifetime of returned string.
    ///
    /// If interned strings can move between insertion this type will be
    /// `&'local str` - indicating that resolved `str` is only valid while
    /// container isn't mutably accessed.
    ///
    /// If interned strings can't move then this type is `&'container str`,
    /// indicating that resolved `str` are valid for as long as interner exists.
    type Access<'l>: AsRef<str>
    where
        Self: 'l,
        'i: 'l;

    /// The iterator over the symbols and their strings.
    type Iter<'l>: Iterator<Item = (Self::Symbol, Self::Access<'l>)>
    where
        'i: 'l,
        Self: 'l;

    /// Creates a new backend for the given capacity.
    ///
    /// The capacity denotes how many strings are expected to be interned.
    fn with_capacity(cap: usize) -> Self;

    /// Interns the given string and returns its interned ref and symbol.
    ///
    /// # Note
    ///
    /// The backend must make sure that the returned symbol maps back to the
    /// original string in its [`resolve`](`Backend::resolve`) method.
    fn intern(&mut self, string: &str) -> Self::Symbol;

    /// Interns the given static string and returns its interned ref and symbol.
    ///
    /// # Note
    ///
    /// The backend must make sure that the returned symbol maps back to the
    /// original string in its [`resolve`](`Backend::resolve`) method.
    #[inline]
    fn intern_static(&mut self, string: &'static str) -> Self::Symbol {
        // The default implementation simply forwards to the normal [`intern`]
        // implementation. Backends that can optimize for this use case should
        // implement this method.
        self.intern(string)
    }

    /// Shrink backend capacity to fit interned symbols exactly.
    fn shrink_to_fit(&mut self);

    /// Resolves the given symbol to its original string contents.
    fn resolve(&self, symbol: Self::Symbol) -> Option<Self::Access<'_>>;

    /// Resolves the given symbol to its original string contents.
    ///
    /// # Safety
    ///
    /// Does not perform validity checks on the given symbol and relies
    /// on the caller to be provided with a symbol that has been generated
    /// by the [`intern`](`Backend::intern`) or
    /// [`intern_static`](`Backend::intern_static`) methods of the same
    /// interner backend.
    unsafe fn resolve_unchecked(&self, symbol: Self::Symbol) -> Self::Access<'_>;

    /// Creates an iterator that yields all interned strings and their symbols.
    fn iter(&self) -> Self::Iter<'_>;
}
