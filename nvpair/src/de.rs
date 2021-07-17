use crate::*;
use ::serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use ::serde::{de, forward_to_deserialize_any, Deserialize};
use serde::de::DeserializeOwned;
use std::ffi::CStr;
use std::marker::PhantomData;

pub fn from_bytes<T>(buf: &[u8]) -> Result<T>
where
    T: DeserializeOwned,
{
    from_nvlist(&NvList::try_unpack(buf)?)
}

pub fn from_nvlist<'a, T>(s: &'a NvList) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_nvlist(s);
    let t = T::deserialize(&mut deserializer)?;
    /*
    match deserializer.iterator.next() {
        Some(pair) => Err(Error::Message(format!(
            "unconsumed nvpairs, including {:?}",
            pair
        ))),
        None => Ok(t),
    }
    */
    Ok(t)
}

#[derive(Debug)]
struct Deserializer<'de> {
    iterator: NvListIter<'de>,
    data: Option<NvData<'de>>,
}

impl<'de> Deserializer<'de> {
    fn from_nvlist(input: &'de NvListRef) -> Self {
        Deserializer {
            iterator: input.iter(),
            data: None,
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> MapAccess<'de> for Deserializer<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        let nvpair = match self.iterator.next() {
            Some(nvpair) => nvpair,
            None => return Ok(None),
        };

        let mut key_deserializer = KeyDeserializer { key: nvpair.name() };
        self.data = Some(nvpair.data());

        seed.deserialize(&mut key_deserializer).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        match self.data.take() {
            Some(data) => {
                let mut value_deserializer = ValueDeserializer { data };
                seed.deserialize(&mut value_deserializer)
            }
            None => Err(Error::Message("expected value but not present".to_string())),
        }
    }
}

#[derive(Debug)]
struct KeyDeserializer<'de> {
    key: &'de CStr,
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut KeyDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.key.to_str()?)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

#[derive(Debug)]
struct ValueDeserializer<'de> {
    data: NvData<'de>,
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut ValueDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.data {
            NvData::Unknown => Err(Error::UnknownNvPairType),
            NvData::Bool => visitor.visit_none(),
            NvData::BoolV(v) => visitor.visit_bool(v),
            NvData::Byte(v) => visitor.visit_u8(v),
            NvData::Int8(v) => visitor.visit_i8(v),
            NvData::Uint8(v) => visitor.visit_u8(v),
            NvData::Int16(v) => visitor.visit_i16(v),
            NvData::Uint16(v) => visitor.visit_u16(v),
            NvData::Int32(v) => visitor.visit_i32(v),
            NvData::Uint32(v) => visitor.visit_u32(v),
            NvData::Int64(v) => visitor.visit_i64(v),
            NvData::Uint64(v) => visitor.visit_u64(v),
            NvData::Str(v) => visitor.visit_borrowed_str(v.to_str()?),
            NvData::NvListRef(v) => visitor.visit_map(Deserializer::from_nvlist(v)),
            NvData::ByteArray(v) => visitor.visit_borrowed_bytes(v),
            NvData::Int8Array(v) => visitor.visit_seq(SeqAccessor::new(v.iter())),
            NvData::Uint8Array(v) => visitor.visit_borrowed_bytes(v),
            NvData::Int16Array(v) => visitor.visit_seq(SeqAccessor::new(v.iter())),
            NvData::Uint16Array(v) => visitor.visit_seq(SeqAccessor::new(v.iter())),
            NvData::Int32Array(v) => visitor.visit_seq(SeqAccessor::new(v.iter())),
            NvData::Uint32Array(v) => visitor.visit_seq(SeqAccessor::new(v.iter())),
            NvData::Int64Array(v) => visitor.visit_seq(SeqAccessor::new(v.iter())),
            NvData::Uint64Array(v) => visitor.visit_seq(SeqAccessor::new(v.iter())),
            NvData::NvListRefArray(_) => Err(Error::UnknownNvPairType),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

#[derive(Debug)]
struct SeqAccessor<'de, I>
where
    I: Iterator,
    <I as Iterator>::Item: Into<NvData<'de>>,
{
    iterator: I,
    _phantom: std::marker::PhantomData<&'de I>,
}

impl<'de, I> SeqAccessor<'de, I>
where
    I: Iterator,
    <I as Iterator>::Item: Into<NvData<'de>>,
{
    fn new(iterator: I) -> Self {
        Self {
            iterator,
            _phantom: PhantomData,
        }
    }
}

impl<'de, I> SeqAccess<'de> for SeqAccessor<'de, I>
where
    I: Iterator,
    <I as Iterator>::Item: Into<NvData<'de>>,
{
    type Error = Error;

    fn next_element_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        let item = match self.iterator.next() {
            Some(item) => item,
            None => return Ok(None),
        };

        let mut vd = ValueDeserializer { data: item.into() };
        seed.deserialize(&mut vd).map(Some)
    }
}

impl From<&u64> for NvData<'_> {
    fn from(v: &u64) -> Self {
        NvData::Uint64(*v)
    }
}

impl From<&i64> for NvData<'_> {
    fn from(v: &i64) -> Self {
        NvData::Int64(*v)
    }
}

impl From<&u32> for NvData<'_> {
    fn from(v: &u32) -> Self {
        NvData::Uint32(*v)
    }
}

impl From<&i32> for NvData<'_> {
    fn from(v: &i32) -> Self {
        NvData::Int32(*v)
    }
}

impl From<&u16> for NvData<'_> {
    fn from(v: &u16) -> Self {
        NvData::Uint16(*v)
    }
}

impl From<&i16> for NvData<'_> {
    fn from(v: &i16) -> Self {
        NvData::Int16(*v)
    }
}

impl From<&u8> for NvData<'_> {
    fn from(v: &u8) -> Self {
        NvData::Uint8(*v)
    }
}

impl From<&i8> for NvData<'_> {
    fn from(v: &i8) -> Self {
        NvData::Int8(*v)
    }
}

/*
impl<'a> From<&'a NvListRef> for NvData<'a> {
    fn from(v: &'a NvListRef) -> Self {
        NvData::NvListRef(v)
    }
}

impl<'a> From<&&'a NvListRef> for NvData<'a> {
    fn from(v: &&'a NvListRef) -> Self {
        NvData::NvListRef(*v)
    }
}
*/
