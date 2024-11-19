use crate::{backend::Backend, StringInterner, Symbol};
use alloc::boxed::Box;
use core::{default::Default, fmt, hash::BuildHasher, marker};
use serde::{
    de::{Deserialize, Deserializer, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};

impl<'i, B, H> Serialize for StringInterner<'i, B, H>
where
    B: Backend<'i>,
    <B as Backend<'i>>::Symbol: Symbol,
    for<'l> &'l B: IntoIterator<Item = (<B as Backend<'i>>::Symbol, &'l str)>,
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

impl<'i: 'de, 'de, B, H> Deserialize<'de> for StringInterner<'i, B, H>
where
    B: Backend<'i>,
    <B as Backend<'i>>::Symbol: Symbol,
    H: BuildHasher + Default,
{
    fn deserialize<D>(deserializer: D) -> Result<StringInterner<'i, B, H>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(StringInternerVisitor::default())
    }
}

struct StringInternerVisitor<'i, B, H>
where
    B: Backend<'i>,
    <B as Backend<'i>>::Symbol: Symbol,
    H: BuildHasher,
{
    mark: marker::PhantomData<(<B as Backend<'i>>::Symbol, B, H)>,
}

impl<'i, B, H> Default for StringInternerVisitor<'i, B, H>
where
    B: Backend<'i>,
    <B as Backend<'i>>::Symbol: Symbol,
    H: BuildHasher,
{
    fn default() -> Self {
        StringInternerVisitor {
            mark: marker::PhantomData,
        }
    }
}

impl<'i: 'de, 'de, B, H> Visitor<'de> for StringInternerVisitor<'i, B, H>
where
    B: Backend<'i>,
    <B as Backend<'i>>::Symbol: Symbol,
    H: BuildHasher + Default,
{
    type Value = StringInterner<'i, B, H>;

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
