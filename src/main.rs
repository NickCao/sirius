#![feature(duration_constants)]
use argh::FromArgs;
use kmpsearch::Haystack;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};
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
    /// path to bwrap
    #[argh(option)]
    bwrap: String,
    /// path to sh
    #[argh(option)]
    sh: String,
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
                let bwrap = args.bwrap.clone();
                let sh = args.sh.clone();
                std::thread::spawn(move || {
                    handle(stream, std::path::Path::new(&store), db, bwrap, sh)
                });
            }
            Err(err) => panic!("{}", err),
        }
    }
}

fn handle(
    conn: std::os::unix::net::UnixStream,
    store: &std::path::Path,
    db: std::sync::Arc<std::sync::RwLock<std::collections::HashMap<String, PathInfo>>>,
    bwrap: String,
    sh: String,
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
        println!("{:?}", op);
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
                let path = String::deserialize(&mut des).unwrap();
                consts::STDERR_LAST.serialize(&mut ser).unwrap();
                let db = db.read().unwrap();
                let info = db.get(&path);
                println!("{:?} {:?}", path, info);
                match info {
                    Some(path) => Some(path.info.clone()).serialize(&mut ser).unwrap(),
                    None => Option::<PathInfoWithoutPath>::None
                        .serialize(&mut ser)
                        .unwrap(),
                }
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
                let drv = BasicDerivation::deserialize(&mut des).unwrap();
                u64::deserialize(&mut des).unwrap();
                let env_overrride: std::collections::HashMap<String, String> = drv
                    .outputs
                    .iter()
                    .map(|x| (x.name.clone(), x.path_s.clone()))
                    .collect();
                let tmp_store = tempdir::TempDir::new("sirius").unwrap();
                let binds: Vec<String> = drv
                    .input_srcs
                    .iter()
                    .map(|x| {
                        [
                            "--ro-bind".to_string(),
                            store
                                .join(std::path::Path::new(&x).strip_prefix("/").unwrap())
                                .to_str()
                                .unwrap()
                                .to_string(),
                            x.to_string(),
                        ]
                    })
                    .flatten()
                    .collect();
                let mut cmd = std::process::Command::new(&bwrap);
                cmd.args([
                    "--unshare-all",
                    "--die-with-parent",
                    "--bind",
                    tmp_store.path().to_str().unwrap(),
                    "/",
                    "--dev",
                    "/dev",
                    "--proc",
                    "/proc",
                    "--tmpfs",
                    "/build",
                    "--chdir",
                    "/build",
                    "--ro-bind",
                    &sh,
                    "/bin/sh",
                ])
                .args(binds)
                .env_clear()
                .env("PATH", "/path-not-set")
                .env("HOME", "/homeless-shelter")
                .env("NIX_STORE", "/nix/store")
                .env("NIX_BUILD_CORES", "12")
                .env("NIX_BUILD_TOP", "/build")
                .env("TMPDIR", "/build")
                .env("TEMPDIR", "/build")
                .env("TMP", "/build")
                .env("TEMP", "/build")
                .envs(drv.env)
                .envs(env_overrride)
                .arg(drv.builder)
                .args(drv.args);
                let status = cmd.status().unwrap();
                if status.success() {
                    let refs = drv
                        .input_srcs
                        .iter()
                        .map(|x| {
                            (
                                &std::path::Path::new(x)
                                    .file_name()
                                    .unwrap()
                                    .to_str()
                                    .unwrap()
                                    .as_bytes()[..32],
                                x,
                            )
                        })
                        .collect::<Vec<(&[u8], &String)>>();
                    drv.outputs
                        .iter()
                        .map(|x| {
                            let from_path = tmp_store
                                .path()
                                .join(std::path::Path::new(&x.path_s).strip_prefix("/").unwrap());
                            let to_path = store
                                .join(std::path::Path::new(&x.path_s).strip_prefix("/").unwrap());
                            // TODO: rewrite in rust
                            std::process::Command::new("mv")
                                .arg(&from_path)
                                .arg(&to_path)
                                .spawn()
                                .unwrap();

                            std::thread::sleep(std::time::Duration::SECOND);
                            let data = libnar::to_vec(&to_path).unwrap();
                            let mut hasher = sha2::Sha256::new();
                            hasher.update(&data);
                            db.write().unwrap().insert(
                                x.path_s.clone(),
                                PathInfo {
                                    path: x.path_s.clone(),
                                    info: PathInfoWithoutPath {
                                        deriver: "".to_string(),
                                        hash: format!("{:x}", hasher.finalize()),
                                        ca: "".to_string(),
                                        nar_size: data.len().try_into().unwrap(),
                                        references: refs
                                            .iter()
                                            .filter(|x| data.contains_needle(x.0))
                                            .map(|x| x.1.clone())
                                            .collect(),
                                        registration_time: 0,
                                        sigs: vec![],
                                        ultimate: true,
                                    },
                                },
                            );
                        })
                        .for_each(drop);
                    consts::STDERR_LAST.serialize(&mut ser).unwrap();
                    consts::BuildStatus::Built.serialize(&mut ser).unwrap();
                } else {
                    consts::STDERR_LAST.serialize(&mut ser).unwrap();
                    consts::BuildStatus::MiscFailure
                        .serialize(&mut ser)
                        .unwrap();
                }
                String::from("built").serialize(&mut ser).unwrap();
                0_u64.serialize(&mut ser).unwrap();
                0_u64.serialize(&mut ser).unwrap();
                0_u64.serialize(&mut ser).unwrap();
                0_u64.serialize(&mut ser).unwrap();
                0_u64.serialize(&mut ser).unwrap(); // map of drv output to realization
            }
            Op::NarFromPath => {
                let path = String::deserialize(&mut des).unwrap();
                consts::STDERR_LAST.serialize(&mut ser).unwrap();
                libnar::to_writer(
                    &mut write,
                    store.join(std::path::Path::new(&path).strip_prefix("/").unwrap()),
                )
                .unwrap()
            }
            _ => {
                println!("{:?}", op);
                unimplemented!();
            }
        }
    }
}
