#![no_std]
#![doc(html_root_url = "https://docs.rs/crate/string-interner/0.18.0")]
#![warn(unsafe_op_in_unsafe_fn, clippy::redundant_closure_for_method_calls)]

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
//! let interner = ["Elephant", "Tiger", "Horse", "Tiger"]
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
//! # use string_interner::{DefaultStringInterner, Symbol};
//! let interner = <DefaultStringInterner>::from_iter(["Earth", "Water", "Fire", "Air"]);
//! for (sym, str) in &interner {
//!     println!("{} = {}", sym.to_usize(), str);
//! }
//! ```
//!
//! ### Example: Use Different Backend
//!
//! ```
//! # use string_interner::StringInterner;
//! use string_interner::backend::BufferBackend;
//! type Interner = StringInterner<BufferBackend>;
//! let mut interner = Interner::new();
//! let sym1 = interner.get_or_intern("Tiger");
//! let sym2 = interner.get_or_intern("Horse");
//! let sym3 = interner.get_or_intern("Tiger");
//! assert_ne!(sym1, sym2);
//! assert_eq!(sym1, sym3); // same!
//! ```
//!
//! ### Example: Use Different Backend & Symbol
//!
//! ```
//! # use string_interner::StringInterner;
//! use string_interner::{backend::BucketBackend, symbol::SymbolU16};
//! type Interner = StringInterner<BucketBackend<SymbolU16>>;
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
//! The `string_interner` crate provides different backends with different strengths.
//! The table below compactly shows when to use which backend according to the following
//! performance characteristics and properties.
//!
//! | **Property** | **BucketBackend** | **StringBackend** | **BufferBackend** | | Explanation |
//! |:-------------|:-----------------:|:-----------------:|:-----------------:|:--|:--|
//! | Fill              | 🤷 | 👍 | ⭐ | | Efficiency of filling an empty string interner. |
//! | Fill Duplicates   | 1) | 1) | 1) | | Efficiency of filling a string interner with strings that are already interned. |
//! | Resolve           | ⭐ | 👍 | 👎 | | Efficiency of resolving a symbol of an interned string. |
//! | Resolve Unchecked | 👍 | 👍 | ⭐ 2) | | Efficiency of unchecked resolving a symbol of an interned string. |
//! | Allocations       | 🤷 | 👍 | ⭐ | | The number of allocations performed by the backend. |
//! | Footprint         | 🤷 | 👍 | ⭐ | | The total heap memory consumed by the backend. |
//! | Iteration         | ⭐ | 👍 | 👎 | | Efficiency of iterating over the interned strings. |
//! |                   | | | | | |
//! | Contiguous        | ✅ | ✅ | ❌ | | The returned symbols have contiguous values. |
//! | Stable Refs       | ✅ | ❌ | ❌ | | The interned strings have stable references. |
//! | Static Strings    | ✅ | ❌ | ❌ | | Allows to intern `&'static str` without heap allocations. |
//!
//! 1. Performance of interning pre-interned string is the same for all backends since
//!    this is implemented in the `StringInterner` front-end via a `HashMap` query for
//!    all `StringInterner` instances.
//!
//! 2. `BufferBackend` is slow with checked resolving because its internal representation
//!    is extremely sensible to the correctness of the symbols, thus a lot of checks
//!    are performed. If you will only use symbols provided by the same instance of
//!    `BufferBackend`, `resolve_unchecked` is a lot faster.
//!
//! ### Legend
//!
//! | ⭐ | **best performance** | 👍 | **good performance** | 🤷 | **okay performance** | 👎 | **bad performance** |
//! |-|-|-|-|-|-|-|-|
//!
//! ## When to use which backend?
//!
//! ### Bucket Backend
//!
//! Given the table above the `BucketBackend` might seem inferior to the other backends.
//! However, it allows to efficiently intern `&'static str` and avoids deallocations.
//!
//! ### String Backend
//!
//! Overall the `StringBackend` performs really well and therefore is the backend
//! that the `StringInterner` uses by default.
//!
//! ### Buffer Backend
//!
//! The `BufferBackend` is in some sense similar to the `StringBackend` on steroids.
//! Some operations are even slightly more efficient and it consumes less memory.
//! However, all this is at the costs of a less efficient resolution of symbols.
//! Note that the symbols generated by the `BufferBackend` are not contiguous.
//!
//! ## Customizing String Hashing
//!
//! To ensure only one copy of each string is interned, [`StringInterner`] relies on [hashbrown]'s
//! [hashmap](hashbrown::HashMap), which necessitates choosing a hashing function for hashing the
//! strings.
//!
//! By default, [`StringInterner`] will use hashbrown's [`DefaultHashBuilder`], which should be
//! appropriate for most users. However, you may customize the hash function via
//! [`StringInterner`]'s second type parameter:
//!
//! ```
//! use std::hash::RandomState;
//! use string_interner::{StringInterner, DefaultBackend};
//!
//! // create a StringInterner with the default backend but using std's RandomState hasher
//! let interned_strs: StringInterner<DefaultBackend, RandomState> = StringInterner::new();
//! ```
//!
//! NB: as of hashbrown v0.15.2, the [`DefaultHashBuilder`] is [foldhash's
//! RandomState](https://docs.rs/foldhash/latest/foldhash/fast/struct.RandomState.html), which
//! relies on a one-time random initialization of shared global state; if you need stable hashes
//! then you may wish to use [foldhash's
//! FixedState](https://docs.rs/foldhash/latest/foldhash/fast/struct.FixedState.html) (or similar)
//! instead.

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
pub type DefaultStringInterner<B = DefaultBackend, H = DefaultHashBuilder> =
    self::interner::StringInterner<B, H>;

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
