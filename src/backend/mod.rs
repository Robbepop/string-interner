//! Backends for the [`StringInterner`](`crate::StringInterner`).
//!
//! The backend is the method or strategy that handles the actual interning.
//! There are trade-offs for the different kinds of backends. A user should
//! find the backend that suits their use case best.

mod bucket;
mod interned_str;
mod simple;

pub use self::{
    bucket::BucketBackend,
    interned_str::InternedStr,
    simple::SimpleBackend,
};
use crate::{
    DefaultSymbol,
    Symbol,
};

/// The default backend recommended for general use.
pub type DefaultBackend = BucketBackend<DefaultSymbol>;

/// Types implementing this trait may act as backends for the string interner.
///
/// The job of a backend is to actually store, manage and organize the interned
/// strings. Different backends have different trade-offs. Users should pick
/// their backend with hinsight of their personal use-case.
pub trait Backend<S>: Default
where
    S: Symbol,
{
    /// Creates a new backend for the given capacity.
    ///
    /// The capacity denotes how many strings are expected to be interned.
    fn with_capacity(cap: usize) -> Self;

    /// Interns the given string and returns its interned ref and symbol.
    ///
    /// # Safety
    ///
    /// The returned `InternedStr` points to an actually interned string. The
    /// backend must make sure that it never moves its interned string arounds.
    /// This is why this method is `unsafe`.
    unsafe fn intern(&mut self, string: &str) -> (InternedStr, S);

    /// Interns the given static string and returns its interned ref and symbol.
    ///
    /// # Safety
    ///
    /// The returned `InternedStr` should point to the static string itself.
    /// Backends should try to not allocate any interned strings in this case.
    #[inline]
    unsafe fn intern_static(&mut self, string: &'static str) -> (InternedStr, S) {
        // The default implementation simply forwards to the normal [`intern`]
        // implementation. Backends that can optimize for this use case should
        // implement this method.
        self.intern(string)
    }

    /// Resolves the given symbol to its original string contents.
    fn resolve(&self, symbol: S) -> Option<&str>;
}
