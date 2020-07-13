use crate::{
    backend::Backend,
    StringInterner2 as StringInterner,
    Symbol,
};
use core::{
    default::Default,
    fmt,
    hash::BuildHasher,
    marker,
};
use serde::{
    de::{
        Deserialize,
        Deserializer,
        SeqAccess,
        Visitor,
    },
    ser::{
        Serialize,
        SerializeSeq,
        Serializer,
    },
};

impl<S, B, H> Serialize for StringInterner<S, B, H>
where
    S: Symbol,
    B: Backend<S>,
    for<'a> &'a B: IntoIterator<Item = (S, &'a str)>,
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

impl<'de, S, B, H> Deserialize<'de> for StringInterner<S, B, H>
where
    S: Symbol,
    B: Backend<S>,
    H: BuildHasher + Default,
{
    fn deserialize<D>(deserializer: D) -> Result<StringInterner<S, B, H>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(StringInternerVisitor::default())
    }
}

struct StringInternerVisitor<S, B, H>
where
    S: Symbol,
    B: Backend<S>,
    H: BuildHasher,
{
    mark: marker::PhantomData<(S, B, H)>,
}

impl<S, B, H> Default for StringInternerVisitor<S, B, H>
where
    S: Symbol,
    B: Backend<S>,
    H: BuildHasher,
{
    fn default() -> Self {
        StringInternerVisitor {
            mark: marker::PhantomData,
        }
    }
}

impl<'de, S, B, H> Visitor<'de> for StringInternerVisitor<S, B, H>
where
    S: Symbol,
    B: Backend<S>,
    H: BuildHasher + Default,
{
    type Value = StringInterner<S, B, H>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Expected a contiguous sequence of strings.")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut interner: StringInterner<S, B, H> =
            StringInterner::with_capacity_and_hasher(
                seq.size_hint().unwrap_or(0),
                H::default(),
            );
        while let Some(s) = seq.next_element::<Box<str>>()? {
            interner.get_or_intern(s);
        }
        Ok(interner)
    }
}
