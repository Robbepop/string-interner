use crate::{backend::Backend, StringInterner, Symbol};
use alloc::boxed::Box;
use core::{default::Default, fmt, hash::BuildHasher, marker};
use serde::{
    de::{Deserialize, Deserializer, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};

impl<B, H> Serialize for StringInterner<B, H>
where
    B: Backend,
    <B as Backend>::Symbol: Symbol,
    for<'a> &'a B: IntoIterator<Item = (<B as Backend>::Symbol, &'a str)>,
    H: BuildHasher,
{
    fn serialize<T>(&self, serializer: T) -> Result<T::Ok, T::Error>
    where
        T: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for (_symbol, string) in self {
            seq.serialize_element(string)?
        }
        seq.end()
    }
}

impl<'de, B, H> Deserialize<'de> for StringInterner<B, H>
where
    B: Backend,
    <B as Backend>::Symbol: Symbol,
    H: BuildHasher + Default,
{
    fn deserialize<D>(deserializer: D) -> Result<StringInterner<B, H>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(StringInternerVisitor::default())
    }
}

struct StringInternerVisitor<B, H>
where
    B: Backend,
    <B as Backend>::Symbol: Symbol,
    H: BuildHasher,
{
    mark: marker::PhantomData<(<B as Backend>::Symbol, B, H)>,
}

impl<B, H> Default for StringInternerVisitor<B, H>
where
    B: Backend,
    <B as Backend>::Symbol: Symbol,
    H: BuildHasher,
{
    fn default() -> Self {
        StringInternerVisitor {
            mark: marker::PhantomData,
        }
    }
}

impl<'de, B, H> Visitor<'de> for StringInternerVisitor<B, H>
where
    B: Backend,
    <B as Backend>::Symbol: Symbol,
    H: BuildHasher + Default,
{
    type Value = StringInterner<B, H>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Expected a contiguous sequence of strings.")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut interner: StringInterner<B, H> =
            StringInterner::with_capacity_and_hasher(seq.size_hint().unwrap_or(0), H::default());
        while let Some(s) = seq.next_element::<Box<str>>()? {
            interner.get_or_intern(s);
        }
        Ok(interner)
    }
}
