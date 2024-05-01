#![cfg(feature = "backends")]

use super::Backend;
use crate::{symbol::expect_valid_symbol, DefaultSymbol, Symbol};
use alloc::vec::Vec;
use core::{marker::PhantomData, mem, str};

/// An interner backend that appends all interned string information in a single buffer.
///
/// # Usage Hint
///
/// Use this backend if memory consumption is what matters most to you.
/// Note though that unlike all other backends symbol values are not contigous!
///
/// # Usage
///
/// - **Fill:** Efficiency of filling an empty string interner.
/// - **Resolve:** Efficiency of interned string look-up given a symbol.
/// - **Allocations:** The number of allocations performed by the backend.
/// - **Footprint:** The total heap memory consumed by the backend.
/// - **Contiguous:** True if the returned symbols have contiguous values.
///
/// Rating varies between **bad**, **ok**, **good** and **best**.
///
/// | Scenario    |  Rating  |
/// |:------------|:--------:|
/// | Fill        | **best** |
/// | Resolve     | **bad**  |
/// | Allocations | **best** |
/// | Footprint   | **best** |
/// | Supports `get_or_intern_static` | **no** |
/// | `Send` + `Sync` | **yes** |
/// | Contiguous  | **no**   |
#[derive(Debug)]
pub struct BufferBackend<S = DefaultSymbol> {
    len_strings: usize,
    buffer: Vec<u8>,
    marker: PhantomData<fn() -> S>,
}

/// We use a special encoding for the length of the string, as follows:
///
/// length<=254: 1-byte length, then payload
/// length>=255: 0xFF, <usize length>, 0xFF, then payload
///
/// This encoding has the following properties:
/// * strings of length<=254 use the shortest possible encoding (1 byte length)
/// * for strings of length>=255, the encoding is sometimes longer than e.g.
///   varlen (LEB128) encoding, but the relative overhead is small: on a 64-bit
///   machine the worst case is a length-255 string, which would take 2+255=257
///   bytes with, and which takes 10+255=265 bytes with this encoding. So
///   the overhead relative to LEB128 is bounded to at most 3.1%. On 32-bit or
///   smaller machines the overhead is even lower.
/// * it can be branchlessly decoded in 4 instructions on x86:
///     mov rdx, byte ptr [rdi-9]   // load the 8-byte length
///     movzx eax, byte ptr [rdi-1] // load the 1-byte length
///     cmp al, 0xFF
///     cmove rax, rdx              // select between the 1-byte and 8-byte lengths
/// * it can be decoded forwards (starting at the lowest byte) and backwards
///   (starting at the highest byte). We decode forwards when iterating over the
///   table. We decode backwards when resolving a symbol to a string.
///
/// To enable the branchless decoding of strings without under-run on the buffer,
/// we ensure that at least 10 bytes are present in the buffer before the beginning
/// of the first payload, by padding the beginning of the buffer as necessary.

impl<S> PartialEq for BufferBackend<S>
where
    S: Symbol,
{
    fn eq(&self, other: &Self) -> bool {
        self.len_strings.eq(&other.len_strings) && self.buffer.eq(&other.buffer)
    }
}

impl<S> Eq for BufferBackend<S> where S: Symbol {}

impl<S> Clone for BufferBackend<S> {
    fn clone(&self) -> Self {
        Self {
            len_strings: self.len_strings,
            buffer: self.buffer.clone(),
            marker: Default::default(),
        }
    }
}

impl<S> Default for BufferBackend<S> {
    #[cfg_attr(feature = "inline-more", inline)]
    fn default() -> Self {
        Self {
            len_strings: 0,
            buffer: Default::default(),
            marker: Default::default(),
        }
    }
}

const SHORT_LEN: usize = 1;
const LONG_LEN: usize = mem::size_of::<usize>();
const LEN_LOOKBEHIND: usize = LONG_LEN + SHORT_LEN;
const MAX_SHORT_LEN: usize = 254;
const SENTINEL_SHORT_LEN: u8 = 255;

#[inline]
fn decode_len_backwards(bytes: &[u8; LEN_LOOKBEHIND]) -> usize {
    let short_len = bytes[LEN_LOOKBEHIND - SHORT_LEN] as usize;
    let long_len_bytes = <&[u8; LONG_LEN]>::try_from(&bytes[..LONG_LEN]).unwrap();
    let long_len = usize::from_le_bytes(*long_len_bytes);
    if short_len == SENTINEL_SHORT_LEN as usize {
        long_len
    } else {
        short_len
    }
}

