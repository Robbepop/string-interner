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

/// An interner backend that appends all interned strings together.
///
/// # Note
///
/// Implementation inspired by [CAD97's](https://github.com/CAD97) research
/// project [`strena`](https://github.com/CAD97/strena).
///
/// # Usage
///
/// - **Fill:** Efficiency of filling an empty string interner.
/// - **Resolve:** Efficiency of interned string look-up given a symbol.
/// - **Allocations:** The number of allocations performed by the backend.
/// - **Footprint:** The total heap memory consumed by the backend.
///
/// Rating varies between **bad**, **ok** and **good**.
///
/// | Scenario    |  Rating  |
/// |:------------|:--------:|
/// | Fill        | **good** |
/// | Resolve     | **good** |
/// | Allocations | **good** |
/// | Footprint   | **good** |
/// | Supports `get_or_intern_static` | **no** |
/// | `Send` + `Sync` | **yes** |
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
        const LEN_USIZE: usize = mem::size_of::<usize>();
        let start_str = index + LEN_USIZE;
        self.buffer
            .get(index..start_str)
            .map(|slice| {
                let mut bytes_len = [0; LEN_USIZE];
                bytes_len.copy_from_slice(slice);
                usize::from_le_bytes(bytes_len)
            })
            .and_then(|str_len| {
                let str_bytes = self.buffer.get(start_str..start_str + str_len)?;
                // SAFETY: It is guaranteed by the backend that only valid strings
                //         are stored in this portion of the buffer.
                let string = unsafe { str::from_utf8_unchecked(str_bytes) };
                Some((string, start_str + str_len))
            })
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
        const LEN_USIZE: usize = mem::size_of::<usize>();
        let start_str = index + LEN_USIZE;
        // SAFETY: The function is marked unsafe so that the caller guarantees
        //         that required invariants are checked.
        let slice_len = unsafe { self.buffer.get_unchecked(index..start_str) };
        let mut bytes_len = [0; LEN_USIZE];
        bytes_len.copy_from_slice(slice_len);
        let str_len = usize::from_le_bytes(bytes_len);
        let str_bytes =
            // SAFETY: The function is marked unsafe so that the caller guarantees
            //         that required invariants are checked.
            unsafe { self.buffer.get_unchecked(start_str..start_str + str_len) };
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
        let symbol = self.next_symbol();
        let str_len = string.len().to_le_bytes();
        let str_bytes = string.as_bytes();
        self.buffer.extend(str_len.iter().chain(str_bytes));
        self.len_strings += 1;
        symbol
    }
}

impl<S> Backend<S> for BufferBackend<S>
where
    S: Symbol,
{
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
    fn intern(&mut self, string: &str) -> S {
        self.push_string(string)
    }

    #[inline]
    fn resolve(&self, symbol: S) -> Option<&str> {
        self.resolve_index_to_str(symbol.to_usize())
            .map(|(string, _next_str_index)| string)
    }

    fn shrink_to_fit(&mut self) {
        self.buffer.shrink_to_fit();
    }

    #[inline]
    unsafe fn resolve_unchecked(&self, symbol: S) -> &str {
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
