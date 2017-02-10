TODO - List
===========

- saner interface to work with `StringInterner` (e.g. `get_or_intern` returns `Symbol` while `get` returns `Option<&str>`)
	- `get_or_intern :: String|&str -> Symbol`
	- `get :: String|&str -> Symbol`
	- `resolve :: Symbol -> Option<&str>`

- make `StringInterner`'s `get_or_intern` work for `&String`. Should work similar to `&str`.
- support different hashers (e.g. very fast fnv-hasher) for `StringInterner`.
- remove implicit impl for `Symbol` for all types but primitives (`u8`, `u16`, ..). Or maybe this feature is wanted?
- reiterate `gensym`'s name and decide if it is useful to move it to the public interface.
- implement run-time checks for `Symbol` types that may not suffice to uniquely represent internally stored strings.
  e.g. this may easily happen for `u8`-Symbols when storing more than 256 unique strings within the `StringInterner`!
  Maybe use `Result<Symbol>` type instead of directly returning a `Symbol` in `get_or_intern` would be useful.
