mod allocator;

use allocator::TracingAllocator;
use string_interner::{
    backend,
    DefaultHashBuilder,
    DefaultSymbol,
    Symbol,
};

#[global_allocator]
static ALLOCATOR: TracingAllocator = TracingAllocator::new();

/// Creates the symbol `S` from the given `usize`.
///
/// # Panics
///
/// Panics if the conversion is invalid.
#[inline]
pub(crate) fn expect_valid_symbol<S>(index: usize) -> S
where
    S: Symbol,
{
    S::try_from_usize(index).expect("encountered invalid symbol")
}

/// Stats for the backend.
pub trait BackendStats {
    /// The expected minimum memory overhead for this string interner backend.
    const MIN_OVERHEAD: f64;
    /// The expected maximum memory overhead for this string interner backend.
    const MAX_OVERHEAD: f64;
    /// The name of the backend for debug display purpose.
    const NAME: &'static str;
}

impl BackendStats for backend::BucketBackend<DefaultSymbol> {
    const MIN_OVERHEAD: f64 = 2.45;
    const MAX_OVERHEAD: f64 = 3.25;
    const NAME: &'static str = "BucketBackend";
}

impl BackendStats for backend::SimpleBackend<DefaultSymbol> {
    const MIN_OVERHEAD: f64 = 2.25;
    const MAX_OVERHEAD: f64 = 2.85;
    const NAME: &'static str = "SimpleBackend";
}

impl BackendStats for backend::StringBackend<DefaultSymbol> {
    const MIN_OVERHEAD: f64 = 1.70;
    const MAX_OVERHEAD: f64 = 2.55;
    const NAME: &'static str = "StringBackend";
}

macro_rules! gen_tests_for_backend {
    ( $backend:ty ) => {
        type StringInterner =
            string_interner::StringInterner<DefaultSymbol, $backend, DefaultHashBuilder>;

        fn profile_memory_usage(words: &[String]) -> f64 {
            ALLOCATOR.reset();
            ALLOCATOR.start_profiling();
            let mut interner = StringInterner::new();
            ALLOCATOR.end_profiling();

            for word in words {
                ALLOCATOR.start_profiling();
                interner.get_or_intern(word);
                ALLOCATOR.end_profiling();
            }

            let stats = ALLOCATOR.stats();
            let len_allocations = stats.len_allocations();
            let len_deallocations = stats.len_deallocations();
            let current_allocated_bytes = stats.current_allocated_bytes();
            let total_allocated_bytes = stats.total_allocated_bytes();

            assert_eq!(interner.len(), words.len());

            println!(
                "\
                \n\t- # words         = {}\
                \n\t- # allocations   = {}\
                \n\t- # deallocations = {}\
                \n\t- allocated bytes = {}\
                \n\t- requested bytes = {}\
                ",
                words.len(),
                len_allocations, len_deallocations, current_allocated_bytes, total_allocated_bytes,
            );

            let ideal_memory_usage = words.len() * words[0].len();
            let memory_usage_overhead =
                (current_allocated_bytes as f64) / (ideal_memory_usage as f64);
            println!("\t- ideal allocated bytes  = {}", ideal_memory_usage);
            println!("\t- actual allocated bytes = {}", current_allocated_bytes);
            println!("\t- % actual overhead      = {:.02}%", memory_usage_overhead * 100.0);

            memory_usage_overhead
        }

        #[test]
        #[cfg_attr(miri, ignore)]
        #[cfg_attr(not(feature = "test-allocations"), ignore)]
        fn test_memory_consumption() {
            let len_words = 1_000_000;
            let words = (0..).take(len_words).map(|i| {
                format!("{:20}", i)
            }).collect::<Vec<_>>();

            println!();
            println!("Benchmark Memory Usage for {}", <$backend as BackendStats>::NAME);
            let mut min_overhead = None;
            let mut max_overhead = None;
            for i in 0..10 {
                let len_words = 100_000 * (i+1);
                let words = &words[0..len_words];
                let overhead = profile_memory_usage(words);
                if min_overhead.map(|min| overhead < min).unwrap_or(true) {
                    min_overhead = Some(overhead);
                }
                if max_overhead.map(|max| overhead > max).unwrap_or(true) {
                    max_overhead = Some(overhead);
                }
            }
            let actual_min_overhead = min_overhead.unwrap();
            let actual_max_overhead = max_overhead.unwrap();
            let expect_min_overhead = <$backend as BackendStats>::MIN_OVERHEAD;
            let expect_max_overhead = <$backend as BackendStats>::MAX_OVERHEAD;

            println!();
            println!("- % min. overhead = {:.02}%", actual_min_overhead * 100.0);
            println!("- % max. overhead = {:.02}%", actual_max_overhead * 100.0);

            assert!(
                actual_min_overhead < expect_min_overhead,
                "{} string interner backend minimum memory overhead is greater than expected. expected = {:?}, actual = {:?}",
                <$backend as BackendStats>::NAME,
                expect_min_overhead,
                actual_min_overhead,
            );
            assert!(
                actual_max_overhead < expect_max_overhead,
                "{} string interner backend maximum memory overhead is greater than expected. expected = {:?}, actual = {:?}",
                <$backend as BackendStats>::NAME,
                expect_max_overhead,
                actual_max_overhead,
            );
        }

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
            let mut interner = StringInterner::new();
            assert_eq!(interner.get_or_intern("a").to_usize(), 0);

            let mut cloned = interner.clone();
            assert_eq!(interner, cloned);
            // And the clone should have the same interned values
            assert_eq!(cloned.get_or_intern("a").to_usize(), 0);
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

mod string_backend {
    use super::*;

    gen_tests_for_backend!(backend::StringBackend<DefaultSymbol>);
}
