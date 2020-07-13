use super::Backend;
use crate::{
    symbol::expect_valid_symbol,
    InternedStr,
    Symbol,
};
use core::{
    marker::PhantomData,
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
    }
}
