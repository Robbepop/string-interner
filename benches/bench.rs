use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    BatchSize,
    Throughput,
    BenchmarkId,
    Criterion,
};
use fnv::FnvHasher;
use lazy_static::lazy_static;
use std::hash::BuildHasherDefault;
use string_interner::StringInterner;

const ALPHABET: [u8; 64] = [
    b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j',
    b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't',
    b'u', b'v', b'w', b'x', b'y', b'z',
    b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J',
    b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T',
    b'U', b'V', b'W', b'X', b'Y', b'Z',
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9',
    b'_', b'-',
];

struct WordBuilder {
    fragments: Vec<u8>,
}

impl WordBuilder {
    pub fn new(word_len: usize) -> Self {
        Self { fragments: Vec::with_capacity(word_len) }
    }

    fn word_len(&self) -> usize {
        self.fragments.len()
    }

    fn next_fragments(&mut self) -> &[u8] {
        for (n, frag) in self.fragments.iter_mut().enumerate() {
            if *frag == (64 - 1) {
                *frag = 0;
                continue;
            } else {
                *frag += 1;
                break;
            }
        }
        &self.fragments[..]
    }

    fn next_word(&mut self) -> String {
        let mut word = String::with_capacity(self.word_len());
        let fragment = self.next_fragments();
        for n in &self.fragments {
            word.push(ALPHABET[*n as usize] as char);
        }
        word
    }
}

fn generate_test_strings(len: usize, word_len: usize) -> Vec<String> {
    let mut builder = WordBuilder::new(word_len);
    let mut words = Vec::with_capacity(len);
    for _ in 0..len {
        words.push(builder.next_word());
    }
    words
}

#[test]
fn test_unique_string_iter() {
    assert_eq!(
        generate_test_strings(5, 5),
        vec![
            "aaaaa".to_owned(),
            "baaaa".to_owned(),
            "caaaa".to_owned(),
            "daaaa".to_owned(),
            "eaaaa".to_owned(),
        ]
    );
}


    pub struct FilledSetup<H>
    where
        H: BuildHasher,
    {
        lines: &'static [&'static str],
        interner: StringInterner<DefaultSymbol, H>,
        symbols: Vec<DefaultSymbol>,
    }

    impl FilledSetup<RandomState> {
        fn new() -> Self {
            let lines = bench_lines();
            let mut interner = StringInterner::with_capacity(lines.len());
            let symbols = lines
                .into_iter()
                .map(|&line| interner.get_or_intern(line))
                .collect::<Vec<_>>();
            FilledSetup {
                lines,
                interner,
                symbols,
            }
        }
    }

    impl<S> FilledSetup<BuildHasherDefault<S>>
    where
        S: Hasher + Default,
    {
        fn new_with_hasher() -> Self {
            let lines = bench_lines();
            let build_hasher = BuildHasherDefault::<S>::default();
            let mut interner =
                StringInterner::with_capacity_and_hasher(lines.len(), build_hasher);
            let symbols = lines
                .into_iter()
                .map(|&line| interner.get_or_intern(line))
                .collect::<Vec<_>>();
            FilledSetup {
                lines,
                interner,
                symbols,
            }
        }
    }

    impl<H> FilledSetup<H>
    where
        H: BuildHasher,
    {
        /// Returns an iterator over the benchmark lines.
        pub fn lines(&self) -> impl Iterator<Item = &'static str> {
            self.lines.iter().copied()
        }

        pub fn filled_interner(&self) -> &StringInterner<DefaultSymbol, H> {
            &self.interner
        }

        pub fn filled_interner_mut(&mut self) -> &mut StringInterner<DefaultSymbol, H> {
            &mut self.interner
        }

        pub fn interned_symbols(&self) -> &[DefaultSymbol] {
            &self.symbols
        }
    }

    pub fn filled_setup() -> FilledSetup<RandomState> {
        FilledSetup::new()
    }
}

const BENCH_SIZES: [usize; 5] = [
    10, 50, 250, 1000, 10_000
];

fn bench_get_or_intern_empty(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_or_intern_empty");
    for bench_size in &BENCH_SIZES {
        group.throughput(Throughput::Elements(*bench_size as u64));
        group.bench_with_input(
            BenchmarkId::new("get_or_intern_empty", bench_size),
            bench_size,
            |b, &bench_size| {
                let setup = utils::empty_setup();
                b.iter(|| {
                    let mut interner = setup.empty_interner();
                    for line in setup.lines().take(bench_size) {
                        black_box(interner.get_or_intern(line));
                    }
                });
            },
        );
    }
}

fn bench_get_or_intern_filled(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_or_intern_filled");
    let mut setup = utils::filled_setup();
    for bench_size in &BENCH_SIZES {
        group.throughput(Throughput::Elements(*bench_size as u64));
        group.bench_with_input(
            BenchmarkId::new("get_or_intern_filled", bench_size),
            bench_size,
            |b, &bench_size| {
                b.iter(|| {
                    for line in setup.lines().take(bench_size) {
                        black_box(setup.filled_interner_mut().get_or_intern(line));
                    }
                });
            },
        );
    }
}

