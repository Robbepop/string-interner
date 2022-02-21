//! Backends for the [`StringInterner`](`crate::StringInterner`).
//!
//! The backend is the method or strategy that handles the actual interning.
//! There are trade-offs for the different kinds of backends. A user should
//! find the backend that suits their use case best.

pub mod bucket;
pub mod buffer;
pub mod simple;
pub mod string;

use len_trait::{
    CapacityMut,
    Len,
    WithCapacity,
};

#[cfg(feature = "backends")]
pub use self::{
    bucket::BucketBackend,
    buffer::BufferBackend,
    simple::SimpleBackend,
    string::StringBackend,
};
use crate::Symbol;

#[cfg(not(feature = "backends"))]
/// Indicates that no proper backend is in use.
pub struct NoBackend;

cfg_if::cfg_if! {
    if #[cfg(feature = "backends")] {
        /// The default backend recommended for general use.
        pub type DefaultBackend = StringBackend;
    } else {
        /// The `backends` crate feature is disabled thus there is no default backend.
        pub type DefaultBackend = NoBackend;
    }
}

/// Types implementing this trait may act as backends for the string interner.
///
/// The job of a backend is to actually store, manage and organize the interned
/// strings. Different backends have different trade-offs. Users should pick
/// their backend with hinsight of their personal use-case.
pub trait Backend: Default {
    /// The type of the interned strings
    type Str: Internable + ?Sized;
    /// The symbol used by the string interner backend.
    type Symbol: Symbol;

    /// Creates a new backend for the given capacity.
    ///
    /// The capacity denotes how many strings are expected to be interned.
    fn with_capacity(cap: usize) -> Self;

    /// Interns the given string and returns its interned ref and symbol.
    ///
    /// # Note
    ///
    /// The backend must make sure that the returned symbol maps back to the
    /// original string in its [`resolve`](`Backend::resolve`) method.
    fn intern(&mut self, string: &Self::Str) -> Self::Symbol;

    /// Interns the given static string and returns its interned ref and symbol.
    ///
    /// # Note
    ///
    /// The backend must make sure that the returned symbol maps back to the
    /// original string in its [`resolve`](`Backend::resolve`) method.
    #[inline]
    fn intern_static(&mut self, string: &'static Self::Str) -> Self::Symbol {
        // The default implementation simply forwards to the normal [`intern`]
        // implementation. Backends that can optimize for this use case should
        // implement this method.
        self.intern(string)
    }

    /// Shrink backend capacity to fit interned symbols exactly.
    fn shrink_to_fit(&mut self);

    /// Resolves the given symbol to its original string contents.
    fn resolve(&self, symbol: Self::Symbol) -> Option<&Self::Str>;

    /// Resolves the given symbol to its original string contents.
    ///
    /// # Safety
    ///
    /// Does not perform validity checks on the given symbol and relies
    /// on the caller to be provided with a symbol that has been generated
    /// by the [`intern`](`Backend::intern`) or
    /// [`intern_static`](`Backend::intern_static`) methods of the same
    /// interner backend.
    unsafe fn resolve_unchecked(&self, symbol: Self::Symbol) -> &Self::Str;
}

/// Represents a type that is internable within all backends.
/// This type must be supported by a slice `&[Self::Element]`.
///
/// This trait is the only bound needed for a type to be compatible
/// with almost all backends implemented in this crate, with the exception
/// of [`BucketBackend`](crate::backend::BucketBackend), which also requires a
/// [`FixedContainer`](crate::backend::bucket::FixedContainer) implementation
/// for `<Self as ToOwned>::Owned`
pub trait Internable: Len {
    /// The container type used as a buffer for storing `Self`
    type Container: Into<Box<Self>> + AsRef<Self> + CapacityMut;
    /// The element type of the slice view
    type Element: Copy;
    /// Convert from a slice `&[U]` to `Self`
    fn from_slice(input: &[Self::Element]) -> &Self;
    /// Convert `Self` to a slice `&[U]`
    fn to_slice(&self) -> &[Self::Element];
    /// Push the contents of Self into a `Self::Container`.
    fn push_str(buffer: &mut Self::Container, str: &Self);
    /// Create a new `Box<Self>` from the data of `Self`.
    fn to_boxed(&self) -> Box<Self> {
        let mut c = <Self::Container>::with_capacity(self.len());
        Self::push_str(&mut c, self);
        c.into()
    }
}

impl Internable for str {
    type Container = String;
    type Element = u8;
    fn from_slice(input: &[Self::Element]) -> &Self {
        // SAFETY: Internally the backends only manipulate `&[u8]` slices
        //         which are valid utf-8.
        unsafe { std::str::from_utf8_unchecked(input) }
    }

    fn to_slice(&self) -> &[Self::Element] {
        self.as_bytes()
    }

    fn push_str(buffer: &mut <Self as ToOwned>::Owned, str: &Self) {
        buffer.push_str(str)
    }
}

impl<T> Internable for [T]
where
    T: Copy,
{
    type Container = Vec<T>;
    type Element = T;
    fn from_slice(input: &[Self::Element]) -> &Self {
        input
    }

    fn to_slice(&self) -> &[Self::Element] {
        self
    }

    fn push_str(buffer: &mut <Self as ToOwned>::Owned, str: &Self) {
        buffer.extend_from_slice(str)
    }
}
