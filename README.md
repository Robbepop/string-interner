String-Interner
===============

A string interning data structure that was designed for minimal memory-overhead
and fast access to the underlying interned string contents.

It uses a similar API as the string interner of the Rust compiler.

Take a look into the [documentation](https://docs.rs/string-interner) to get to know how to use it!

Link to [crates.io](https://crates.io/crates/string-interner).

Warning: This library uses the nightly feature `conservative_impl_trait` as of Rust `stable-1.14`
