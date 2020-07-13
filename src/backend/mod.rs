//! Backends for the [`StringInterner`](`crate::StringInterner`).
//!
//! The backend is the method or strategy that handles the actual interning.
//! There are trade-offs for the different kinds of backends. A user should
//! find the backend that suits their use case best.

mod interned_str;
mod simple;

pub use self::{
    interned_str::InternedStr,
    simple::SimpleBackend,
};
use crate::{
    DefaultSymbol,
    Symbol,
};

/// The default backend recommended for general use.
pub type DefaultBackend = SimpleBackend<DefaultSymbol>;

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

    /// Interns the given string returns its interned view and its symbol.
    ///
    /// # Note
    ///
    /// The returned `InternedStr` points to an actually interned string. The
    /// backend must make sure that it never moves its interned string arounds.
    /// This is why this method is `unsafe`.
    unsafe fn intern(&mut self, string: &str) -> (InternedStr, S);

    /// Resolves the given symbol to its original string contents.
    fn resolve(&self, symbol: S) -> Option<&str>;
}
