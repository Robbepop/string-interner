# String Interner

|        Linux        |       Windows       |       Coverage       |          Docs         |     Crates.io      |       Licence      |
|:-------------------:|:-------------------:|:--------------------:|:---------------------:|:------------------:|:------------------:|
| [![travisCI][1]][2] | [![appveyor][3]][4] | [![coveralls][5]][6] | [![licence][11]][12 ] | [![chat][9]][10]   | [![licence][7]][8] |

A string interning data structure that was designed for minimal memory overhead,
fast access to the underlying interned strings and cache-efficient iteration through its contents.

This implementation uses a similar API as the string interner of the Rust compiler.

### What is a string interner?

String internment is an efficient bi-directional mapping between strings and very cheap identifiers (symbols)
that are used as representant for a certain string instead of the string itself.

### Internals

Internally a hashmap and a vector is used. The vector stored the true contents of interned strings
while the hashmap has internal references into the internal vector to avoid duplicates.

### Planned Features

- Safe abstraction wrapper that protects the user from the following misusage

	- Using symbols of a different string interner instance to resolve string in another
	- Using symbols that are already no longer valid (i.e. the associated string interner is no longer available)

- Even more flexibility for input into the string interner

## Changelog

- 0.6.3

	- fixed a bug that `StringInterner`'s `Send` impl didn't respect its generic `HashBuilder` parameter. Fixes GitHub [issue #4](https://github.com/Robbepop/string-interner/issues/4).

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
