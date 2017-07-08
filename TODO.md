TODO - List
===========

This is a list of features that may be planned for future versions of this crate.

- Implement optional support for serde serialization and deserialization.
- Make `StringInterner`'s `get_or_intern` work for `&String`. Should work similar to `&str`.
- Decide if `StringInterner::intern` should be public. This would eliminate uniqueness of interned strings!
- Implement run-time checks for `Symbol` types that may not suffice to uniquely represent internally stored strings.
  e.g. this may easily happen for `u8`-Symbols when storing more than 256 unique strings within the `StringInterner`!
  Maybe use `Result<Symbol>` type instead of directly returning a `Symbol` in `get_or_intern` would be useful.
