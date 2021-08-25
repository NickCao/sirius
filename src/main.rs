use argh::FromArgs;
use serde::{Deserialize, Serialize};
use sirius::consts::{self, Op};
use sirius::de::Deserializer;
use sirius::ser::Serializer;
use sirius::types::*;

#[derive(FromArgs)]
/// sirius
struct Args {
    /// path to store
    #[argh(option)]
    store: String,
    /// path to socket
    #[argh(option)]
    socket: String,
}

fn main() {
    let args: Args = argh::from_env();
    let ln = std::os::unix::net::UnixListener::bind(args.socket).unwrap();
    let db = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    for stream in ln.incoming() {
        match stream {
            Ok(stream) => {
                let db = db.clone();
                let store = args.store.clone();
                std::thread::spawn(move || handle(stream, std::path::Path::new(&store), db));
            }
            Err(err) => panic!("{}", err),
        }
    }
}

fn handle(
    conn: std::os::unix::net::UnixStream,
    store: &std::path::Path,
    db: std::sync::Arc<std::sync::RwLock<std::collections::HashMap<String, PathInfo>>>,
) {
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
            /*
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
            */
            Op::QueryPathInfo => {
                String::deserialize(&mut des).unwrap();
                consts::STDERR_LAST.serialize(&mut ser).unwrap();
                Some(PathInfoWithoutPath {
                    deriver: "".to_string(),
                    hash: "0sg9f58l1jj88w6pdrfdpj5x9b1zrwszk84j81zvby36q9whhhqa".to_string(),
                    references: vec![],
                    registration_time: 0,
                    nar_size: 120,
                    ultimate: true,
                    sigs: vec![],
                    ca: "".to_string(),
                })
                .serialize(&mut ser)
                .unwrap();
            }
            Op::QueryValidPaths => {
                let paths = Vec::<String>::deserialize(&mut des).unwrap();
                bool::deserialize(&mut des).unwrap();
                // TODO: impl path query
                consts::STDERR_LAST.serialize(&mut ser).unwrap();
                let db = &db.read().unwrap();
                db.iter()
                    .filter(|x| paths.contains(x.0))
                    .map(|x| x.0.clone())
                    .collect::<Vec<String>>()
                    .serialize(&mut ser)
                    .unwrap();
            }
            Op::AddMultipleToStore => {
                bool::deserialize(&mut des).unwrap();
                bool::deserialize(&mut des).unwrap();
                let mut fr = sirius::de::FramedReader::new(&mut read);
                let num_paths = u64::deserialize(&mut Deserializer::new(&mut fr)).unwrap();
                for _i in 0..num_paths {
                    let path = PathInfo::deserialize(&mut Deserializer::new(&mut fr)).unwrap();
                    let mut nar = libnar::Archive::new(&mut fr);
                    let s = store.join(std::path::Path::new(&path.path).strip_prefix("/").unwrap());
                    let s = s.to_str().unwrap();
                    println!("{}", s);
                    nar.unpack(s).unwrap();
                    db.write().unwrap().insert(path.path.clone(), path);
                }
                consts::STDERR_LAST.serialize(&mut ser).unwrap();
            }
            Op::BuildDerivation => {
                println!("{:?}", BasicDerivation::deserialize(&mut des).unwrap());
                u64::deserialize(&mut des).unwrap();
                consts::STDERR_LAST.serialize(&mut ser).unwrap();
                consts::BuildStatus::TransientFailure
                    .serialize(&mut ser)
                    .unwrap();
                String::from("built").serialize(&mut ser).unwrap();
                0_u64.serialize(&mut ser).unwrap();
                0_u64.serialize(&mut ser).unwrap();
                0_u64.serialize(&mut ser).unwrap();
                0_u64.serialize(&mut ser).unwrap();
                0_u64.serialize(&mut ser).unwrap();
            }
            Op::NarFromPath => {
                println!("{:?}", String::deserialize(&mut des).unwrap());
                consts::STDERR_LAST.serialize(&mut ser).unwrap();
                "nix-archive-1".serialize(&mut ser).unwrap();
                "(".serialize(&mut ser).unwrap();
                "type".serialize(&mut ser).unwrap();
                "regular".serialize(&mut ser).unwrap();
                "contents".serialize(&mut ser).unwrap();
                "hello".serialize(&mut ser).unwrap();
                ")".serialize(&mut ser).unwrap();
            }
            _ => {
                println!("{:?}", op);
                unimplemented!();
            }
        }
    }
}
