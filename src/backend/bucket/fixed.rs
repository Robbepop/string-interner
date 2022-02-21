use std::{
    ops::Deref,
    ptr::NonNull,
};

use len_trait::{
    Capacity,
    Empty,
    Len,
    WithCapacity,
};

/// Represents a container with a fixed initial capacity that
/// is capable of pushing elements of type `&S` into its internal buffer only if the
/// elements don't exceed its fixed capacity.
///
/// # Safety
///
/// It is Undefined Behaviour if any mutable or internally mutable operation
/// invalidates previously generated [`NonNull<S>`] pointers.
///
/// In other words, implementations must guarantee that no reallocations
/// occur after creating the container.
pub unsafe trait FixedContainer: Deref + WithCapacity + Len {
    /// Push the given string into the fixed string if there is enough capacity.
    ///
    /// Returns a reference to the pushed string if there was enough capacity to
    /// perform the operation. Otherwise returns `None`.
    fn push_str(&mut self, string: &Self::Target) -> Option<NonNull<Self::Target>>;
}

/// A [`String`] buffer that can only be created with an specific
/// capacity, and cannot reallocate after.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FixedString {
    contents: String,
}

impl Deref for FixedString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &*self.contents
    }
}

impl Empty for FixedString {
    fn is_empty(&self) -> bool {
        self.contents.is_empty()
    }
}

impl Len for FixedString {
    fn len(&self) -> usize {
        self.contents.len()
    }
}

impl Capacity for FixedString {
    fn capacity(&self) -> usize {
        self.contents.capacity()
    }
}

impl WithCapacity for FixedString {
    fn with_capacity(capacity: usize) -> Self {
        FixedString {
            contents: String::with_capacity(capacity),
        }
    }
}

unsafe impl FixedContainer for FixedString {
    #[inline]
    fn push_str(&mut self, string: &str) -> Option<NonNull<str>> {
        let len = self.len();
        if self.capacity() < len + string.len() {
            return None
        }
        self.contents.push_str(string);
        debug_assert_eq!(self.len(), len + string.len());
        Some(NonNull::from(
            // SAFETY: We convert from bytes to utf8 from which we know through the
            //         input string that they must represent valid utf8.
            unsafe {
                core::str::from_utf8_unchecked(
                    &self.contents.as_bytes()[len..len + string.len()],
                )
            },
        ))
    }
}

/// A [`Vec`] buffer that can only be created with an specific
/// capacity, and cannot reallocate after.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedVec<T> {
    contents: Vec<T>,
}

impl<T> Default for FixedVec<T> {
    fn default() -> Self {
        Self {
            contents: Vec::default(),
        }
    }
}

impl<T> Deref for FixedVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &*self.contents
    }
}

impl<T> Empty for FixedVec<T> {
    fn is_empty(&self) -> bool {
        self.contents.is_empty()
    }
}

impl<T> Len for FixedVec<T> {
    fn len(&self) -> usize {
        self.contents.len()
    }
}

impl<T> Capacity for FixedVec<T> {
    fn capacity(&self) -> usize {
        self.contents.capacity()
    }
}

impl<T> WithCapacity for FixedVec<T> {
    fn with_capacity(capacity: usize) -> Self {
        FixedVec {
            contents: Vec::with_capacity(capacity),
        }
    }
}

unsafe impl<T> FixedContainer for FixedVec<T>
where
    T: Clone,
{
    #[inline]
    fn push_str(&mut self, string: &[T]) -> Option<NonNull<[T]>> {
        let len = self.len();
        if self.capacity() < len + string.len() {
            return None
        }
        self.contents.extend_from_slice(string);
        debug_assert_eq!(self.len(), len + string.len());
        Some(NonNull::from(&self.contents[len..len + string.len()]))
    }
}
