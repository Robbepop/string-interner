use crate::InternedStr;
use core::{
    borrow::Borrow,
    fmt::{
        Debug,
        Formatter,
        Result,
    },
    hash::{
        Hash,
        Hasher,
    },
};

/// Crate private new-type wrapper around the public [`InternedStr`].
///
/// # Note
///
/// This allows to have some utility trait implementations since we know that
/// we will safely handle it. It will never be part of the public API.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct InternalStr(InternedStr);

impl Debug for InternalStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "InternalStr({:?})", unsafe { self.0.as_str() })
    }
}

impl From<InternedStr> for InternalStr {
    #[inline]
    fn from(str: InternedStr) -> Self {
        Self(str)
    }
}

impl Hash for InternalStr {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe { self.0.as_str().hash(state) }
    }
}

impl Eq for InternalStr {}

impl PartialEq for InternalStr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.0.as_str() == other.0.as_str() }
    }
}

impl Borrow<str> for InternalStr {
    #[inline]
    fn borrow(&self) -> &str {
        unsafe { self.0.as_str() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_of() {
        use std::mem;
        assert_eq!(mem::size_of::<InternedStr>(), mem::size_of::<&str>());
    }

    #[test]
    fn eq() {
        // same origin (aka pointer to str)
        let s = "bar";
        assert_eq!(
            InternalStr::from(InternedStr::new(s)),
            InternalStr::from(InternedStr::new(s))
        );
        // different origins (aka pointers)
        assert_eq!(
            InternalStr::from(InternedStr::new("foo")),
            InternalStr::from(InternedStr::new("foo"))
        );
    }

    #[test]
    fn ne() {
        assert_ne!(
            InternalStr::from(InternedStr::new("foo")),
            InternalStr::from(InternedStr::new("bar"))
        )
    }

    #[test]
    fn hash_same_as_str() {
        use std::{
            collections::hash_map::DefaultHasher,
            hash::Hash,
        };
        let (s0, s1) = ("foo", "bar");
        let (r0, r1) = (
            InternalStr::from(InternedStr::new(s0)),
            InternalStr::from(InternedStr::new(s1)),
        );
        let mut sip = DefaultHasher::new();
        assert_eq!(r0.hash(&mut sip), s0.hash(&mut sip));
        assert_eq!(r1.hash(&mut sip), s1.hash(&mut sip));
    }
}
