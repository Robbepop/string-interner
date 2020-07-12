//! Interfaces and types to be used as symbols for the
//! [`StringInterner`](`crate::StringInterner`).
//!
//! The [`StringInterner::get_or_intern`](`crate::StringInterner::get_or_intern`)
//! method returns `Symbol` types that allow to look-up the original string
//! using [`StringInterner::resolve`](`crate::StringInterner::resolve`).

use core::num::{
    NonZeroU16,
    NonZeroU32,
    NonZeroUsize,
};

/// Types implementing this trait can be used as symbols for string interners.
///
/// The [`StringInterner::get_or_intern`](`crate::StringInterner::get_or_intern`)
/// method returns `Symbol` types that allow to look-up the original string
/// using [`StringInterner::resolve`](`crate::StringInterner::resolve`).
///
/// # Note
///
/// Optimal symbols allow for efficient comparisons and have a small memory footprint.
pub trait Symbol: Copy + Eq {
    /// Creates a symbol from a `usize`.
    ///
    /// Returns `None` if `index` is out of bounds for the symbol.
    fn try_from_usize(index: usize) -> Option<Self>;

    /// Returns the `usize` representation of `self`.
    fn to_usize(self) -> usize;
}

/// The symbol type that is used by default.
pub type DefaultSymbol = SymbolUsize;

macro_rules! gen_symbol_for {
    ( $name:ident, $non_zero:ty, $base_ty:ty ) => {
        /// Symbol that is the same size as a pointer (`usize`).
        ///
        /// Is space-optimized for used in `Option`.
        #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name {
            value: $non_zero,
        }

        impl Symbol for $name {
            #[inline]
            fn try_from_usize(index: usize) -> Option<Self> {
                if index < usize::MAX {
                    return Some(Self {
                        value: unsafe { <$non_zero>::new_unchecked(index as $base_ty + 1) },
                    })
                }
                None
            }

            #[inline]
            fn to_usize(self) -> usize {
                self.value.get() as usize - 1
            }
        }
    };
}
gen_symbol_for!(SymbolU16, NonZeroU16, u16);
gen_symbol_for!(SymbolU32, NonZeroU32, u32);
gen_symbol_for!(SymbolUsize, NonZeroUsize, usize);
