use super::InternedStr;
use crate::Result;
#[cfg(not(feature = "std"))]
use alloc::string::String;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FixedString {
    contents: String,
}

impl FixedString {
    /// Creates a new fixed string with the given fixed capacity.
    #[inline]
    pub fn try_with_capacity(cap: usize) -> Result<Self> {
        let mut contents = String::new();
        contents.try_reserve(cap)?;
        Ok(Self { contents })
        // FIXME: try_with_capacity #91913, replace with the following:
        // Ok(Self {
        //     contents: String::try_with_capacity(cap)?,
        // })
    }
    /// Returns the underlying [`String`].
    ///
    /// Guarantees not to perform any reallocations in this process.
    #[inline]
    pub fn finish(self) -> String {
        self.contents
    }

    /// Returns the capacity in bytes of the fixed string.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.contents.capacity()
    }

    /// Returns the length in bytes of the fixed string.
    #[inline]
    pub fn len(&self) -> usize {
        self.contents.len()
    }

    /// Pushes the given string into the fixed string if there is enough capacity.
    ///
    /// Returns a reference to the pushed string if there was enough capacity to
    /// perform the operation. Otherwise returns `None`.
    #[inline]
    pub fn push_str(&mut self, string: &str) -> Option<InternedStr> {
        let len = self.len();
        let new_len = len + string.len();
        if self.capacity() < new_len {
            return None;
        }
        self.contents.push_str(string);
        debug_assert_eq!(self.contents.len(), new_len);
        Some(InternedStr::new(
            // SAFETY: We convert from bytes to utf8 from which we know through the
            //         input string that they must represent valid utf8.
            unsafe { core::str::from_utf8_unchecked(&self.contents.as_bytes()[len..new_len]) },
        ))
    }
}
