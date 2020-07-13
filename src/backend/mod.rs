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

/// TODO: Docs
pub type DefaultBackend = SimpleBackend<DefaultSymbol>;

/// TODO: Docs
pub trait Backend<S>: Default
where
    S: Symbol,
{
    /// TODO: Docs
    fn with_capacity(cap: usize) -> Self;
    /// TODO: Docs
    unsafe fn intern(&mut self, string: &str) -> (InternedStr, S);
    /// TODO: Docs
    fn resolve(&self, symbol: S) -> Option<&str>;
}