impl<S> BufferBackend<S>
where
    S: Symbol,
{
    /// Resolves the string for the given symbol if any.
    ///
    /// The given index is a "backwards" index.
    fn resolve_index_to_str(&self, index: usize) -> Option<&str> {
        let len_bytes = self.buffer.get(index - LEN_LOOKBEHIND..index)?;
        let len_bytes = <&[u8; LEN_LOOKBEHIND]>::try_from(len_bytes).unwrap();
        let len = decode_len_backwards(len_bytes);
        let str_bytes = self.buffer.get(index..index + len)?;
        // SAFETY: It is guaranteed by the backend that only valid strings
        //         are stored in this portion of the buffer.
        let string = unsafe { str::from_utf8_unchecked(str_bytes) };
        Some(string)
    }

    /// Resolves the string for the given symbol.
    ///
    /// The given index is a "backwards" index.
    ///
    /// # Note
    ///
    /// It is undefined behavior if the index does not resemble a string.
    ///
    /// # Safety
    ///
    /// The caller of the function has to ensure that calling this method
    /// is safe to do.
    unsafe fn resolve_index_to_str_unchecked(&self, index: usize) -> &str {
        // SAFETY: The function is marked unsafe so that the caller guarantees
        //         that required invariants are checked.
        let len_bytes = unsafe { self.buffer.get_unchecked(index - LEN_LOOKBEHIND..index) };
        let len_bytes = <&[u8; LEN_LOOKBEHIND]>::try_from(len_bytes).unwrap();
        let len = decode_len_backwards(len_bytes);
        let str_bytes = unsafe { self.buffer.get_unchecked(index..index + len) };
        // SAFETY: It is guaranteed by the backend that only valid strings
        //         are stored in this portion of the buffer.
        unsafe { str::from_utf8_unchecked(str_bytes) }
    }

    /// Pushes the given string into the buffer and returns its span.
    ///
    /// # Panics
    ///
    /// If the backend ran out of symbols.
    fn push_string(&mut self, string: &str) -> S {
        if self.buffer.is_empty() {
            // Ensure at least LONG_LEN bytes are present in the buffer, so that LEN_LOOKBEHIND works.
            self.buffer.resize(LONG_LEN, 0);
        }
        if string.len() <= MAX_SHORT_LEN {
            self.buffer.reserve(SHORT_LEN + string.len());
            self.buffer.push(string.len() as u8);
        } else {
            self.buffer
                .reserve(SHORT_LEN + LONG_LEN + SHORT_LEN + string.len());
            self.buffer.push(SENTINEL_SHORT_LEN);
            self.buffer.extend_from_slice(&string.len().to_ne_bytes());
            self.buffer.push(SENTINEL_SHORT_LEN);
        }
        let symbol = expect_valid_symbol(self.buffer.len());
        self.buffer.extend_from_slice(string.as_bytes());
        self.len_strings += 1;
        symbol
    }
}

impl<S> Backend for BufferBackend<S>
where
    S: Symbol,
{
    type Symbol = S;

    #[cfg_attr(feature = "inline-more", inline)]
    fn with_capacity(capacity: usize) -> Self {
        /// According to google the approx. word length is 5.
        const DEFAULT_STR_LEN: usize = 5;
        let bytes_per_string = DEFAULT_STR_LEN + SHORT_LEN;
        Self {
            len_strings: 0,
            buffer: Vec::with_capacity(capacity * bytes_per_string),
            marker: Default::default(),
        }
    }

    #[inline]
    fn intern(&mut self, string: &str) -> Self::Symbol {
        self.push_string(string)
    }

    #[inline]
    fn resolve(&self, symbol: Self::Symbol) -> Option<&str> {
        self.resolve_index_to_str(symbol.to_usize())
    }

    fn shrink_to_fit(&mut self) {
        self.buffer.shrink_to_fit();
    }

    #[inline]
    unsafe fn resolve_unchecked(&self, symbol: Self::Symbol) -> &str {
        // SAFETY: The function is marked unsafe so that the caller guarantees
        //         that required invariants are checked.
        unsafe { self.resolve_index_to_str_unchecked(symbol.to_usize()) }
    }
}

impl<'a, S> IntoIterator for &'a BufferBackend<S>
where
    S: Symbol,
{
    type Item = (S, &'a str);
    type IntoIter = Iter<'a, S>;

    #[cfg_attr(feature = "inline-more", inline)]
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

pub struct Iter<'a, S> {
    backend: &'a BufferBackend<S>,
    yielded: usize,
    current: usize,
}

impl<'a, S> Iter<'a, S> {
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn new(backend: &'a BufferBackend<S>) -> Self {
        Self {
            backend,
            yielded: 0,
            current: LONG_LEN,
        }
    }
}

impl<'a, S> Iterator for Iter<'a, S>
where
    S: Symbol,
{
    type Item = (S, &'a str);

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len();
        (remaining, Some(remaining))
    }

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.backend.len_strings == self.yielded {
            return None;
        }
        // Parse length forwards, not backwards. To avoid the need for padding at the *end* of the buffer,
        // when decoding forwards we use branchy code, not branchless.
        let i = self.current;
        // SAFETY: index is guaranteed valid because the iterator controls it.
        let short_len = unsafe { *self.backend.buffer.get_unchecked(i) };
        let (len, len_len) = if short_len == SENTINEL_SHORT_LEN {
            // Long-string decoding.
            self.current += 1;
            // SAFETY: index is guaranteed valid because the iterator controls it.
            let len_bytes = unsafe {
                self.backend
                    .buffer
                    .get_unchecked(i + SHORT_LEN..i + SHORT_LEN + LONG_LEN)
            };
            let len = usize::from_ne_bytes(<[u8; LONG_LEN]>::try_from(len_bytes).unwrap()) as usize;
            (len, SHORT_LEN + LONG_LEN + SHORT_LEN)
        } else {
            (short_len as usize, SHORT_LEN)
        };
        // SAFETY: index is guaranteed valid because the iterator controls it.
        let str_bytes = unsafe {
            self.backend
                .buffer
                .get_unchecked(i + len_len..i + len_len + len)
        };
        // SAFETY: payload guaranteed valid utf8, because that's how it was written to the buffer.
        let str = unsafe { str::from_utf8_unchecked(str_bytes) };
        self.current += len_len + len;
        self.yielded += 1;
        Some((expect_valid_symbol(i + len_len), str))
    }
}

impl<'a, S> ExactSizeIterator for Iter<'a, S>
where
    S: Symbol,
{
    fn len(&self) -> usize {
        self.backend.len_strings - self.yielded
    }
}
