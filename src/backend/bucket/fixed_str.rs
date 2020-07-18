#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedString {
    contents: Box<str>,
    len: usize,
}

impl Default for FixedString {
    fn default() -> Self {
        Self {
            contents: String::new().into_boxed_str(),
            len: 0,
        }
    }
}

impl FixedString {
    /// Creates a new fixed string with the given fixed capacity.
    pub fn with_capacity(cap: usize) -> Self {
        let contents = String::from_utf8(vec![0x00; cap])
            .expect("encountered invalid utf8 init sequence")
            .into_boxed_str();
        debug_assert_eq!(contents.len(), cap);
        Self { contents, len: 0 }
    }

    /// Returns the underlying [`Box<str>`].
    ///
    /// Guarantees not to perform any reallocations in this process.
    pub fn into_boxed_str(self) -> Box<str> {
        self.contents
    }

    /// Returns the capacity in bytes of the fixed string.
    pub fn capacity(&self) -> usize {
        self.contents.len()
    }

    /// Returns the length in bytes of the fixed string.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Pushes the given string into the fixed string if there is enough capacity.
    ///
    /// Returns a reference to the pushed string if there was enough capacity to
    /// perform the operation. Otherwise returns `None`.
    pub fn push_str<'a>(&'a mut self, str: &str) -> Option<&'a str> {
        let len = self.len();
        if self.capacity() < len + str.len() {
            return None
        }
        // SAFETY: This operation is safe since we checked beforehand if the
        //         capacity of the fixed string is large enough to fix the
        //         contents of the newly pushed string.
        //         Also the newly pushed string is of type `str` which means
        //         that it respects unicode points properly.
        unsafe {
            self.contents.as_bytes_mut()[len..len + str.len()]
                .copy_from_slice(str.as_bytes())
        };
        self.len += str.len();
        Some(unsafe {
            core::str::from_utf8_unchecked(
                &self.contents.as_bytes()[len..len + str.len()],
            )
        })
    }
}
