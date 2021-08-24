use crate::error::Error;

use serde::de;
use serde::Deserialize;

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
    fn parse_bool(&mut self) -> crate::error::Result<bool> {
        let num = self.parse_u64()?;
        Ok(num != 0)
    }
    fn parse_bytes(&mut self) -> crate::error::Result<Vec<u8>> {
        let len: usize = self.parse_u64()?.try_into()?;
        let rem = len % 8;
        let pad = if rem == 0 { 0 } else { 8 - rem };
        let mut buf = vec![0; len + pad];
        self.read.read_exact(&mut buf)?;
        buf.truncate(len);
        Ok(buf)
    }
    fn parse_string(&mut self) -> crate::error::Result<String> {
        let buf = self.parse_bytes()?;
        Ok(String::from_utf8(buf)?)
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
    fn deserialize_bool<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_bool(self.parse_bool()?)
    }
    fn deserialize_option<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let some = self.parse_bool()?;
        if some {
            visitor.visit_some(self)
        } else {
            visitor.visit_none()
        }
    }
    fn deserialize_byte_buf<V: de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_byte_buf(self.parse_bytes()?)
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
    fn deserialize_tuple<V: de::Visitor<'de>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        let seq = SeqDeserializer {
            de: self,
            remain: len.try_into()?,
        };
        visitor.visit_seq(seq)
    }
    fn deserialize_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_tuple(fields.len(), visitor)
    }
    fn deserialize_tuple_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_tuple(len, visitor)
    }
    deserialize_unimplemented!(deserialize_str);
    deserialize_unimplemented!(deserialize_any);
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
    deserialize_unimplemented!(deserialize_bytes);
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
    fn deserialize_enum<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        unimplemented!()
    }
}

struct FramedReader<'a, R> {
    read: &'a mut R,
    rem: usize,
}

impl<'a, R: std::io::Read> std::io::Read for FramedReader<'a, R> {
    fn read(&mut self, buf: &mut [u8]) -> std::result::Result<usize, std::io::Error> {
        if self.rem == 0 {
            self.rem = usize::deserialize(&mut Deserializer {
                read: &mut self.read,
            })
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        }
        let size = self.read.take(self.rem.try_into().unwrap()).read(buf)?;
        self.rem -= size;
        Ok(size)
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
        0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, b'h', b'e', b'l', b'l', b'o', 0x00, 0x00,
        0x00,
    ][..];
    assert_eq!(
        "hello",
        String::deserialize(&mut Deserializer { read: &mut read }).unwrap()
    );
}

#[test]
fn test_string_seq() {
    let mut read: &[u8] = &[
        0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, b'h', b'e', b'l', b'l', b'o', 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, b'h', b'e', b'l', b'l', b'o', 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, b'h', b'e', b'l', b'l', b'o', 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, b'h', b'e', b'l', b'l', b'o', 0x00, 0x00, 0x00, 0x05, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, b'h', b'e', b'l', b'l', b'o', 0x00, 0x00, 0x00,
    ][..];
    assert_eq!(
        vec!["hello"; 5],
        Vec::<String>::deserialize(&mut Deserializer { read: &mut read }).unwrap()
    )
}

#[test]
fn test_some() {
    let mut read: &[u8] = &[
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ][..];
    assert_eq!(
        Some(42),
        Option::<u64>::deserialize(&mut Deserializer { read: &mut read }).unwrap()
    )
}

#[test]
fn test_none() {
    let mut read: &[u8] = &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00][..];
    assert_eq!(
        None,
        Option::<u64>::deserialize(&mut Deserializer { read: &mut read }).unwrap()
    )
}

#[test]
fn test_tuple() {
    let mut read: &[u8] = &[
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ][..];
    assert_eq!(
        (1, 2, 3),
        <(u64, u64, u64)>::deserialize(&mut Deserializer { read: &mut read }).unwrap()
    )
}

#[test]
fn test_struct() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Test {
        name: String,
        num: u64,
    }
    let mut read: &[u8] = &[
        0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, b'h', b'e', b'l', b'l', b'o', 0x00, 0x00,
        0x00, 0x2a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ][..];
    assert_eq!(
        Test {
            name: String::from("hello"),
            num: 42
        },
        Test::deserialize(&mut Deserializer { read: &mut read }).unwrap()
    )
}

/*
#[test]
fn test_large() {
    let mut file = std::fs::File::open("data/de").unwrap();
    {
        let mut des = Deserializer { read: &mut file };
        assert_eq!(
            crate::consts::WORKER_MAGIC_1,
            u64::deserialize(&mut des).unwrap()
        );
        assert_eq!(288, u64::deserialize(&mut des).unwrap());
        assert_eq!(Some(0), Option::<u64>::deserialize(&mut des).unwrap());
    }
    loop {
        let mut des = Deserializer { read: &mut file };
        let op = match Op::deserialize(&mut des) {
            Ok(op) => op,
            Err(Error::IO(e)) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    break;
                }
                panic!()
            }
            _ => panic!(),
        };
        match op {
            Op::Nop => (),
            Op::QueryPathInfo => {
                String::deserialize(&mut des).unwrap();
            }
            Op::QueryValidPaths => {
                Vec::<String>::deserialize(&mut des).unwrap();
                bool::deserialize(&mut des).unwrap();
            }
            Op::AddMultipleToStore => {
                bool::deserialize(&mut des).unwrap();
                bool::deserialize(&mut des).unwrap();
                let mut read = FramedReader {
                    read: &mut file,
                    rem: 0,
                };
                let mut num_paths = 0;
                {
                    let mut des = Deserializer { read: &mut read };
                    num_paths = u64::deserialize(&mut des).unwrap();
                }
                for i in 0..num_paths {
                    {
                        let mut des = Deserializer { read: &mut read };
                        println!("{:?}", PathInfo::deserialize(&mut des).unwrap());
                    }
                    let mut nar = libnar::Archive::new(&mut read);
                    let entries = nar.entries().unwrap();
                    for entry in entries {
                        entry.unwrap();
                    }
                }
            }
            Op::BuildDerivation => {
                println!("{:?}", BasicDerivation::deserialize(&mut des).unwrap());
                u64::deserialize(&mut des).unwrap();
            }
            Op::NarFromPath => {
                println!("{:?}", String::deserialize(&mut des).unwrap());
            }
            _ => {
                println!("{:?}", op);
                unimplemented!();
            }
        }
    }
}
*/
