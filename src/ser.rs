
use crate::error::Error;
use serde::ser;
use serde::Serialize;

pub struct SeqSerializer<'a, 'b, T> {
    ser: &'a mut Serializer<'b, T>,
    len_sent: bool,
    remain: u64,
}

macro_rules! implement_serialize_seq {
    ($ty:ty) => {
        impl<'a, 'b, W: std::io::Write> $ty for SeqSerializer<'a, 'b, W> {
            type Ok = ();
            type Error = Error;
            fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
            where
                T: Serialize,
            {
                if self.remain == 0 {
                    return Err(Self::Error::Message("too many elements".to_string()));
                }
                if !self.len_sent {
                    self.remain.serialize(&mut *self.ser)?;
                }
                self.remain -= 1;
                value.serialize(&mut *self.ser)
            }
            fn end(self) -> Result<Self::Ok, Self::Error> {
                if self.remain == 0 {
                    return Ok(());
                }
                Err(Self::Error::Message("too less elements".to_string()))
            }
        }
    };
}

macro_rules! implement_serialize_struct {
    ($ty:ty) => {
        impl<'a, 'b, W: std::io::Write> $ty for SeqSerializer<'a, 'b, W> {
            type Ok = ();
            type Error = Error;
            fn serialize_field<T: ?Sized>(
                &mut self,
                _key: &'static str,
                value: &T,
            ) -> Result<(), Self::Error>
            where
                T: Serialize,
            {
                if self.remain == 0 {
                    return Err(Self::Error::Message("too many elements".to_string()));
                }
                if !self.len_sent {
                    self.remain.serialize(&mut *self.ser)?;
                }
                self.remain -= 1;
                value.serialize(&mut *self.ser)
            }
            fn end(self) -> Result<Self::Ok, Self::Error> {
                if self.remain == 0 {
                    return Ok(());
                }
                Err(Self::Error::Message("too less elements".to_string()))
            }
        }
    };
}

macro_rules! implement_serialize_tuple_struct {
    ($ty:ty) => {
        impl<'a, 'b, W: std::io::Write> $ty for SeqSerializer<'a, 'b, W> {
            type Ok = ();
            type Error = Error;
            fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
            where
                T: Serialize,
            {
                if self.remain == 0 {
                    return Err(Self::Error::Message("too many elements".to_string()));
                }
                if !self.len_sent {
                    self.remain.serialize(&mut *self.ser)?;
                }
                self.remain -= 1;
                value.serialize(&mut *self.ser)
            }
            fn end(self) -> Result<Self::Ok, Self::Error> {
                if self.remain == 0 {
                    return Ok(());
                }
                Err(Self::Error::Message("too less elements".to_string()))
            }
        }
    };
}

impl<'a, 'b, W: std::io::Write> ser::SerializeMap for SeqSerializer<'a, 'b, W> {
    type Ok = ();
    type Error = Error;
    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        key.serialize(&mut *self.ser)
    }
    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        if self.remain == 0 {
            return Err(Self::Error::Message("too many elements".to_string()));
        }
        if !self.len_sent {
            self.remain.serialize(&mut *self.ser)?;
        }
        self.remain -= 1;
        value.serialize(&mut *self.ser)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.remain == 0 {
            return Ok(());
        }
        Err(Self::Error::Message("too less elements".to_string()))
    }
}

implement_serialize_seq!(ser::SerializeSeq);
implement_serialize_seq!(ser::SerializeTuple);
implement_serialize_tuple_struct!(ser::SerializeTupleStruct);
implement_serialize_tuple_struct!(ser::SerializeTupleVariant);
implement_serialize_struct!(ser::SerializeStruct);
implement_serialize_struct!(ser::SerializeStructVariant);

pub struct Serializer<'a, W> {
    write: &'a mut W,
}

impl<'a, W: std::io::Write> Serializer<'a, W> {
    fn write_u64(&mut self, v: u64) -> crate::error::Result<()> {
        Ok(self.write.write_all(&v.to_le_bytes())?)
    }
    fn write_bytes(&mut self, v: &[u8]) -> crate::error::Result<()> {
        let len = v.len();
        let rem = len % 8;
        let pad = if rem == 0 { 0 } else { 8 - rem };
        self.write_u64(len.try_into()?)?;
        self.write.write_all(v)?;
        Ok(self.write.write_all(&vec![0; pad])?)
    }
}

macro_rules! serialize_unimplemented {
    ($name:ident) => {
        fn $name(self) -> Result<Self::Ok, Self::Error> {
            Err(Self::Error::NotImplemented)
        }
    };
    ($name:ident, $($arg:ident : $ty:ty),*) => {
        fn $name(self, $($arg: $ty,)*) -> Result<Self::Ok, Self::Error> {
            Err(Self::Error::NotImplemented)
        }
    };
}

impl<'a, 'b, W: std::io::Write> ser::Serializer for &'b mut Serializer<'a, W> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = SeqSerializer<'b, 'a, W>;
    type SerializeTuple = SeqSerializer<'b, 'a, W>;
    type SerializeTupleStruct = SeqSerializer<'b, 'a, W>;
    type SerializeTupleVariant = SeqSerializer<'b, 'a, W>;
    type SerializeMap = SeqSerializer<'b, 'a, W>;
    type SerializeStruct = SeqSerializer<'b, 'a, W>;
    type SerializeStructVariant = SeqSerializer<'b, 'a, W>;

    serialize_unimplemented!(serialize_i8, _v: i8);
    serialize_unimplemented!(serialize_i16, _v: i16);
    serialize_unimplemented!(serialize_i32, _v: i32);
    serialize_unimplemented!(serialize_i64, _v: i64);
    serialize_unimplemented!(serialize_f32, _v: f32);
    serialize_unimplemented!(serialize_f64, _v: f64);
    serialize_unimplemented!(serialize_char, _v: char);
    serialize_unimplemented!(serialize_unit);
    serialize_unimplemented!(serialize_unit_struct, _name: &'static str);
    serialize_unimplemented!(
        serialize_unit_variant,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str
    );

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v.into())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v.into())
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v.into())
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v.into())
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.write_u64(v)
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.serialize_bytes(v.as_bytes())
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let len = v.len();
        let rem = len % 8;
        let pad = if rem == 0 { 0 } else { 8 - rem };
        self.write.write_all(v)?;
        Ok(self.write.write_all(&vec![0; pad])?)
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        self.serialize_bool(true)?;
        value.serialize(self)
    }
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        if len.is_none() {
            return Err(Self::Error::NotImplemented);
        }
        let len = len.unwrap();
        Ok(SeqSerializer {
            ser: self,
            len_sent: false,
            remain: len.try_into()?,
        })
    }
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.serialize_seq(Some(len))
    }
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.serialize_seq(len)
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.serialize_seq(Some(len))
    }
}

#[test]
fn test_u64() {}
