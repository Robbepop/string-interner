TODO - List
===========

- Make `StringInterner`'s `get_or_intern` work for `&String`. Should work similar to `&str`.
- Support different hashers (e.g. very fast fnv-hasher) for `StringInterner`.
- Decide if `StringInterner::intern` should be public. This would eliminate uniqueness of interned strings!
- Remove implicit impl for `Symbol` for all types but primitives (`u8`, `u16`, ..). Or maybe this feature is wanted?
- Implement run-time checks for `Symbol` types that may not suffice to uniquely represent internally stored strings.
  e.g. this may easily happen for `u8`-Symbols when storing more than 256 unique strings within the `StringInterner`!
  Maybe use `Result<Symbol>` type instead of directly returning a `Symbol` in `get_or_intern` would be useful.
