use crate::*;
use ::serde::{ser, Serialize};
use std::ffi;

#[derive(Debug)]
struct Serializer {}

#[derive(Debug)]
enum NvDataOwned {
    Unknown,
    None,
    Bool,
    BoolV(bool),
    Byte(u8),
    Int8(i8),
    Uint8(u8),
    Int16(i16),
    Uint16(u16),
    Int32(i32),
    Uint32(u32),
    Int64(i64),
    Uint64(u64),
    String(CString),
    NvList(NvList),
    ByteArray(Vec<u8>),
    Int8Array(Vec<i8>),
    Uint8Array(Vec<u8>),
    Int16Array(Vec<i16>),
    Uint16Array(Vec<u16>),
    Int32Array(Vec<i32>),
    Uint32Array(Vec<u32>),
    Int64Array(Vec<i64>),
    Uint64Array(Vec<u64>),
    NvListArray(Vec<NvList>),
    /* TODO:
    pub const DATA_TYPE_STRING_ARRAY: Type = 17;
    pub const DATA_TYPE_HRTIME: Type = 18;
    pub const DATA_TYPE_BOOLEAN_ARRAY: Type = 24;
    pub const DATA_TYPE_DOUBLE: Type = 27;
    */
}

impl NvEncode for NvDataOwned {
    fn insert_into<S: CStrArgument>(&self, name: S, nv: &mut NvListRef) -> io::Result<()> {
        match self {
            NvDataOwned::Unknown => todo!(),
            NvDataOwned::None => Ok(()), // ignore None values
            NvDataOwned::Bool => nv.add_boolean(name),
            NvDataOwned::BoolV(v) => v.insert_into(name, nv),
            NvDataOwned::Byte(v) => v.insert_into(name, nv),
            NvDataOwned::Int8(v) => v.insert_into(name, nv),
            NvDataOwned::Uint8(v) => v.insert_into(name, nv),
            NvDataOwned::Int16(v) => v.insert_into(name, nv),
            NvDataOwned::Uint16(v) => v.insert_into(name, nv),
            NvDataOwned::Int32(v) => v.insert_into(name, nv),
            NvDataOwned::Uint32(v) => v.insert_into(name, nv),
            NvDataOwned::Int64(v) => v.insert_into(name, nv),
            NvDataOwned::Uint64(v) => v.insert_into(name, nv),
            NvDataOwned::String(v) => v.insert_into(name, nv),
            NvDataOwned::NvList(v) => v.insert_into(name, nv),
            NvDataOwned::ByteArray(v) => v.insert_into(name, nv),
            NvDataOwned::Int8Array(v) => v.insert_into(name, nv),
            NvDataOwned::Uint8Array(v) => v.insert_into(name, nv),
            NvDataOwned::Int16Array(v) => v.insert_into(name, nv),
            NvDataOwned::Uint16Array(v) => v.insert_into(name, nv),
            NvDataOwned::Int32Array(v) => v.insert_into(name, nv),
            NvDataOwned::Uint32Array(v) => v.insert_into(name, nv),
            NvDataOwned::Int64Array(v) => v.insert_into(name, nv),
            NvDataOwned::Uint64Array(v) => v.insert_into(name, nv),
            NvDataOwned::NvListArray(_v) => todo!(),
        }
    }
}

pub fn to_bytes<T>(value: &T, code: NvEncoding) -> Result<Vec<u8>>
where
    T: Serialize,
{
    Ok(to_nvlist(value)?.pack(code)?)
}

pub fn to_nvlist<T>(value: &T) -> Result<NvList>
where
    T: Serialize,
{
    let mut serializer = Serializer {};
    match value.serialize(&mut serializer)? {
        NvDataOwned::NvList(v) => Ok(v),
        _ => todo!(),
    }
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = NvDataOwned;
    type Error = Error;

    type SerializeSeq = SeqSerializer<'a>;
    type SerializeTuple = Self::SerializeSeq;
    type SerializeTupleStruct = Self::SerializeSeq;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self::SerializeStruct;
    type SerializeStruct = StructSerializer<'a>;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        Ok(NvDataOwned::BoolV(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        Ok(NvDataOwned::Int8(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        Ok(NvDataOwned::Int16(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        Ok(NvDataOwned::Int32(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        Ok(NvDataOwned::Int64(v))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        Ok(NvDataOwned::Uint8(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        Ok(NvDataOwned::Uint16(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        Ok(NvDataOwned::Uint32(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        Ok(NvDataOwned::Uint64(v))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        Ok(NvDataOwned::String(ffi::CString::new(v.to_string())?))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        Ok(NvDataOwned::String(ffi::CString::new(v)?))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        // XXX copying around a potentially big buffer
        Ok(NvDataOwned::ByteArray(v.to_owned()))
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        // This member will be ignored
        Ok(NvDataOwned::None)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        // transparent
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        // Note: Option<()> will serialize to either not present, or nvlist_add_boolean().
        Ok(NvDataOwned::Bool)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        let mut nvl = NvList::new_unique_names();
        nvl.insert(variant, &value.serialize(self)?)?;
        Ok(NvDataOwned::NvList(nvl))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SeqSerializer::new(self, len))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        // XXX SeqSerializer requires that they be the same type (it turns into
        // nvlist_insert_*_array()), so maybe we should serialize as nvlist with
        // keys "0", "1", "2", etc.
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        todo!()
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(StructSerializer::new(self))
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        // name of the struct is ignored
        Ok(StructSerializer::new(self))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        todo!()
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = NvDataOwned;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok> {
        todo!()
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = NvDataOwned;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok> {
        todo!()
    }
}

