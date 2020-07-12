#![doc(html_root_url = "https://docs.rs/crate/string-interner/0.9.0")]
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
//! # use string_interner::DefaultStringInterner;
//! let interner = vec!["Elephant", "Tiger", "Horse", "Tiger"]
//!     .into_iter()
//!     .collect::<DefaultStringInterner>();
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
//! # use string_interner::DefaultStringInterner;
//! let interner = vec!["Earth", "Water", "Fire", "Air"]
//!     .into_iter()
//!     .collect::<DefaultStringInterner>();
//! for (sym, str) in interner {
//!     // iteration code here!
//! }
//! ```

#[cfg(test)]
mod tests;

#[cfg(feature = "serde-1")]
mod serde_impl;

pub mod backend;
mod compat;
mod internal_str;
mod interner;
mod interner2;
pub mod iter;
pub mod symbol;

use self::internal_str::InternalStr;

#[doc(inline)]
pub use self::{
    backend::{
        DefaultBackend,
        InternedStr,
    },
    interner::{
        DefaultStringInterner,
        StringInterner,
    },
    interner2::StringInterner as StringInterner2,
    symbol::{
        DefaultSymbol,
        Symbol,
    },
};
