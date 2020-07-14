#![doc(html_root_url = "https://docs.rs/crate/string-interner/0.10.1")]
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

//! Caches strings efficiently, with minimal memory footprint and associates them with unique symbols.
//! These symbols allow constant time comparisons and look-ups to the underlying interned strings.
//!
//! ### Example: Interning & Symbols
//!
//! ```
//! use string_interner::StringInterner;
//!
//! let mut interner = StringInterner::default();
//! let sym0 = interner.get_or_intern("Elephant");
//! let sym1 = interner.get_or_intern("Tiger");
//! let sym2 = interner.get_or_intern("Horse");
//! let sym3 = interner.get_or_intern("Tiger");
//! assert_ne!(sym0, sym1);
//! assert_ne!(sym0, sym2);
//! assert_ne!(sym1, sym2);
//! assert_eq!(sym1, sym3); // same!
//! ```
//!
//! ### Example: Creation by `FromIterator`
//!
//! ```
//! # use string_interner::StringInterner;
//! let interner = vec!["Elephant", "Tiger", "Horse", "Tiger"]
//!     .into_iter()
//!     .collect::<StringInterner>();
//! ```
//!
//! ### Example: Look-up
//!
//! ```
//! # use string_interner::StringInterner;
//! let mut interner = StringInterner::default();
//! let sym = interner.get_or_intern("Banana");
//! assert_eq!(interner.resolve(sym), Some("Banana"));
//! ```
//!
//! ### Example: Iteration
//!
//! ```
//! # use string_interner::StringInterner;
//! let interner = vec!["Earth", "Water", "Fire", "Air"]
//!     .into_iter()
//!     .collect::<StringInterner>();
//! for (sym, str) in &interner {
//!     // iteration code here!
//! }
//! ```

#[cfg(test)]
mod tests;

#[cfg(feature = "serde-1")]
mod serde_impl;

pub mod backend;
mod compat;
mod interner;
pub mod symbol;

#[doc(inline)]
pub use self::{
    backend::DefaultBackend,
    compat::DefaultHashBuilder,
    interner::StringInterner,
    symbol::{
        DefaultSymbol,
        Symbol,
    },
};