#[derive(Debug)]
struct StructSerializer<'a> {
    serializer: &'a mut Serializer,
    nvl: NvList,
}

impl<'a> StructSerializer<'a> {
    fn new(serializer: &'a mut Serializer) -> Self {
        StructSerializer {
            serializer,
            nvl: NvList::new_unique_names(),
        }
    }
}

impl<'a> ser::SerializeStruct for StructSerializer<'a> {
    type Ok = NvDataOwned;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let data = value.serialize(&mut *self.serializer)?;
        self.nvl.insert(key, &data)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(NvDataOwned::NvList(self.nvl))
    }
}

impl<'a> ser::SerializeMap for StructSerializer<'a> {
    type Ok = NvDataOwned;
    type Error = Error;

    fn end(self) -> Result<Self::Ok> {
        Ok(NvDataOwned::NvList(self.nvl))
    }

    fn serialize_key<T>(&mut self, _key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::Message(
            "map must have whole entry available at once".to_string(),
        ))
    }

    fn serialize_value<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::Message(
            "map must have whole entry available at once".to_string(),
        ))
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<()>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        let key_owned: NvDataOwned = key.serialize(&mut *self.serializer)?;
        let key_string = match key_owned {
            NvDataOwned::String(v) => v,
            other => {
                return Err(Error::Message(format!(
                    "map keys must be strings, not {:?}",
                    other
                )))
            }
        };

        let data = value.serialize(&mut *self.serializer)?;
        self.nvl.insert(key_string, &data)?;
        Ok(())
    }
}

#[derive(Debug)]
struct SeqSerializer<'a> {
    serializer: &'a mut Serializer,
    vec: Vec<NvDataOwned>,
}

impl<'a> SeqSerializer<'a> {
    fn new(serializer: &'a mut Serializer, len: Option<usize>) -> Self {
        SeqSerializer {
            serializer,
            vec: Vec::with_capacity(len.unwrap_or(0)),
        }
    }
}

impl<'a> ser::SerializeTuple for SeqSerializer<'a> {
    type Ok = NvDataOwned;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a> ser::SerializeTupleStruct for SeqSerializer<'a> {
    type Ok = NvDataOwned;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a> ser::SerializeSeq for SeqSerializer<'a> {
    type Ok = NvDataOwned;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let data = value.serialize(&mut *self.serializer)?;
        self.vec.push(data);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        if self.vec.is_empty() {
            return Err(Error::Message(
                "zero-length (untyped) sequences not supported".to_string(),
            ));
        }
        match self.vec[0] {
            NvDataOwned::Unknown => Err(Error::Message(
                "sequence of unknown not supported".to_string(),
            )),
            NvDataOwned::None => todo!(),
            NvDataOwned::Bool => Err(Error::Message(
                "sequence of (value-less) bool not supported".to_string(),
            )),
            NvDataOwned::BoolV(_) => todo!(),
            NvDataOwned::Byte(_) => {
                let mut vec = Vec::with_capacity(self.vec.len());
                let len = self.vec.len();
                let mut iter = self.vec.into_iter();
                while let Some(NvDataOwned::Byte(v)) = iter.next() {
                    vec.push(v);
                }
                if vec.len() == len {
                    Ok(NvDataOwned::ByteArray(vec))
                } else {
                    Err(Error::Message(
                        "hetrogenious sequences not supported".to_string(),
                    ))
                }
            }
            NvDataOwned::Int8(_) => {
                todo!()
            }
            NvDataOwned::Uint8(_) => {
                todo!()
            }
            NvDataOwned::Int16(_) => {
                todo!()
            }
            NvDataOwned::Uint16(_) => todo!(),
            NvDataOwned::Int32(_) => todo!(),
            NvDataOwned::Uint32(_) => todo!(),
            NvDataOwned::Int64(_) => todo!(),
            NvDataOwned::Uint64(_) => todo!(),
            NvDataOwned::String(_) => todo!(),
            NvDataOwned::NvList(_) => {
                let mut vec = Vec::with_capacity(self.vec.len());
                let len = self.vec.len();
                let mut iter = self.vec.into_iter();
                while let Some(NvDataOwned::NvList(v)) = iter.next() {
                    vec.push(v);
                }
                if vec.len() == len {
                    Ok(NvDataOwned::NvListArray(vec))
                } else {
                    Err(Error::Message(
                        "hetrogenious sequences not supported".to_string(),
                    ))
                }
            }
            NvDataOwned::ByteArray(_) => Err(Error::Message(
                "sequence of sequence not supported".to_string(),
            )),
            NvDataOwned::Int8Array(_) => Err(Error::Message(
                "sequence of sequence not supported".to_string(),
            )),
            NvDataOwned::Uint8Array(_) => Err(Error::Message(
                "sequence of sequence not supported".to_string(),
            )),
            NvDataOwned::Int16Array(_) => Err(Error::Message(
                "sequence of sequence not supported".to_string(),
            )),
            NvDataOwned::Uint16Array(_) => Err(Error::Message(
                "sequence of sequence not supported".to_string(),
            )),
            NvDataOwned::Int32Array(_) => Err(Error::Message(
                "sequence of sequence not supported".to_string(),
            )),
            NvDataOwned::Uint32Array(_) => Err(Error::Message(
                "sequence of sequence not supported".to_string(),
            )),
            NvDataOwned::Int64Array(_) => Err(Error::Message(
                "sequence of sequence not supported".to_string(),
            )),
            NvDataOwned::Uint64Array(_) => Err(Error::Message(
                "sequence of sequence not supported".to_string(),
            )),
            NvDataOwned::NvListArray(_) => Err(Error::Message(
                "sequence of sequence not supported".to_string(),
            )),
        }
    }
}
