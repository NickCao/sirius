use serde::{Deserialize, Serialize};
use sirius::consts::{self, Op};
use sirius::de::Deserializer;
use sirius::ser::Serializer;
use sirius::types::*;

fn handle(conn: std::os::unix::net::UnixStream) {
    let mut read = conn.try_clone().unwrap();
    let mut write = conn.try_clone().unwrap();
    {
        let mut ser = Serializer::new(&mut write);
        let mut des = Deserializer::new(&mut read);
        assert_eq!(consts::WORKER_MAGIC_1, u64::deserialize(&mut des).unwrap());
        consts::WORKER_MAGIC_2.serialize(&mut ser).unwrap();
        consts::PROTOCOL_VERSION.serialize(&mut ser).unwrap();
        assert_eq!(288, u64::deserialize(&mut des).unwrap());
        assert_eq!(None, Option::<u64>::deserialize(&mut des).unwrap());
        consts::STDERR_LAST.serialize(&mut ser).unwrap();
    }
    loop {
        let mut des = Deserializer::new(&mut read);
        let mut ser = Serializer::new(&mut write);
        let op = match Op::deserialize(&mut des) {
            Ok(op) => op,
            Err(sirius::error::Error::IO(e)) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    break;
                }
                panic!()
            }
            _ => panic!(),
        };
        match op {
            Op::Nop => (),
            Op::SetOptions => {
                println!("{:?}", ClientSettings::deserialize(&mut des).unwrap());
                consts::STDERR_LAST.serialize(&mut ser).unwrap();
            }
            Op::AddToStoreNar => {
                PathInfo::deserialize(&mut des).unwrap();
                bool::deserialize(&mut des).unwrap();
                bool::deserialize(&mut des).unwrap();
                let mut fr = sirius::de::FramedReader::new(&mut read);
                let mut nar = libnar::Archive::new(&mut fr);
                let entries = nar.entries().unwrap();
                for entry in entries {
                    println!("{:?}", entry.unwrap());
                }
                consts::STDERR_LAST.serialize(&mut ser).unwrap();
            }
            /*
            Op::QueryPathInfo => {
                String::deserialize(&mut des).unwrap();
            }
            */
            Op::QueryValidPaths => {
                Vec::<String>::deserialize(&mut des).unwrap();
                bool::deserialize(&mut des).unwrap();
                // TODO: impl path query
                consts::STDERR_LAST.serialize(&mut ser).unwrap();
                0_u64.serialize(&mut ser).unwrap();
            }
            Op::AddMultipleToStore => {
                bool::deserialize(&mut des).unwrap();
                bool::deserialize(&mut des).unwrap();
                let mut fr = sirius::de::FramedReader::new(&mut read);
                let num_paths = u64::deserialize(&mut Deserializer::new(&mut fr)).unwrap();
                for _i in 0..num_paths {
                    println!(
                        "{:?}",
                        PathInfo::deserialize(&mut Deserializer::new(&mut fr)).unwrap()
                    );
                    let mut nar = libnar::Archive::new(&mut fr);
                    let entries = nar.entries().unwrap();
                    for entry in entries {
                        entry.unwrap();
                    }
                }
                consts::STDERR_LAST.serialize(&mut ser).unwrap();
            }
            Op::BuildDerivation => {
                println!("{:?}", BasicDerivation::deserialize(&mut des).unwrap());
                u64::deserialize(&mut des).unwrap();
                // TODO: impl
            }
            /*
            Op::NarFromPath => {
                println!("{:?}", String::deserialize(&mut des).unwrap());
            }
            */
            _ => {
                println!("{:?}", op);
                unimplemented!();
            }
        }
    }
}

fn main() {
    let ln = std::os::unix::net::UnixListener::bind("target/socket").unwrap();
    for stream in ln.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(|| handle(stream));
            }
            Err(err) => panic!("{}", err),
        }
    }
}
