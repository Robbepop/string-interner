//! Backends for the [`StringInterner`](`crate::StringInterner`).
//!
//! The backend is the method or strategy that handles the actual interning.
//! There are trade-offs for the different kinds of backends. A user should
//! find the backend that suits their use case best.

mod bucket;
mod interned_str;
mod simple;
mod string;

#[cfg(feature = "backends")]
use self::interned_str::InternedStr;
#[cfg(feature = "backends")]
pub use self::{
    bucket::BucketBackend,
    simple::SimpleBackend,
    string::StringBackend,
};
use crate::Symbol;

#[cfg(not(feature = "backends"))]
/// Indicates that no proper backend is in use.
pub struct NoBackend<S>(core::marker::PhantomData<S>);

cfg_if::cfg_if! {
    if #[cfg(feature = "backends")] {
        /// The default backend recommended for general use.
        pub type DefaultBackend<S> = BucketBackend<S>;
    } else {
        /// The `backends` crate feature is disabled thus there is no default backend.
        pub type DefaultBackend<S> = NoBackend<S>;
    }
}

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
    /// # Note
    ///
    /// The backend must make sure that the returned symbol maps back to the
    /// original string in its [`resolve`](`Backend::resolve`) method.
    fn intern(&mut self, string: &str) -> S;

    /// Interns the given static string and returns its interned ref and symbol.
    ///
    /// # Note
    ///
    /// The backend must make sure that the returned symbol maps back to the
    /// original string in its [`resolve`](`Backend::resolve`) method.
    #[inline]
    fn intern_static(&mut self, string: &'static str) -> S {
        // The default implementation simply forwards to the normal [`intern`]
        // implementation. Backends that can optimize for this use case should
        // implement this method.
        self.intern(string)
    }

    /// Resolves the given symbol to its original string contents.
    fn resolve(&self, symbol: S) -> Option<&str>;

    /// Resolves the given symbol to its original string contents.
    ///
    /// # Safety
    ///
    /// Does not perform validity checks on the given symbol and relies
    /// on the caller to be provided with a symbol that has been generated
    /// by the [`Backend::intern`](`intern`) or
    /// [`Backend::intern_static`](`intern_static`) methods of the same
    /// interner backend.
    unsafe fn resolve_unchecked(&self, symbol: S) -> &str;
}
