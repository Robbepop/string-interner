# String Interner

| Continuous Integration |     Test Coverage    |  Documentation   |       Crates.io      |
|:----------------------:|:--------------------:|:----------------:|:--------------------:|
| [![travisCI][1]][2]    | [![codecov][5]][6]   | [![docs][9]][10] | [![crates][11]][12]  |

A data structure to cache strings efficiently, with minimal memory footprint and the ability to assicate
the interned strings with unique symbols.
These symbols allow for constant time comparisons and look-ups to the underlying interned string contents.
Also, iterating through the interned strings is cache efficient.

### Internals

- Internally a hashmap `M` and a vector `V` is used.
- `V` stores the contents of interned strings while `M` has internal references into the string of `V` to avoid duplicates.
- `V` stores the strings with an indirection to avoid iterator invalidation.
- Returned symbols usually have a low memory footprint and are efficiently comparable.

### Planned Features

- Safe abstraction wrapper that protects the user from the following misusages:
	- Using symbols of a different string interner instance to resolve string in another.
	- Using symbols that are already no longer valid (i.e. the associated string interner is no longer available).

## License

Licensed under either of

 * Apache license, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Dual licence: [![badge][license-mit-badge]](LICENSE-MIT) [![badge][license-apache-badge]](LICENSE-APACHE)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

## Changelog

- 0.8.0

    - Make it possible to use this crate in `no_std` environments
        - Use the new `hashbrown` crate feature together with `no_std`
    - Rename `Sym` to `DefaultSymbol`
    - Add `IntoIterator` impl for `&StringInterner`
    - Add some `#[inline]` annotations which improve performance for queries
    - Various internal improvements (uses `Pin` self-referentials now)

- 0.7.1

    - **CRITICAL** fix use after free bug in `StringInterner::clone()`
    - implement `std::iter::Extend` for `StringInterner`
    - `Sym::from_usize` now avoids using `unsafe` code
    - optimize `FromIterator` impl of `StringInterner`
    - move to Rust 2018 edition

    Thanks [YOSHIOKA Takuma](https://github.com/lo48576) for implementing this release.

- 0.7.0

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

- 0.6.3

	- fixed a bug that `StringInterner`'s `Send` impl didn't respect its generic `HashBuilder` parameter. Fixes GitHub [issue #4][gh-issue-4].

- 0.6.2

	- added `shrink_to_fit` public method to `StringInterner` - (by artemshein)

- 0.6.1

	- fixed a bug that inserting non-owning string types (e.g. `str`) was broken due to dangling pointers (Thanks to artemshein for fixing it!)

- 0.6.0

	- added optional serde serialization and deserialization support
	- more efficient and generic `PartialEq` implementation for `StringInterner`
	- made `StringInterner` generic over `BuildHasher` to allow for custom hashers

- 0.5.0

	- added `IntoIterator` trait implementation for `StringInterner`
	- greatly simplified iterator code

- 0.4.0

	- removed restrictive constraint for `Unsigned` for `Symbol`

- 0.3.3

	- added `Send` and `Sync` to `InternalStrRef` to make `StringInterner` itself `Send` and `Sync`

[1]: https://github.com/Robbepop/string-interner/workflows/Rust%20-%20Continuous%20Integration/badge.svg?branch=master
[2]: https://github.com/Robbepop/string-interner/workflows/Rust%20-%20Continuous%20Integration
[5]:  https://codecov.io/gh/robbepop/string-interner/branch/master/graph/badge.svg
[6]:  https://codecov.io/gh/Robbepop/string-interner/branch/master
[9]:  https://docs.rs/string-interner/badge.svg
[10]: https://docs.rs/string-interner
[11]: https://img.shields.io/crates/v/string-interner.svg
[12]: https://crates.io/crates/string-interner

[gh-issue-4]: (https://github.com/Robbepop/string-interner/issues/4)

[license-mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[license-apache-badge]: https://img.shields.io/badge/license-APACHE-orange.svg

[gh-user-koute]: https://github.com/koute
[gh-user-madklad]: https://github.com/matklad
