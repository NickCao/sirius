use crate::error::Error;
use serde::de::{self, Deserialize};

pub struct Deserializer<'de, R> {
    read: &'de mut R,
}

macro_rules! deserialize_unimplemented {
    ($name:ident) => {
        fn $name<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
            unimplemented!()
        }
    };
}

pub struct SeqDeserializer<'a, 'de: 'a, T> {
    de: &'a mut Deserializer<'de, T>,
    remain: u64,
}

impl<'de, 'a, R: std::io::Read> de::SeqAccess<'de> for SeqDeserializer<'a, 'de, R> {
    type Error = Error;
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.remain == 0 {
            return Ok(None);
        }
        self.remain -= 1;
        seed.deserialize(&mut *self.de).map(Some)
    }
}

impl<'de, R> Deserializer<'de, R>
where
    R: std::io::Read,
{
    fn parse_u64(&mut self) -> crate::error::Result<u64> {
        let mut buf: [u8; 8] = [0; 8];
        self.read.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }
    fn parse_string(&mut self) -> crate::error::Result<String> {
        let len: usize = self.parse_u64()?.try_into()?;
        let rem = len % 8;
        let pad = if rem == 0 { 0 } else { 8 - rem };
        let mut buf = vec![0; len + pad];
        self.read.read_exact(&mut buf)?;
        Ok(String::from_utf8(buf[..len].to_vec())?)
    }
}

impl<'de, 'a, R> de::Deserializer<'de> for &'a mut Deserializer<'de, R>
where
    R: std::io::Read,
{
    type Error = Error;
    fn deserialize_u64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u64(self.parse_u64()?)
    }
    fn deserialize_string<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_string(self.parse_string()?)
    }
    fn deserialize_seq<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let len = self.parse_u64()?;
        let seq = SeqDeserializer {
            de: self,
            remain: len,
        };
        visitor.visit_seq(seq)
    }
    deserialize_unimplemented!(deserialize_any);
    deserialize_unimplemented!(deserialize_bool);
    deserialize_unimplemented!(deserialize_i8);
    deserialize_unimplemented!(deserialize_i16);
    deserialize_unimplemented!(deserialize_i32);
    deserialize_unimplemented!(deserialize_i64);
    deserialize_unimplemented!(deserialize_u8);
    deserialize_unimplemented!(deserialize_u16);
    deserialize_unimplemented!(deserialize_u32);
    deserialize_unimplemented!(deserialize_f32);
    deserialize_unimplemented!(deserialize_f64);
    deserialize_unimplemented!(deserialize_char);
    deserialize_unimplemented!(deserialize_str);
    deserialize_unimplemented!(deserialize_bytes);
    deserialize_unimplemented!(deserialize_byte_buf);
    deserialize_unimplemented!(deserialize_option);
    deserialize_unimplemented!(deserialize_unit);
    deserialize_unimplemented!(deserialize_map);
    deserialize_unimplemented!(deserialize_identifier);
    deserialize_unimplemented!(deserialize_ignored_any);
    fn deserialize_unit_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        unimplemented!()
    }
    fn deserialize_newtype_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        unimplemented!()
    }
    fn deserialize_tuple<V: de::Visitor<'de>>(
        self,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        unimplemented!()
    }
    fn deserialize_tuple_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        unimplemented!()
    }
    fn deserialize_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        unimplemented!()
    }
    fn deserialize_enum<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        unimplemented!()
    }
}

#[test]
fn test_u64() {
    let mut read: &[u8] = &[0x2a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00][..];
    assert_eq!(
        42,
        u64::deserialize(&mut Deserializer { read: &mut read }).unwrap()
    );
}

#[test]
fn test_string() {
    let mut read: &[u8] = &[
        0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8,
        'o' as u8, 0x00, 0x00, 0x00,
    ][..];
    assert_eq!(
        "hello",
        String::deserialize(&mut Deserializer { read: &mut read }).unwrap()
    );
}
