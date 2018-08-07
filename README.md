# String Interner

|        Linux        |       Windows       |       Coverage      |        Docs        |     Crates.io      |
|:-------------------:|:-------------------:|:-------------------:|:------------------:|:------------------:|
| [![travisCI][1]][2] | [![appveyor][3]][4] | [![coverage][5]][6] | [![docs][11]][12 ] | [![chat][9]][10]   |

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

[1]: https://travis-ci.org/Robbepop/string-interner.svg?branch=master
[2]: https://travis-ci.org/Robbepop/string-interner
[3]: https://ci.appveyor.com/api/projects/status/16fc9l6rtroo4xqd?svg=true
[4]: https://ci.appveyor.com/project/Robbepop/string-interner/branch/master
[5]: https://coveralls.io/repos/github/Robbepop/string-interner/badge.svg?branch=master
[6]: https://coveralls.io/github/Robbepop/string-interner?branch=master
[7]: https://img.shields.io/badge/license-MIT-blue.svg
[8]: ./LICENCE
[9]: https://img.shields.io/crates/v/string-interner.svg
[10]: https://crates.io/crates/string-interner
[11]: https://docs.rs/string-interner/badge.svg
[12]: https://docs.rs/string-interner

[gh-issue-4]: (https://github.com/Robbepop/string-interner/issues/4)

[license-mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[license-apache-badge]: https://img.shields.io/badge/license-APACHE-orange.svg

[gh-user-koute]: https://github.com/koute
[gh-user-madklad]: https://github.com/matklad
