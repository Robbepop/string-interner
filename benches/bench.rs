use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    BatchSize,
    Criterion,
};
use string_interner::StringInterner;

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

criterion_group!(bench_resolve, bench_resolve_already_filled,);
criterion_group!(bench_get, bench_get_already_filled,);
criterion_group!(bench_iter, bench_iter_already_filled,);
criterion_group!(
    bench_get_or_intern,
    bench_get_or_intern_fill,
    bench_get_or_intern_fill_with_capacity,
    bench_get_or_intern_already_filled,
);
criterion_main!(bench_get_or_intern, bench_resolve, bench_get, bench_iter);

const BENCH_LEN_WORDS: usize = 100_000;
const BENCH_WORD_LEN: usize = 5;

fn bench_get_or_intern_fill_with_capacity(c: &mut Criterion) {
    let mut g = c.benchmark_group("get_or_intern");
    g.bench_with_input(
        "fill empty using with_capacity",
        &(BENCH_LEN_WORDS, BENCH_WORD_LEN),
        |bencher, &(len_words, word_len)| {
            let words = generate_test_strings(len_words, word_len);
            bencher.iter_batched_ref(
                || <StringInterner>::with_capacity(BENCH_LEN_WORDS),
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

fn bench_get_or_intern_fill(c: &mut Criterion) {
    let mut g = c.benchmark_group("get_or_intern");
    g.bench_with_input(
        "fill empty",
        &(BENCH_LEN_WORDS, BENCH_WORD_LEN),
        |bencher, &(len_words, word_len)| {
            let words = generate_test_strings(len_words, word_len);
            bencher.iter_batched_ref(
                || StringInterner::default(),
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

fn bench_get_or_intern_already_filled(c: &mut Criterion) {
    let mut g = c.benchmark_group("get_or_intern");
    g.bench_with_input(
        "already filled",
        &(BENCH_LEN_WORDS, BENCH_WORD_LEN),
        |bencher, &(len_words, word_len)| {
            let words = generate_test_strings(len_words, word_len);
            bencher.iter_batched_ref(
                || words.iter().collect::<StringInterner>(),
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

fn bench_resolve_already_filled(c: &mut Criterion) {
    let mut g = c.benchmark_group("resolve");
    g.bench_with_input(
        "already filled",
        &(BENCH_LEN_WORDS, BENCH_WORD_LEN),
        |bencher, &(len_words, word_len)| {
            let words = generate_test_strings(len_words, word_len);
            bencher.iter_batched_ref(
                || {
                    let mut interner = StringInterner::default();
                    let mut word_ids = Vec::new();
                    for word in words.clone() {
                        let word_id = interner.get_or_intern(word);
                        word_ids.push(word_id);
                    }
                    (interner, word_ids)
                },
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

fn bench_get_already_filled(c: &mut Criterion) {
    let mut g = c.benchmark_group("get");
    g.bench_with_input(
        "already filled",
        &(BENCH_LEN_WORDS, BENCH_WORD_LEN),
        |bencher, &(len_words, word_len)| {
            let words = generate_test_strings(len_words, word_len);
            bencher.iter_batched_ref(
                || words.iter().collect::<StringInterner>(),
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

fn bench_iter_already_filled(c: &mut Criterion) {
    let mut g = c.benchmark_group("iter");
    g.bench_with_input(
        "already filled",
        &(BENCH_LEN_WORDS, BENCH_WORD_LEN),
        |bencher, &(len_words, word_len)| {
            let words = generate_test_strings(len_words, word_len);
            bencher.iter_batched_ref(
                || words.iter().collect::<StringInterner>(),
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
