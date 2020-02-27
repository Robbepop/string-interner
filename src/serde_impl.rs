use super::*;

use std::default::Default;

use std::fmt;

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

impl<Sym, H> Serialize for StringInterner<Sym, H>
where
    Sym: Symbol,
    H: BuildHasher,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for s in self.iter_values() {
            seq.serialize_element(s)?
        }
        seq.end()
    }
}

impl<'de, Sym, H> Deserialize<'de> for StringInterner<Sym, H>
where
    Sym: Symbol,
    H: BuildHasher + Default,
{
    fn deserialize<D>(deserializer: D) -> Result<StringInterner<Sym, H>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(StringInternerVisitor::default())
    }
}

struct StringInternerVisitor<Sym, H>
where
    Sym: Symbol,
    H: BuildHasher,
{
    mark: marker::PhantomData<(Sym, H)>,
}

impl<Sym, H> Default for StringInternerVisitor<Sym, H>
where
    Sym: Symbol,
    H: BuildHasher,
{
    fn default() -> Self {
        StringInternerVisitor {
            mark: marker::PhantomData,
        }
    }
}

impl<'de, Sym, H> Visitor<'de> for StringInternerVisitor<Sym, H>
where
    Sym: Symbol,
    H: BuildHasher + Default,
{
    type Value = StringInterner<Sym, H>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Expected a contiguous sequence of strings.")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut interner: StringInterner<Sym, H> =
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
