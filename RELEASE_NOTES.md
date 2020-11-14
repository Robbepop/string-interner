# Release Notes

## 0.12.1 - 2020/11/14

- The `BucketBackend` now implements `Send` + `Sync`.
- Implemented some minor internal improvements.

## 0.12.0 - 2020/07/15

- Make `DefaultBackend` generic over its symbol type.
	- This simplifies type ascription of string interners that do not use the
	  default symbol type.
	  - E.g. `StringInterner<usize>` is now possible to write (again).
- Add `backends` crate feature.
	- Enabled by default.
	- Disable this if you do not use any of the backends provided by the
	  `string-interner` crate.

## 0.11.3 - 2020/07/15

- Add `Symbol` implementation for `usize`.

## 0.11.2 - 2020/07/15

- Add new `StringBackend` that is optimized for memory allocations and footprint.
	- Use it if your memory constraints are most important to you.

## 0.11.1 - 2020/07/14

Special thanks to [Ten0](https://github.com/Ten0) for help with this release!

- Remove usage of `unsafe` in `Symbol::try_from_usize` methods.
- Remove no longer correct `unsafe impls` for `Send` and `Sync` for `StringInterner`.
- Add new crate feature `more-inline` that puts more `#[inline]` on public methods.
	- The new `more-inline` crate feature is enabled by default. If you want to
	  turn it off use `--no-default-features`.
	- Enabling `more-inline` also enables `hashbrown/more-inline`.
- Remove `&B: IntoIter` trait bound from `Clone` impl of `StringInterner`

## 0.11.0 - 2020/07/14

Thanks a lot (again) to [CAD97](https://dev.to/cad97) who is the vanguard
of the technical improvements in this release with their
[blog post](https://dev.to/cad97/string-interners-in-rust-797).

- Significantly improved `StringInterner`'s memory consumption independend
	from the used internment backend.
- Significantly improved `StringInterner`'s throughput for interning strings.
- Change the `Backend` trait:
	- `intern` is no longer `unsafe`
	- `intern` returns `S` (symbol type) instead of `(InternedStr, S)`
	- same as above for `intern_static`
	- add `unsafe fn resolve_unchecked` which does the same as `resolve`
		but explicitely without bounds checking
- No longer export `backend::InternedStr` type
- Make `hashbrown` a mandatory dependency.
	**Note:** We depend on it for the moment for its `raw_entry` API that has not yet been
	stabilized for Rust. Also benchmarks show that it is 20-30% faster than Rust's
	hashmap implementation.
- Benchmarks now show performance when using `FxBuildHasher` as build hasher.

## 0.10.1 - 2020/07/14

- Allow to intern `&'static str` using `get_or_intern_static` API.
	- This is a common use case and more efficient since the interner can
		skip some allocations in this special case.
- Fix bug in `SymbolU16` and `SymbolU32` that instantiating them with values
	greater or equal to `u16::MAX` or `u32::MAX` respectively caused them to
	panic instead of returning `None`.
	- Special thanks to [Ten0](https://github.com/Ten0) for reporting the issue!
- Add a bunch of additional unit tests to further solifidy the implementation.

## 0.10.0 - 2020/07/13

Special thanks to [CAD97](https://dev.to/cad97) who motivated me to craft this
release through [their blog post](https://dev.to/cad97/string-interners-in-rust-797)
"String interners in Rust".
Also I want to thank [matklad](https://github.com/matklad) who wrote a nice
[blog post](https://matklad.github.io/2020/03/22/fast-simple-rust-interner.html)
that inspired the design of the new `BucketBackend` for `StringInterner`.

- Implement pluggable backends for `StringInterner`.
	Uses the new `BucketBackend` by default which results in significant
	performance boosts and lower memory consumption as well as fewer overall
	memory allocations.

	This makes it possible for dependencies to alter the behavior of internment.
	The `string-interner` crate comes with 2 predefined backends:
	1. `SimpleBackend`: Which is how the `StringInterner` of previous versions
		worked by default. It performs one allocation per interned string.
	2. `BucketBackend`: Tries to minimize memory allocations and packs
		interned strings densely. This is the new default behavior for this crate.
- Due to the above introduction of backends some APIs have been removed:
	- `reserve`
	- `capacity`
	- the entire `iter` module
		- Note: Simple iteration through the `StringInterer`'s interned strings
				and their symbols is still possible if the used backend supports
				iteration.
	- `resolve_unchecked`: Has no replacement, yet but might be reintroduced
							in future versions again.
	- `shrink_to_fit`: The API design was never really a good fit for interners.

## 0.9.0 - 2020/07/12

- Remove `Ord` trait bound from `Symbol` trait
	- Also change `Symbol::from_usize(usize) -> Self` to `Symbol::try_from_usize(usize) -> Option<Self>`
- Minor performance improvements for `DefaultSymbol::try_from_usize`
- Put all iterator types into the `iter` sub module
- Put all symbol types into the `symbol` sub module
- Add new symbol types:
	- `SymbolU16`: 16-bit wide symbol
	- `SymbolU32`: 32-bit wide symbol (default)
	- `SymbolUsize`: same size as `usize`
- Various internal improvements and reorganizations

## 0.8.0 - 2020/07/12

- Make it possible to use this crate in `no_std` environments
	- Use the new `hashbrown` crate feature together with `no_std`
- Rename `Sym` to `DefaultSymbol`
- Add `IntoIterator` impl for `&StringInterner`
- Add some `#[inline]` annotations which improve performance for queries
- Various internal improvements (uses `Pin` self-referentials now)

## 0.7.1 - 2019/09/01

- **CRITICAL** fix use after free bug in `StringInterner::clone()`
- implement `std::iter::Extend` for `StringInterner`
- `Sym::from_usize` now avoids using `unsafe` code
- optimize `FromIterator` impl of `StringInterner`
- move to Rust 2018 edition

Thanks [YOSHIOKA Takuma](https://github.com/lo48576) for implementing this release.

## 0.7.0 - 2019/08/07

- changed license from MIT to MIT/APACHE2.0
- removed generic impl of `Symbol` for types that are `From<usize>` and `Into<usize>`
- removed `StringInterner::clear` API since its usage breaks invariants
- added `StringInterner::{capacity, reserve}` APIs
- introduced a new default symbol type `Sym` that is a thin wrapper around `NonZeroU32` (idea by [koute][gh-user-koute])
- made `DefaultStringInterner` a type alias for the new `StringInterner<Sym>`
- added convenient `FromIterator` impl to `StringInterner<S: Sym>`
- dev
	- rewrote all unit tests (serde tests are still missing)
	- entirely refactored benchmark framework
	- added `html_root_url` to crate root

Thanks [matklad][gh-user-madklad] for suggestions and impulses

## 0.6.4 - 2019/09/04

- **CRITICAL:** fix use after free bug in `StringInterner::clone` implementation.

## 0.6.3 - 2017/09/20

- fixed a bug that `StringInterner`'s `Send` impl didn't respect its generic `HashBuilder` parameter. Fixes GitHub [issue #4][gh-issue-4].

## 0.6.2 - 2017/08/13

- added `shrink_to_fit` public method to `StringInterner` - (by artemshein)

## 0.6.1  - 2017/07/31

- fixed a bug that inserting non-owning string types (e.g. `str`) was broken due to dangling pointers (Thanks to artemshein for fixing it!)

## 0.6.0 - 2017/07/09

- added optional serde serialization and deserialization support
- more efficient and generic `PartialEq` implementation for `StringInterner`
- made `StringInterner` generic over `BuildHasher` to allow for custom hashers

## 0.5.0 - 2017/07/08

- added `IntoIterator` trait implementation for `StringInterner`
- greatly simplified iterator code

## 0.4.0 - 2017/05/20

- removed restrictive constraint for `Unsigned` for `Symbol`

## 0.3.3 - 2017/02/27

- added `Send` and `Sync` to `InternalStrRef` to make `StringInterner` itself `Send` and `Sync`

## 0.2.1 - 2017/02/10

## 0.2.0 - 2017/02/10

## 0.1.0 - 2017/02/06

[gh-issue-4]: (https://github.com/Robbepop/string-interner/issues/4)

[gh-user-koute]: https://github.com/koute
[gh-user-madklad]: https://github.com/matklad
