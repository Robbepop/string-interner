use core::{
    hash::{
        Hash,
        Hasher,
    },
    pin::Pin,
    ptr::NonNull,
};

/// Internal reference to an interned `str`.
///
/// This is a self-referential from the interners string map
/// into the interner's actual vector of strings.
#[derive(Debug, Copy, Clone, Eq)]
pub struct PinnedStr(NonNull<str>);

impl PinnedStr {
    /// Creates a new `PinnedStr` from the given `str`.
    pub fn from_str(val: &str) -> Self {
        PinnedStr(NonNull::from(val))
    }

    /// Creates a new `PinnedStr` from the given pinned `str`.
    pub fn from_pin(pinned: Pin<&str>) -> Self {
        PinnedStr(NonNull::from(&*pinned))
    }

    /// Returns a shared reference to the underlying `str`.
    pub fn as_str(&self) -> &str {
        // SAFETY: This is safe since we only ever operate on interned `str`
        //         that are never moved around in memory to avoid danling
        //         references.
        unsafe { self.0.as_ref() }
    }
}

impl Hash for PinnedStr {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl PartialEq for PinnedStr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl core::borrow::Borrow<str> for PinnedStr {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_of() {
        use std::mem;
        assert_eq!(mem::size_of::<PinnedStr>(), mem::size_of::<&str>());
    }

    #[test]
    fn eq() {
        // same origin (aka pointer to str)
        let s = "bar";
        assert_eq!(PinnedStr::from_str(s), PinnedStr::from_str(s));
        // different origins (aka pointers)
        assert_eq!(PinnedStr::from_str("foo"), PinnedStr::from_str("foo"));
    }

    #[test]
    fn ne() {
        assert_ne!(PinnedStr::from_str("foo"), PinnedStr::from_str("bar"))
    }

    #[test]
    fn hash_same_as_str() {
        use std::{
            collections::hash_map::DefaultHasher,
            hash::Hash,
        };
        let (s0, s1) = ("foo", "bar");
        let (r0, r1) = (PinnedStr::from_str(s0), PinnedStr::from_str(s1));
        let mut sip = DefaultHasher::new();
        assert_eq!(r0.hash(&mut sip), s0.hash(&mut sip));
        assert_eq!(r1.hash(&mut sip), s1.hash(&mut sip));
    }
}
