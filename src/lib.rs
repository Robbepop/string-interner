#![no_std]
#![doc(html_root_url = "https://docs.rs/crate/string-interner/0.18.0")]
#![warn(unsafe_op_in_unsafe_fn, clippy::redundant_closure_for_method_calls)]

//! Caches strings efficiently, with minimal memory footprint and associates them with
//! unique symbols. These symbols allow constant time equality comparison and look-ups to
//! the underlying interned strings.
//! 
//! For more information on purpose of string interning, refer to the corresponding
//! [wikipedia article].
//! 
//! See the [**comparison table**](crate::_docs::comparison_table) for a detailed
//! comparison summary of different backends.
//! 
//! ## Examples
//!
//! #### Interning & Symbols
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
//! #### Creation by `FromIterator`
//!
//! ```
//! # use string_interner::DefaultStringInterner;
//! let interner = ["Elephant", "Tiger", "Horse", "Tiger"]
//!     .into_iter()
//!     .collect::<DefaultStringInterner>();
//! ```
//!
//! #### Look-up
//!
//! ```
//! # use string_interner::StringInterner;
//! let mut interner = StringInterner::default();
//! let sym = interner.get_or_intern("Banana");
//! assert_eq!(interner.resolve(sym), Some("Banana"));
//! ```
//!
//! #### Iteration
//!
//! ```
//! # use string_interner::{DefaultStringInterner, Symbol};
//! let interner = <DefaultStringInterner>::from_iter(["Earth", "Water", "Fire", "Air"]);
//! for (sym, str) in &interner {
//!     println!("{} = {}", sym.to_usize(), str);
//! }
//! ```
//!
//! #### Use Different Backend
//!
//! ```
//! # use string_interner::StringInterner;
//! use string_interner::backend::BufferBackend;
//! type Interner<'i> = StringInterner<'i, BufferBackend<'i>>;
//! let mut interner = Interner::new();
//! let sym1 = interner.get_or_intern("Tiger");
//! let sym2 = interner.get_or_intern("Horse");
//! let sym3 = interner.get_or_intern("Tiger");
//! assert_ne!(sym1, sym2);
//! assert_eq!(sym1, sym3); // same!
//! ```
//!
//! #### Use Different Backend & Symbol
//!
//! ```
//! # use string_interner::StringInterner;
//! use string_interner::{backend::BucketBackend, symbol::SymbolU16};
//! type Interner<'i> = StringInterner<'i, BucketBackend<'i, SymbolU16>>;
//! let mut interner = Interner::new();
//! let sym1 = interner.get_or_intern("Tiger");
//! let sym2 = interner.get_or_intern("Horse");
//! let sym3 = interner.get_or_intern("Tiger");
//! assert_ne!(sym1, sym2);
//! assert_eq!(sym1, sym3); // same!
//! ```
//!
//! ## Backends
//!
//! The `string_interner` crate provides different backends with different strengths.<br/>
//! 
//! #### [Bucket Backend](backend/struct.BucketBackend.html)
//! 
//! Stores strings in buckets which stay allocated for the lifespan of [`StringInterner`].
//! This allows resolved symbols to be used even after new strings have been interned.
//!
//! **Ideal for:** storing strings in persistent location in memory
//!
//! #### [String Backend](backend/struct.StringBackend.html)
//! 
//! Concatenates all interned string contents into one large buffer
//! [`String`][alloc::string::String], keeping interned string lenghts in a separate
//! [`Vec`][alloc::vec::Vec].
//!
//! **Ideal for:** general use
//!
//! #### [Buffer Backend](backend/struct.BufferBackend.html)
//!
//! Concatenates all interned string contents into one large buffer
//! [`String`][alloc::string::String], and keeps interned string lenghts as prefixes.
//!
//! **Ideal for:** storing many small (<255 characters) strings
//! 
//! [Comparison table][crate::_docs::comparison_table] shows a high-level overview of
//! different backend characteristics.
//! 
//! [wikipedia article]: https://en.wikipedia.org/wiki/String_interning

#[cfg(doc)]
#[path ="docs.rs"]
pub mod _docs;

extern crate alloc;
#[cfg(feature = "std")]
#[macro_use]
extern crate std;

#[cfg(feature = "serde")]
mod serde_impl;

pub mod backend;
mod interner;
pub mod symbol;

/// A convenience [`StringInterner`] type based on the [`DefaultBackend`].
#[cfg(feature = "backends")]
pub type DefaultStringInterner<'i, B = DefaultBackend<'i>, H = DefaultHashBuilder> =
    self::interner::StringInterner<'i, B, H>;

#[cfg(feature = "backends")]
#[doc(inline)]
pub use self::backend::DefaultBackend;
#[doc(inline)]
pub use self::{
    interner::StringInterner,
    symbol::{DefaultSymbol, Symbol},
};

#[doc(inline)]
pub use hashbrown::DefaultHashBuilder;
