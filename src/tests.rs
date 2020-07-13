use crate::{
    backend,
    compat::DefaultHashBuilder,
    symbol::expect_valid_symbol,
    DefaultSymbol,
    Symbol,
};

macro_rules! gen_tests_for_backend {
    ( $backend:ty ) => {
        type StringInterner =
            crate::StringInterner<DefaultSymbol, $backend, DefaultHashBuilder>;

        #[test]
        fn new_works() {
            let interner = StringInterner::new();
            assert_eq!(interner.len(), 0);
            assert!(interner.is_empty());
            let other = StringInterner::new();
            assert_eq!(interner, other);
        }

        #[test]
        fn is_empty_works() {
            let mut interner = StringInterner::new();
            assert!(interner.is_empty());
            interner.get_or_intern("a");
            assert!(!interner.is_empty());
        }

        #[test]
        fn clone_works() {
            let interner = StringInterner::new();
            let cloned = interner.clone();
            assert_eq!(interner, cloned);
        }

        #[test]
        fn get_or_intern_works() {
            let mut interner = StringInterner::new();
            // Insert 3 unique strings:
            assert_eq!(interner.get_or_intern("a").to_usize(), 0);
            assert_eq!(interner.get_or_intern("b").to_usize(), 1);
            assert_eq!(interner.get_or_intern("c").to_usize(), 2);
            assert_eq!(interner.len(), 3);
            // Insert the same 3 unique strings, yield the same symbols:
            assert_eq!(interner.get_or_intern("a").to_usize(), 0);
            assert_eq!(interner.get_or_intern("b").to_usize(), 1);
            assert_eq!(interner.get_or_intern("c").to_usize(), 2);
            assert_eq!(interner.len(), 3);
        }

        #[test]
        fn get_or_intern_static_works() {
            let mut interner = StringInterner::new();
            // Insert 3 unique strings:
            assert_eq!(interner.get_or_intern_static("a").to_usize(), 0);
            assert_eq!(interner.get_or_intern_static("b").to_usize(), 1);
            assert_eq!(interner.get_or_intern_static("c").to_usize(), 2);
            assert_eq!(interner.len(), 3);
            // Insert the same 3 unique strings, yield the same symbols:
            assert_eq!(interner.get_or_intern_static("a").to_usize(), 0);
            assert_eq!(interner.get_or_intern_static("b").to_usize(), 1);
            assert_eq!(interner.get_or_intern_static("c").to_usize(), 2);
            assert_eq!(interner.len(), 3);
        }

        #[test]
        fn resolve_works() {
            let mut interner = StringInterner::new();
            // Insert 3 unique strings:
            let symbol_a = interner.get_or_intern("a");
            let symbol_b = interner.get_or_intern("b");
            let symbol_c = interner.get_or_intern("c");
            assert_eq!(interner.len(), 3);
            // Resolve valid symbols:
            assert_eq!(interner.resolve(symbol_a), Some("a"));
            assert_eq!(interner.resolve(symbol_b), Some("b"));
            assert_eq!(interner.resolve(symbol_c), Some("c"));
            assert_eq!(interner.len(), 3);
            // Resolve invalid symbols:
            let symbol_d = expect_valid_symbol(4);
            assert_ne!(symbol_a, symbol_d);
            assert_ne!(symbol_b, symbol_d);
            assert_ne!(symbol_c, symbol_d);
            assert_eq!(interner.resolve(symbol_d), None);
        }

        #[test]
        fn get_works() {
            let mut interner = StringInterner::new();
            // Insert 3 unique strings:
            let symbol_a = interner.get_or_intern("a");
            let symbol_b = interner.get_or_intern("b");
            let symbol_c = interner.get_or_intern("c");
            assert_eq!(interner.len(), 3);
            // Get the symbols of the same 3 strings:
            assert_eq!(interner.get("a"), Some(symbol_a));
            assert_eq!(interner.get("b"), Some(symbol_b));
            assert_eq!(interner.get("c"), Some(symbol_c));
            assert_eq!(interner.len(), 3);
            // Get the symbols of some unknown strings:
            assert_eq!(interner.get("d"), None);
            assert_eq!(interner.get("e"), None);
            assert_eq!(interner.get("f"), None);
            assert_eq!(interner.len(), 3);
        }

        #[test]
        fn from_iter_works() {
            let strings = ["a", "b", "c", "d"];
            let expected = {
                let mut interner = StringInterner::new();
                for &string in &strings {
                    interner.get_or_intern(string);
                }
                interner
            };
            let actual = strings.iter().copied().collect::<StringInterner>();
            assert_eq!(actual.len(), strings.len());
            assert_eq!(actual, expected);
        }

        #[test]
        fn extend_works() {
            let strings = ["a", "b", "c", "d"];
            let expected = {
                let mut interner = StringInterner::new();
                for &string in &strings {
                    interner.get_or_intern(string);
                }
                interner
            };
            let actual = {
                let mut interner = StringInterner::new();
                interner.extend(strings.iter().copied());
                interner
            };
            assert_eq!(actual.len(), strings.len());
            assert_eq!(actual, expected);
        }
    };
}

mod bucket_backend {
    use super::*;

    gen_tests_for_backend!(backend::BucketBackend<DefaultSymbol>);
}

mod simple_backend {
    use super::*;

    gen_tests_for_backend!(backend::SimpleBackend<DefaultSymbol>);
}
