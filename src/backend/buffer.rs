#![cfg(feature = "backends")]

use super::Backend;
use crate::{
    compat::Vec,
    symbol::expect_valid_symbol,
    Symbol,
};
use core::{
    marker::PhantomData,
    mem,
    str,
};

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
/// | Fill        | **good** |
/// | Resolve     | **ok**   |
/// | Allocations | **best** |
/// | Footprint   | **best** |
/// | Supports `get_or_intern_static` | **no** |
/// | `Send` + `Sync` | **yes** |
/// | Contiguous  | **no**   |
#[derive(Debug)]
pub struct BufferBackend<S> {
    len_strings: usize,
    buffer: Vec<u8>,
    marker: PhantomData<fn() -> S>,
}

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

impl<S> BufferBackend<S>
where
    S: Symbol,
{
    /// Returns the next available symbol.
    fn next_symbol(&self) -> S {
        expect_valid_symbol(self.buffer.len())
    }

    /// Resolves the string for the given symbol if any.
    ///
    /// # Note
    ///
    /// Returns the string from the given index if any as well
    /// as the index of the next string in the buffer.
    fn resolve_index_to_str(&self, index: usize) -> Option<(&str, usize)> {
        let (str_len, str_len_bytes) =
            self.buffer.get(index..).map(decode_var_usize).flatten()?;
        let start_str = index + str_len_bytes;
        let str_bytes = self.buffer.get(start_str..start_str + str_len)?;
        // SAFETY: It is guaranteed by the backend that only valid strings
        //         are stored in this portion of the buffer.
        let string = unsafe { str::from_utf8_unchecked(str_bytes) };
        Some((string, start_str + str_len))
    }

    /// Resolves the string for the given symbol.
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
        let slice_len = unsafe { self.buffer.get_unchecked(index..) };
        let (str_len, str_len_bytes) = decode_var_usize(slice_len).unwrap_or_else(|| {
            panic!(
                "could not decode variable `usize` from bytes: {:?}",
                slice_len
            )
        });
        let start_str = index + str_len_bytes;
        let str_bytes =
            // SAFETY: The function is marked unsafe so that the caller guarantees
            //         that required invariants are checked.
            unsafe { self.buffer.get_unchecked(start_str..start_str + str_len) };
        // SAFETY: It is guaranteed by the backend that only valid strings
        //         are stored in this portion of the buffer.
        unsafe { str::from_utf8_unchecked(str_bytes) }
    }

    /// Pushes the given value onto the buffer with `var7` encoding.
    ///
    /// Returns the amount of `var7` encoded bytes.
    #[inline]
    fn encode_var_usize(&mut self, value: usize) -> usize {
        encode_var_usize(&mut self.buffer, value)
    }

    /// Pushes the given string into the buffer and returns its span.
    ///
    /// # Panics
    ///
    /// If the backend ran out of symbols.
    fn push_string(&mut self, string: &str) -> S {
        let symbol = self.next_symbol();
        let str_len = string.len();
        let str_bytes = string.as_bytes();
        self.encode_var_usize(str_len);
        self.buffer.extend(str_bytes);
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
        /// We encode the `usize` string length into the buffer as well.
        const LEN_USIZE: usize = mem::size_of::<usize>();
        /// According to google the approx. word length is 5.
        const DEFAULT_STR_LEN: usize = 5;
        let bytes_per_string = DEFAULT_STR_LEN * LEN_USIZE;
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
            .map(|(string, _next_str_index)| string)
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

/// Encodes the value using variable length encoding into the buffer.
///
/// Returns the amount of bytes used for the encoding.
fn encode_var_usize(buffer: &mut Vec<u8>, mut value: usize) -> usize {
    if value <= 0x7F {
        // Shortcut the common case for low value.
        buffer.push(value as u8);
        return 1
    }
    let mut len_chunks = 0;
    loop {
        let mut chunk = (value as u8) & 0x7F_u8;
        value >>= 7;
        chunk |= ((value != 0) as u8) << 7;
        buffer.push(chunk);
        len_chunks += 1;
        if value == 0 {
            break
        }
    }
    len_chunks
}

/// Decodes from a variable length encoded `usize` from the buffer.
///
/// Returns the decoded value as first return value.
/// Returns the number of decoded bytes as second return value.
fn decode_var_usize(buffer: &[u8]) -> Option<(usize, usize)> {
    if buffer.get(0)? <= &0x7F_u8 {
        // Shortcut the common case for low values.
        return Some((buffer[0] as usize, 1))
    }
    let mut result: usize = 0;
    let mut i = 0;
    loop {
        let byte = *buffer.get(i)?;
        let shifted = ((byte & 0x7F_u8) as usize).checked_shl((i * 7) as u32)?;
        result = result.checked_add(shifted)?;
        if (byte & 0x80) == 0 {
            break
        }
        i += 1;
    }
    Some((result, i + 1))
}

#[cfg(test)]
mod tests {
    use super::{
        decode_var_usize,
        encode_var_usize,
    };

    #[test]
    fn encode_var_usize_1_byte_works() {
        let mut buffer = Vec::new();
        for i in 0..2usize.pow(7) {
            buffer.clear();
            assert_eq!(encode_var_usize(&mut buffer, i), 1);
            assert_eq!(buffer, [i as u8]);
            assert_eq!(decode_var_usize(&buffer), Some((i, 1)));
        }
    }

    #[test]
    fn encode_var_usize_2_bytes_works() {
        let mut buffer = Vec::new();
        for i in 2usize.pow(7)..2usize.pow(14) {
            buffer.clear();
            assert_eq!(encode_var_usize(&mut buffer, i), 2);
            assert_eq!(buffer, [0x80 | ((i & 0x7F) as u8), (0x7F & (i >> 7) as u8)]);
            assert_eq!(decode_var_usize(&buffer), Some((i, 2)));
        }
    }

    #[test]
    fn encode_var_usize_3_bytes_works() {
        let mut buffer = Vec::new();
        for i in 2usize.pow(14)..2usize.pow(21) {
            buffer.clear();
            assert_eq!(encode_var_usize(&mut buffer, i), 3);
            assert_eq!(
                buffer,
                [
                    0x80 | ((i & 0x7F) as u8),
                    0x80 | (0x7F & (i >> 7) as u8),
                    (0x7F & (i >> 14) as u8),
                ]
            );
            assert_eq!(decode_var_usize(&buffer), Some((i, 3)));
        }
    }

    /// Allows to split up the test into multiple fragments that can run in parallel.
    fn assert_encode_var_usize_4_bytes(range: core::ops::Range<usize>) {
        let mut buffer = Vec::new();
        for i in range {
            buffer.clear();
            assert_eq!(encode_var_usize(&mut buffer, i), 4);
            assert_eq!(
                buffer,
                [
                    0x80 | ((i & 0x7F) as u8),
                    0x80 | (0x7F & (i >> 7) as u8),
                    0x80 | (0x7F & (i >> 14) as u8),
                    (0x7F & (i >> 21) as u8),
                ]
            );
            assert_eq!(decode_var_usize(&buffer), Some((i, 4)));
        }
    }

    #[test]
    fn encode_var_usize_4_bytes_01_works() {
        assert_encode_var_usize_4_bytes(2usize.pow(21)..2usize.pow(24));
    }

    #[test]
    fn encode_var_usize_4_bytes_02_works() {
        assert_encode_var_usize_4_bytes(2usize.pow(24)..2usize.pow(26));
    }

    #[test]
    fn encode_var_usize_4_bytes_03_works() {
        assert_encode_var_usize_4_bytes(2usize.pow(26)..2usize.pow(27));
    }

    #[test]
    fn encode_var_usize_4_bytes_04_works() {
        assert_encode_var_usize_4_bytes(2usize.pow(27)..2usize.pow(28));
    }

    #[test]
    fn encode_var_u32_max_works() {
        let mut buffer = Vec::new();
        let i = u32::MAX as usize;
        assert_eq!(encode_var_usize(&mut buffer, i), 5);
        assert_eq!(buffer, [0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
        assert_eq!(decode_var_usize(&buffer), Some((i, 5)));
    }

    #[test]
    fn encode_var_u64_max_works() {
        let mut buffer = Vec::new();
        let i = u64::MAX as usize;
        assert_eq!(encode_var_usize(&mut buffer, i), 10);
        assert_eq!(
            buffer,
            [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]
        );
        assert_eq!(decode_var_usize(&buffer), Some((i, 10)));
    }

    #[test]
    fn decode_var_fail() {
        // Empty buffer.
        assert_eq!(decode_var_usize(&[]), None);
        // Missing buffer bytes.
        assert_eq!(decode_var_usize(&[0x80]), None);
        // Out of range encoded value.
        // assert_eq!(
        //     decode_var_usize(&[
        //         0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x03
        //     ]),
        //     None,
        // );
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
            current: 0,
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
        self.backend.resolve_index_to_str(self.current).and_then(
            |(string, next_string_index)| {
                let symbol = S::try_from_usize(self.current)?;
                self.current = next_string_index;
                self.yielded += 1;
                Some((symbol, string))
            },
        )
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
