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

macro_rules! impl_serde_for_symbol {
    ($name:ident, $ty:ty) => {
        impl ::serde::Serialize for $crate::symbol::$name {
            fn serialize<T: ::serde::Serializer>(
                &self,
                serializer: T,
            ) -> ::core::result::Result<T::Ok, T::Error> {
                self.to_usize().serialize(serializer)
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $crate::symbol::$name {
            fn deserialize<D: ::serde::Deserializer<'de>>(
                deserializer: D,
            ) -> ::core::result::Result<Self, D::Error> {
                let index = <$ty as ::serde::Deserialize<'de>>::deserialize(deserializer)?;
                let ::core::option::Option::Some(symbol) = Self::new(index) else {
                    return ::core::result::Result::Err(<D::Error as ::serde::de::Error>::custom(
                        ::core::concat!(
                            "invalid index value for `",
                            ::core::stringify!($name),
                            "`"
                        ),
                    ));
                };
                ::core::result::Result::Ok(symbol)
            }
        }
    };
}
impl_serde_for_symbol!(SymbolU16, u16);
impl_serde_for_symbol!(SymbolU32, u32);
impl_serde_for_symbol!(SymbolUsize, usize);

#[cfg(test)]
mod tests {
    use crate::{
        symbol::{SymbolU16, SymbolU32, SymbolUsize},
        Symbol,
    };
    use serde_json;

    fn symbol_round_trip_serializes<S>(symbol: S) -> bool
    where
        S: Symbol + std::fmt::Debug + serde::Serialize + serde::de::DeserializeOwned + PartialEq,
    {
        let serialized = serde_json::to_string(&symbol).expect("serialization should succeed");
        let deserialized: S =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        symbol == deserialized
    }

    #[test]
    fn symbol_u16_round_trips() {
        assert!(symbol_round_trip_serializes(
            SymbolU16::try_from_usize(0).unwrap()
        ));
        assert!(symbol_round_trip_serializes(
            SymbolU16::try_from_usize(42).unwrap()
        ));
        assert!(symbol_round_trip_serializes(
            SymbolU16::try_from_usize(u16::MAX as usize - 1).unwrap()
        ));
    }

    #[test]
    fn symbol_u32_round_trips() {
        assert!(symbol_round_trip_serializes(
            SymbolU32::try_from_usize(0).unwrap()
        ));
        assert!(symbol_round_trip_serializes(
            SymbolU32::try_from_usize(42).unwrap()
        ));
        assert!(symbol_round_trip_serializes(
            SymbolU32::try_from_usize(u32::MAX as usize - 1).unwrap()
        ));
    }

    #[test]
    fn symbol_usize_round_trips() {
        assert!(symbol_round_trip_serializes(
            SymbolUsize::try_from_usize(0).unwrap()
        ));
        assert!(symbol_round_trip_serializes(
            SymbolUsize::try_from_usize(42).unwrap()
        ));
        assert!(symbol_round_trip_serializes(
            SymbolUsize::try_from_usize(usize::MAX as usize - 1).unwrap()
        ));
    }

    #[test]
    fn raw_usize_round_trips() {
        assert!(symbol_round_trip_serializes(
            usize::try_from_usize(0).unwrap()
        ));
        assert!(symbol_round_trip_serializes(
            usize::try_from_usize(42).unwrap()
        ));
        assert!(symbol_round_trip_serializes(
            usize::try_from_usize(usize::MAX).unwrap()
        ));
    }
}
