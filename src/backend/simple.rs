use super::Backend;
use crate::{
    InternedStr,
    Symbol,
};

/// TODO: Docs
#[derive(Debug, Default)]
pub struct SimpleBackend;

impl<S> Backend<S> for SimpleBackend
where
    S: Symbol,
{
    fn with_capacity(cap: usize) -> Self {
        todo!()
    }

    unsafe fn intern(&mut self, string: &str) -> (InternedStr, S) {
        todo!()
    }

    fn resolve(&self, symbol: S) -> Option<&str> {
        todo!()
    }
}
