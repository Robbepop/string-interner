#![cfg(feature = "backends")]

//! An interner backend that appends all interned string information in a single buffer.
//!
//! # Usage Hint
//!
//! Use this backend if memory consumption is what matters most to you.
//! Note though that unlike all other backends symbol values are not contigous!
//!
//! # Usage
//!
//! - **Fill:** Efficiency of filling an empty string interner.
//! - **Resolve:** Efficiency of interned string look-up given a symbol.
//! - **Allocations:** The number of allocations performed by the backend.
//! - **Footprint:** The total heap memory consumed by the backend.
//! - **Contiguous:** True if the returned symbols have contiguous values.
//!
//! Rating varies between **bad**, **ok**, **good** and **best**.
//!
//! | Scenario    |  Rating  |
//! |:------------|:--------:|
//! | Fill        | **best** |
//! | Resolve     | **bad**  |
//! | Allocations | **best** |
//! | Footprint   | **best** |
//! | Supports `get_or_intern_static` | **no** |
//! | `Send` + `Sync` | **yes** |
//! | Contiguous  | **no**   |

use super::{
    Backend,
    Sliced,
};
use crate::{
    compat::Vec,
    symbol::expect_valid_symbol,
    DefaultSymbol,
    Symbol,
};
use core::{
    marker::PhantomData,
    mem,
    slice,
};

/// An interner backend that appends all interned string information in a single buffer.
///
/// See the [module-level documentation](self) for more.
#[derive(Debug)]
pub struct BufferBackend<S, Sym = DefaultSymbol>
where
    S: ?Sized + Sliced,
{
    len_strings: usize,
    buffer: Vec<u8>,
    marker: PhantomData<fn(&S) -> Sym>,
}

impl<S, Sym> PartialEq for BufferBackend<S, Sym>
where
    Sym: Symbol,
    S: ?Sized + Sliced,
{
    fn eq(&self, other: &Self) -> bool {
        self.len_strings.eq(&other.len_strings) && self.buffer.eq(&other.buffer)
    }
}

impl<S, Sym> Eq for BufferBackend<S, Sym>
where
    Sym: Symbol,
    S: ?Sized + Sliced,
{
}

impl<S, Sym> Clone for BufferBackend<S, Sym>
where
    S: ?Sized + Sliced,
{
    fn clone(&self) -> Self {
        Self {
            len_strings: self.len_strings,
            buffer: self.buffer.clone(),
            marker: Default::default(),
        }
    }
}

impl<S, Sym> Default for BufferBackend<S, Sym>
where
    S: ?Sized + Sliced,
{
    #[cfg_attr(feature = "inline-more", inline)]
    fn default() -> Self {
        Self {
            len_strings: 0,
            buffer: Default::default(),
            marker: Default::default(),
        }
    }
}

