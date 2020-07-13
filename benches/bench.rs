use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    measurement::WallTime,
    BatchSize,
    BenchmarkGroup,
    Criterion,
};
use string_interner::{
    backend::{
        Backend,
        BucketBackend,
        SimpleBackend,
    },
    DefaultHashBuilder,
    DefaultSymbol,
    StringInterner,
};

/// Alphabet containing all characters that may be put into a benchmark string.
const ALPHABET: [u8; 64] = [
    b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n',
    b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z', b'A', b'B',
    b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P',
    b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', b'0', b'1', b'2', b'3',
    b'4', b'5', b'6', b'7', b'8', b'9', b'_', b'-',
];

/// A word builder for benchmark purposes.
///
/// Creates unique words of same sizes.
struct WordBuilder {
    indices: Vec<u8>,
}

impl WordBuilder {
    /// Creates a new word builder for words with given length.
    pub fn new(word_len: usize) -> Self {
        Self {
            indices: vec![0x00; word_len],
        }
    }

    /// Fills the internal buffer with the next unique word indices.
    fn next_indices(&mut self) -> Option<&[u8]> {
        'l: for index in &mut self.indices {
            if *index == (64 - 1) {
                *index = 0;
                continue 'l
            }
            *index += 1;
            return Some(&self.indices[..])
        }
        None
    }

    /// Returns the next unique word of the same size.
    fn next_word(&mut self) -> Option<String> {
        self.next_indices()
            .map(|indices| {
                indices
                    .iter()
                    .map(|&index| {
                        assert!(index < 64);
                        ALPHABET[index as usize]
                    })
                    .collect::<Vec<_>>()
            })
            .map(|bytes| String::from_utf8(bytes).unwrap())
    }
}

impl Iterator for WordBuilder {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_word()
    }
}

/// Generates a vector of `len` unique words of the same given length.
fn generate_test_strings(len: usize, word_len: usize) -> Vec<String> {
    let words = WordBuilder::new(word_len).take(len).collect::<Vec<_>>();
    assert_eq!(words.len(), len);
    assert_eq!(words[0].len(), word_len);
    words
}

/// The number of strings that are going to be interned in the benchmarks.
const BENCH_LEN_STRINGS: usize = 100_000;

/// The length of a single interned string.
const BENCH_STRING_LEN: usize = 5;

criterion_group!(bench_resolve, bench_resolve_already_filled,);
criterion_group!(bench_get, bench_get_already_filled,);
criterion_group!(bench_iter, bench_iter_already_filled,);
criterion_group!(
    bench_get_or_intern,
    bench_get_or_intern_fill,
    bench_get_or_intern_fill_with_capacity,
    bench_get_or_intern_already_filled,
    bench_get_or_intern_static,
);
criterion_main!(bench_get_or_intern, bench_resolve, bench_get, bench_iter);

type StringInternerWith<B> = StringInterner<DefaultSymbol, B, DefaultHashBuilder>;

trait BackendBenchmark {
    const NAME: &'static str;
    type Backend: Backend<DefaultSymbol>;

    fn setup() -> StringInternerWith<Self::Backend>;
    fn setup_with_capacity(cap: usize) -> StringInternerWith<Self::Backend>;
    fn setup_filled(words: &[String]) -> StringInternerWith<Self::Backend>;
    fn setup_filled_with_ids(
        words: &[String],
    ) -> (StringInternerWith<Self::Backend>, Vec<DefaultSymbol>);
}

struct BenchBucket;
impl BackendBenchmark for BenchBucket {
    const NAME: &'static str = "BucketBackend";
    type Backend = BucketBackend<DefaultSymbol>;

    fn setup() -> StringInternerWith<Self::Backend> {
        <StringInternerWith<Self::Backend>>::new()
    }

    fn setup_with_capacity(cap: usize) -> StringInternerWith<Self::Backend> {
        <StringInternerWith<Self::Backend>>::with_capacity(cap)
    }

