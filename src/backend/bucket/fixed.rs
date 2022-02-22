use std::{
    ops::Deref,
    ptr::NonNull,
};

use len_trait::{
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
pub unsafe trait FixedContainer<S: ?Sized>:
    Deref<Target = S> + WithCapacity + Len
{
    /// Push the given string into the fixed string if there is enough capacity.
    ///
    /// Returns a reference to the pushed string if there was enough capacity to
    /// perform the operation. Otherwise returns `None`.
    fn try_push_str(&mut self, string: &S) -> Option<NonNull<S>>;
}

unsafe impl FixedContainer<str> for String {
    #[inline]
    fn try_push_str(&mut self, string: &str) -> Option<NonNull<str>> {
        let len = self.len();
        if self.capacity() < len + string.len() {
            return None
        }
        self.push_str(string);
        debug_assert_eq!(self.len(), len + string.len());
        Some(NonNull::from(
            // SAFETY: We convert from bytes to utf8 from which we know through the
            //         input string that they must represent valid utf8.
            unsafe {
                core::str::from_utf8_unchecked(&self.as_bytes()[len..len + string.len()])
            },
        ))
    }
}

unsafe impl<T> FixedContainer<[T]> for Vec<T>
where
    T: Clone,
{
    #[inline]
    fn try_push_str(&mut self, string: &[T]) -> Option<NonNull<[T]>> {
        let len = self.len();
        if self.capacity() < len + string.len() {
            return None
        }
        self.extend_from_slice(string);
        debug_assert_eq!(self.len(), len + string.len());
        Some(NonNull::from(&self[len..len + string.len()]))
    }
}
