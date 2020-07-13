use super::Backend;
use crate::{
    symbol::expect_valid_symbol,
    InternedStr,
    Symbol,
};
use core::{
    iter::Enumerate,
    marker::PhantomData,
    pin::Pin,
    slice,
};

/// TODO: Docs
#[derive(Debug)]
pub struct SimpleBackend<S> {
    strings: Vec<Pin<Box<str>>>,
    symbol_marker: PhantomData<fn() -> S>,
}

impl<S> Default for SimpleBackend<S> {
    #[inline]
    fn default() -> Self {
        Self {
            strings: Vec::new(),
            symbol_marker: Default::default(),
        }
    }
}

impl<S> Backend<S> for SimpleBackend<S>
where
    S: Symbol,
{
    #[inline]
    fn with_capacity(cap: usize) -> Self {
        Self {
            strings: Vec::with_capacity(cap),
            symbol_marker: Default::default(),
        }
    }

    #[inline]
    unsafe fn intern(&mut self, string: &str) -> (InternedStr, S) {
        let symbol = expect_valid_symbol(self.strings.len());
        let str = Pin::new(string.to_string().into_boxed_str());
        let interned = InternedStr::new(&*str);
        self.strings.push(str);
        (interned, symbol)
    }

    #[inline]
    fn resolve(&self, symbol: S) -> Option<&str> {
        self.strings.get(symbol.to_usize()).map(|pinned| &**pinned)
    }
}

impl<'a, S> IntoIterator for &'a SimpleBackend<S>
where
    S: Symbol,
{
    type Item = (S, &'a str);
    type IntoIter = Iter<'a, S>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

pub struct Iter<'a, S> {
    iter: Enumerate<slice::Iter<'a, Pin<Box<str>>>>,
    symbol_marker: PhantomData<fn() -> S>,
}

impl<'a, S> Iter<'a, S> {
    #[inline]
    pub fn new(backend: &'a SimpleBackend<S>) -> Self {
        Self {
            iter: backend.strings.iter().enumerate(),
            symbol_marker: Default::default(),
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
        self.iter.size_hint()
    }

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(id, pinned)| (expect_valid_symbol(id), &**pinned))
    }
}