    fn setup_filled(words: &[String]) -> StringInternerWith<Self::Backend> {
        words.iter().collect::<StringInternerWith<Self::Backend>>()
    }

    fn setup_filled_with_ids(
        words: &[String],
    ) -> (StringInternerWith<Self::Backend>, Vec<DefaultSymbol>) {
        let mut interner = <StringInternerWith<Self::Backend>>::new();
        let mut word_ids = Vec::new();
        for word in words {
            let word_id = interner.get_or_intern(word);
            word_ids.push(word_id);
        }
        (interner, word_ids)
    }
}

struct BenchSimple;
impl BackendBenchmark for BenchSimple {
    const NAME: &'static str = "SimpleBackend";
    type Backend = SimpleBackend<DefaultSymbol>;

    fn setup() -> StringInternerWith<Self::Backend> {
        <StringInternerWith<Self::Backend>>::new()
    }

    fn setup_with_capacity(cap: usize) -> StringInternerWith<Self::Backend> {
        <StringInternerWith<Self::Backend>>::with_capacity(cap)
    }

    fn setup_filled(words: &[String]) -> StringInternerWith<Self::Backend> {
        words.iter().collect::<StringInternerWith<Self::Backend>>()
    }

    fn setup_filled_with_ids(
        words: &[String],
    ) -> (StringInternerWith<Self::Backend>, Vec<DefaultSymbol>) {
        let mut interner = <StringInternerWith<Self::Backend>>::new();
        let mut word_ids = Vec::new();
        for word in words {
            let word_id = interner.get_or_intern(word);
            word_ids.push(word_id);
        }
        (interner, word_ids)
    }
}