criterion_group!(
    bench_get_or_intern,
    // bench_get_or_intern_empty,
    bench_get_or_intern_filled,
    /* get_or_intern_empty,
     * get_or_intern_filled,
     * get_empty,
     * get_filled,
     * resolve,
     * resolve_unchecked,
     * iter,
     * values_iter,
     * clone,
     * into_iterator, */
);
criterion_main!(bench_get_or_intern,);

// #[bench]
// fn from_iterator(bencher: &mut Bencher) {
//     let setup = empty_setup();
//     bencher.iter(|| {
//         black_box(DefaultStringInterner::from_iter(
//             setup.lines().into_iter().map(|&line| line),
//         ))
//     })
// }

// #[bench]
// fn get_or_intern_empty(bencher: &mut Bencher) {
//     let setup = empty_setup();
//     bencher.iter(|| {
//         let mut interner = setup.empty_interner();
//         for &line in setup.lines() {
//             black_box(interner.get_or_intern(line));
//         }
//     });
// }

// #[bench]
// fn get_or_intern_filled(bencher: &mut Bencher) {
//     let mut setup = filled_setup();
//     bencher.iter(|| {
//         for &line in setup.lines() {
//             black_box(setup.filled_interner_mut().get_or_intern(line));
//         }
//     });
// }

// #[bench]
// fn get_empty(bencher: &mut Bencher) {
//     let setup = empty_setup();
//     bencher.iter(|| {
//         let interner = setup.empty_interner();
//         for &line in setup.lines() {
//             black_box(interner.get(line));
//         }
//     });
// }

// #[bench]
// fn get_filled(bencher: &mut Bencher) {
//     let setup = filled_setup();
//     bencher.iter(|| {
//         for &line in setup.lines() {
//             black_box(setup.filled_interner().get(line));
//         }
//     });
// }

// #[bench]
// fn resolve(bencher: &mut Bencher) {
//     let setup = filled_setup();
//     bencher.iter(|| {
//         for &sym in setup.interned_symbols() {
//             black_box(setup.filled_interner().resolve(sym));
//         }
//     });
// }

// #[bench]
// fn resolve_unchecked(bencher: &mut Bencher) {
//     let setup = filled_setup();
//     bencher.iter(|| {
//         for &sym in setup.interned_symbols() {
//             black_box(unsafe { setup.filled_interner().resolve_unchecked(sym) });
//         }
//     });
// }

// #[bench]
// fn iter(bencher: &mut Bencher) {
//     let setup = filled_setup();
//     bencher.iter(|| {
//         for (sym, str_ref) in setup.filled_interner().iter() {
//             black_box((sym, str_ref));
//         }
//     })
// }

// #[bench]
// fn values_iter(bencher: &mut Bencher) {
//     let setup = filled_setup();
//     bencher.iter(|| {
//         for str_ref in setup.filled_interner().iter_values() {
//             black_box(str_ref);
//         }
//     })
// }

// /// Mainly needed to approximate the `into_iterator` test below.
// #[bench]
// fn clone(bencher: &mut Bencher) {
//     let setup = filled_setup();
//     bencher.iter(|| {
//         black_box(setup.filled_interner().clone());
//     })
// }

// /// This benchmark performs an internal `StringInterner::clone` so that
// /// has to be subtracted for the real timing of this operation.
// #[bench]
// fn into_iterator(bencher: &mut Bencher) {
//     let setup = filled_setup();
//     bencher.iter(|| {
//         for (sym, string) in setup.filled_interner().clone().into_iter() {
//             black_box((sym, string));
//         }
//     })
// }

// mod fnv {
//     use super::*;

//     type FnvBuildHasher = BuildHasherDefault<FnvHasher>;

//     fn empty_fnv_setup() -> EmptySetup<FnvBuildHasher> {
//         EmptySetup::<FnvBuildHasher>::new_with_hasher()
//     }

//     fn filled_fnv_setup() -> FilledSetup<BuildHasherDefault<FnvHasher>> {
//         FilledSetup::new_with_hasher()
//     }

//     #[bench]
//     fn new_empty(bencher: &mut Bencher) {
//         let setup = empty_fnv_setup();
//         bencher.iter(|| {
//             for &_line in setup.lines() {
//                 black_box(setup.empty_interner());
//             }
//         })
//     }

//     #[bench]
//     fn get_or_intern_empty(bencher: &mut Bencher) {
//         let setup = empty_fnv_setup();
//         bencher.iter(|| {
//             let mut interner = setup.empty_interner();
//             for &line in setup.lines() {
//                 black_box(interner.get_or_intern(line));
//             }
//         })
//     }

//     #[bench]
//     fn get_or_intern_filled(bencher: &mut Bencher) {
//         let mut setup = filled_fnv_setup();
//         bencher.iter(|| {
//             for &line in setup.lines() {
//                 black_box(setup.filled_interner_mut().get_or_intern(line));
//             }
//         });
//     }

//     #[bench]
//     fn get_empty(bencher: &mut Bencher) {
//         let setup = empty_fnv_setup();
//         bencher.iter(|| {
//             let interner = setup.empty_interner();
//             for &line in setup.lines() {
//                 black_box(interner.get(line));
//             }
//         })
//     }

//     #[bench]
//     fn get_filled(bencher: &mut Bencher) {
//         let setup = filled_fnv_setup();
//         bencher.iter(|| {
//             for &line in setup.lines() {
//                 black_box(setup.filled_interner().get(line));
//             }
//         });
//     }
// }
