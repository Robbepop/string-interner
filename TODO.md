
- saner interface to work with StringInterner (e.g. 'get_or_intern' returns Sym while 'get' returns 'Option<&str>')
	- get_or_intern :: String|&str -> Symbol
	- get :: String|&str -> Symbol
	- resolve :: Symbol -> Option<&str>

- make StringInterner's 'get_or_intern' work for &String.
- implement non-nightly-dependent version of 'iter()' and 'iter_interned' for StringInterner.
- support different Hashers for StringInterner.
- remove implicit impl for Symbol for all types but primitives. (or maybe this feature is good?)
- reiterate 'gensym' name and decide if it is useful to move it to the public interface.
- implement run-time checks for Symbol types that may not suffice to uniquely represent internally stored strings.
  e.g. this may easily happen for u8-Symbols when storing more than 256 unique Strings within the StringInterner!
  Maybe result type instead of directly returning a Symbol in 'get_or_intern' would be useful.