impl<S, Sym> BufferBackend<S, Sym>
where
    Sym: Symbol,
    S: ?Sized + Sliced,
    S::Element: Copy,
{
    /// Returns the next available symbol.
    #[inline]
    fn next_symbol(&self) -> Sym {
        expect_valid_symbol(self.buffer.len())
    }

    /// Resolves the string for the given symbol if any.
    ///
    /// # Note
    ///
    /// Returns the string from the given index if any as well
    /// as the index of the next string in the buffer.
    fn resolve_index_to_str(&self, index: usize) -> Option<(&S, usize)> {
        let buffer = self.buffer.get(index..)?;
        let (str_len, decoded_bytes) = decode_var_usize(buffer)?;
        let str_bytes_len = str_len * mem::size_of::<S::Element>();
        let index_str = index + decoded_bytes;
        let str_bytes = self.buffer.get(index_str..index_str + str_bytes_len)?;

        // SAFETY: It is guaranteed by the backend that only valid strings
        //         are stored in this portion of the buffer.
        unsafe {
            let string =
                slice::from_raw_parts(str_bytes.as_ptr().cast::<S::Element>(), str_len);
            Some((S::from_slice(string), index_str + str_bytes_len))
        }
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
    unsafe fn resolve_index_to_str_unchecked(&self, index: usize) -> &S {
        // SAFETY: The function is marked unsafe so that the caller guarantees
        //         that required invariants are checked.
        let buffer = unsafe { self.buffer.get_unchecked(index..) };

        // SAFETY: The function is marked unsafe so that the caller guarantees
        //         that required invariants are checked.
        let (str_len, decoded_bytes) = unsafe { decode_var_usize_unchecked(buffer) };

        let str_bytes_len = str_len * mem::size_of::<S::Element>();

        let start_str = index + decoded_bytes;

        // SAFETY: The function is marked unsafe so that the caller guarantees
        //         that required invariants are checked.
        let str_bytes = unsafe {
            self.buffer
                .get_unchecked(start_str..start_str + str_bytes_len)
        };

        // SAFETY: It is guaranteed by the backend that only valid strings
        //         are stored in this portion of the buffer.
        unsafe {
            let string =
                slice::from_raw_parts(str_bytes.as_ptr().cast::<S::Element>(), str_len);
            S::from_slice(string)
        }
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
    fn push_string(&mut self, string: &S) -> Sym {
        let symbol = self.next_symbol();
        let string = string.to_slice();
        let str_len = string.len();

        // Safety: The `S::Element: Copy` bound ensures that only bit-copiable
        //         types are casted to bytes, making them valid for storing
        //         in our buffer.
        let str_bytes = unsafe {
            let n_bytes = str_len * mem::size_of::<S::Element>();
            slice::from_raw_parts(string.as_ptr().cast::<u8>(), n_bytes)
        };
        self.encode_var_usize(str_len);
        self.buffer.extend_from_slice(str_bytes);
        self.len_strings += 1;
        symbol
    }
}

impl<S, Sym> Backend for BufferBackend<S, Sym>
where
    Sym: Symbol,
    S: ?Sized + Sliced,
    S::Element: Copy,
{
    type Str = S;
    type Symbol = Sym;

    #[cfg_attr(feature = "inline-more", inline)]
    fn with_capacity(capacity: usize) -> Self {
        /// We encode the `usize` string length into the buffer as well.
        const LEN_USIZE: usize = mem::size_of::<usize>();
        /// According to google the approx. word length is 5.
        const DEFAULT_STR_LEN: usize = 5;
        let bytes_per_string = DEFAULT_STR_LEN * LEN_USIZE * mem::size_of::<S::Element>();
        Self {
            len_strings: 0,
            buffer: Vec::with_capacity(capacity * bytes_per_string),
            marker: Default::default(),
        }
    }

    #[inline]
    fn intern(&mut self, string: &S) -> Self::Symbol {
        self.push_string(string)
    }

    #[inline]
    fn resolve(&self, symbol: Self::Symbol) -> Option<&S> {
        self.resolve_index_to_str(symbol.to_usize())
            .map(|(string, _next_str_index)| string)
    }

    fn shrink_to_fit(&mut self) {
        self.buffer.shrink_to_fit();
    }

    #[inline]
    unsafe fn resolve_unchecked(&self, symbol: Self::Symbol) -> &S {
        // SAFETY: The function is marked unsafe so that the caller guarantees
        //         that required invariants are checked.
        unsafe { self.resolve_index_to_str_unchecked(symbol.to_usize()) }
    }
}

/// Encodes the value using variable length encoding into the buffer.
///
/// Returns the amount of bytes used for the encoding.
#[inline]
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
///
/// # Safety
///
/// The caller has to make sure that the buffer contains the necessary
/// bytes needed to properly decode a valid `usize` value.
#[inline]
unsafe fn decode_var_usize_unchecked(buffer: &[u8]) -> (usize, usize) {
    let first = unsafe { *buffer.get_unchecked(0) };
    if first <= 0x7F_u8 {
        return (first as usize, 1)
    }
    let mut result: usize = 0;
    let mut i = 0;
    loop {
        let byte = unsafe { *buffer.get_unchecked(i) };
        let shifted = ((byte & 0x7F_u8) as usize) << ((i * 7) as u32);
        result += shifted;
        if (byte & 0x80) == 0 {
            break
        }
        i += 1;
    }
    (result, i + 1)
}

/// Decodes from a variable length encoded `usize` from the buffer.
///
/// Returns the decoded value as first return value.
/// Returns the number of decoded bytes as second return value.
fn decode_var_usize(buffer: &[u8]) -> Option<(usize, usize)> {
    if !buffer.is_empty() && buffer[0] <= 0x7F_u8 {
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
    #[cfg_attr(any(miri, tarpaulin), ignore)]
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
    #[cfg_attr(any(miri, tarpaulin), ignore)]
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
    #[cfg_attr(any(miri, tarpaulin), ignore)]
    fn encode_var_usize_4_bytes_01_works() {
        assert_encode_var_usize_4_bytes(2usize.pow(21)..2usize.pow(24));
    }

    #[test]
    #[cfg_attr(any(miri, tarpaulin), ignore)]
    fn encode_var_usize_4_bytes_02_works() {
        assert_encode_var_usize_4_bytes(2usize.pow(24)..2usize.pow(26));
    }

    #[test]
    #[cfg_attr(any(miri, tarpaulin), ignore)]
    fn encode_var_usize_4_bytes_03_works() {
        assert_encode_var_usize_4_bytes(2usize.pow(26)..2usize.pow(27));
    }

    #[test]
    #[cfg_attr(any(miri, tarpaulin), ignore)]
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

impl<'a, S, Sym> IntoIterator for &'a BufferBackend<S, Sym>
where
    Sym: Symbol,
    S: ?Sized + Sliced,
    S::Element: Copy,
{
    type Item = (Sym, &'a S);
    type IntoIter = Iter<'a, S, Sym>;

    #[cfg_attr(feature = "inline-more", inline)]
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

/// Iterator for a [`BufferBackend`](crate::backend::buffer::BufferBackend)
/// that returns all of its interned strings.
pub struct Iter<'a, S, Sym>
where
    S: ?Sized + Sliced,
{
    backend: &'a BufferBackend<S, Sym>,
    yielded: usize,
    current: usize,
}

impl<'a, S, Sym> Iter<'a, S, Sym>
where
    S: ?Sized + Sliced,
{
    #[cfg_attr(feature = "inline-more", inline)]
    pub(super) fn new(backend: &'a BufferBackend<S, Sym>) -> Self {
        Self {
            backend,
            yielded: 0,
            current: 0,
        }
    }
}

impl<'a, S, Sym> Iterator for Iter<'a, S, Sym>
where
    Sym: Symbol,
    S: ?Sized + Sliced,
    S::Element: Copy,
{
    type Item = (Sym, &'a S);

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len();
        (remaining, Some(remaining))
    }

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.backend.resolve_index_to_str(self.current).and_then(
            |(string, next_string_index)| {
                let symbol = Sym::try_from_usize(self.current)?;
                self.current = next_string_index;
                self.yielded += 1;
                Some((symbol, string))
            },
        )
    }
}

impl<'a, S, Sym> ExactSizeIterator for Iter<'a, S, Sym>
where
    Sym: Symbol,
    S: ?Sized + Sliced,
    S::Element: Copy,
{
    fn len(&self) -> usize {
        self.backend.len_strings - self.yielded
    }
}
