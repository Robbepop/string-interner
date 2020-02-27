use core::num::NonZeroU32;

/// Types implementing this trait are able to act as symbols for string interners.
///
/// Symbols are returned by `StringInterner::get_or_intern` and allow look-ups of the
/// original string contents with `StringInterner::resolve`.
///
/// # Note
///
/// Optimal symbols allow for efficient comparisons and have a small memory footprint.
pub trait Symbol: Copy + Ord + Eq {
    /// Creates a symbol from a `usize`.
    ///
    /// # Note
    ///
    /// Implementations panic if the operation cannot succeed.
    fn from_usize(val: usize) -> Self;

    /// Returns the `usize` representation of `self`.
    fn to_usize(self) -> usize;
}

/// Fast and space efficient symbol type.
///
/// # Note
///
/// - Has a memory footprint of 32 bit.
/// - Allow for space optimizations, i.e.
///   `size_of<Option<DefaultSymbol>() == size_of<DefaultSymbol>()`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DefaultSymbol(NonZeroU32);

impl Symbol for DefaultSymbol {
    /// Creates a `Sym` from the given `usize`.
    ///
    /// # Panics
    ///
    /// If the given `usize` is greater than `u32::MAX - 1`.
    fn from_usize(val: usize) -> Self {
        assert!(
            val < core::u32::MAX as usize,
            "{} is out of bounds for the default symbol",
            val
        );
        Self(
            NonZeroU32::new((val + 1) as u32)
                // Due to the assert above we can assume that this always succeeds.
                .unwrap_or_else(|| unsafe { core::hint::unreachable_unchecked() }),
        )
    }

    fn to_usize(self) -> usize {
        (self.0.get() as usize) - 1
    }
}