fn bench_get_or_intern_static(c: &mut Criterion) {
    let mut g = c.benchmark_group("get_or_intern_static");
    fn bench_for_backend<BB: BackendBenchmark>(g: &mut BenchmarkGroup<WallTime>) {
        let static_strings = &[
            "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m",
            "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z",
            "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M",
            "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
        ];
        g.bench_with_input(
            format!("{}/{}", BB::NAME, "get_or_intern"),
            static_strings,
            |bencher, words| {
                bencher.iter_batched_ref(
                    || BB::setup(),
                    |interner| {
                        for word in words.iter().copied() {
                            black_box(interner.get_or_intern(word));
                        }
                    },
                    BatchSize::SmallInput,
                )
            },
        );
        g.bench_with_input(
            format!("{}/{}", BB::NAME, "get_or_intern_static"),
            static_strings,
            |bencher, words| {
                bencher.iter_batched_ref(
                    || BB::setup(),
                    |interner| {
                        for word in words.iter().copied() {
                            black_box(interner.get_or_intern_static(word));
                        }
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }
    bench_for_backend::<BenchSimple>(&mut g);
    bench_for_backend::<BenchBucket>(&mut g);
}

fn bench_get_or_intern_fill_with_capacity(c: &mut Criterion) {
    let mut g = c.benchmark_group("get_or_intern/fill-empty/with_capacity");
    fn bench_for_backend<BB: BackendBenchmark>(g: &mut BenchmarkGroup<WallTime>) {
        g.bench_with_input(
            BB::NAME,
            &(BENCH_LEN_STRINGS, BENCH_STRING_LEN),
            |bencher, &(len_words, word_len)| {
                let words = generate_test_strings(len_words, word_len);
                bencher.iter_batched_ref(
                    || BB::setup_with_capacity(BENCH_LEN_STRINGS),
                    |interner| {
                        for word in &words {
                            black_box(interner.get_or_intern(word));
                        }
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }
    bench_for_backend::<BenchSimple>(&mut g);
    bench_for_backend::<BenchBucket>(&mut g);
}

fn bench_get_or_intern_fill(c: &mut Criterion) {
    let mut g = c.benchmark_group("get_or_intern/fill-empty/new");
    fn bench_for_backend<BB: BackendBenchmark>(g: &mut BenchmarkGroup<WallTime>) {
        g.bench_with_input(
            BB::NAME,
            &(BENCH_LEN_STRINGS, BENCH_STRING_LEN),
            |bencher, &(len_words, word_len)| {
                let words = generate_test_strings(len_words, word_len);
                bencher.iter_batched_ref(
                    || BB::setup(),
                    |interner| {
                        for word in &words {
                            black_box(interner.get_or_intern(word));
                        }
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }
    bench_for_backend::<BenchSimple>(&mut g);
    bench_for_backend::<BenchBucket>(&mut g);
}

fn bench_get_or_intern_already_filled(c: &mut Criterion) {
    let mut g = c.benchmark_group("get_or_intern/already-filled");
    fn bench_for_backend<BB: BackendBenchmark>(g: &mut BenchmarkGroup<WallTime>) {
        g.bench_with_input(
            BB::NAME,
            &(BENCH_LEN_STRINGS, BENCH_STRING_LEN),
            |bencher, &(len_words, word_len)| {
                let words = generate_test_strings(len_words, word_len);
                bencher.iter_batched_ref(
                    || BB::setup_filled(&words),
                    |interner| {
                        for word in &words {
                            black_box(interner.get_or_intern(word));
                        }
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }
    bench_for_backend::<BenchSimple>(&mut g);
    bench_for_backend::<BenchBucket>(&mut g);
}

fn bench_resolve_already_filled(c: &mut Criterion) {
    let mut g = c.benchmark_group("resolve/already-filled");
    fn bench_for_backend<BB: BackendBenchmark>(g: &mut BenchmarkGroup<WallTime>) {
        g.bench_with_input(
            BB::NAME,
            &(BENCH_LEN_STRINGS, BENCH_STRING_LEN),
            |bencher, &(len_words, word_len)| {
                let words = generate_test_strings(len_words, word_len);
                bencher.iter_batched_ref(
                    || BB::setup_filled_with_ids(&words),
                    |(interner, word_ids)| {
                        for &word_id in &*word_ids {
                            black_box(interner.resolve(word_id));
                        }
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }
    bench_for_backend::<BenchSimple>(&mut g);
    bench_for_backend::<BenchBucket>(&mut g);
}

fn bench_get_already_filled(c: &mut Criterion) {
    let mut g = c.benchmark_group("get/already-filled");
    fn bench_for_backend<BB: BackendBenchmark>(g: &mut BenchmarkGroup<WallTime>) {
        g.bench_with_input(
            BB::NAME,
            &(BENCH_LEN_STRINGS, BENCH_STRING_LEN),
            |bencher, &(len_words, word_len)| {
                let words = generate_test_strings(len_words, word_len);
                bencher.iter_batched_ref(
                    || BB::setup_filled(&words),
                    |interner| {
                        for word in &words {
                            black_box(interner.get(word));
                        }
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }
    bench_for_backend::<BenchSimple>(&mut g);
    bench_for_backend::<BenchBucket>(&mut g);
}

fn bench_iter_already_filled(c: &mut Criterion) {
    let mut g = c.benchmark_group("iter/already-filled");
    fn bench_for_backend<BB: BackendBenchmark>(g: &mut BenchmarkGroup<WallTime>)
    where
        for<'a> &'a <BB as BackendBenchmark>::Backend:
            IntoIterator<Item = (DefaultSymbol, &'a str)>,
    {
        g.bench_with_input(
            BB::NAME,
            &(BENCH_LEN_STRINGS, BENCH_STRING_LEN),
            |bencher, &(len_words, word_len)| {
                let words = generate_test_strings(len_words, word_len);
                bencher.iter_batched_ref(
                    || BB::setup_filled(&words),
                    |interner| {
                        for word in &*interner {
                            black_box(word);
                        }
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }
    bench_for_backend::<BenchSimple>(&mut g);
    bench_for_backend::<BenchBucket>(&mut g);
}
