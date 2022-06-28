use crate::de::Deserializer;
use crate::protocol::*;
use crate::ser::Serializer;
use crate::types::ValidPathInfo;
use serde::{Deserialize, Serialize};
use std::os::unix::net::UnixStream;
use thiserror::Error;

type Result<T> = std::result::Result<T, ClientError>;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("{0:?}")]
    IO(#[from] std::io::Error),
    #[error("{0}")]
    Serde(#[from] crate::error::Error),
    #[error("{0}")]
    Generic(String),
}

pub struct Client<W, R> {
    w: W,
    r: R,
}

pub fn daemon() -> Result<Client<UnixStream, UnixStream>> {
    let stream = UnixStream::connect("/nix/var/nix/daemon-socket/socket")?;
    Client::new(stream.try_clone()?, stream.try_clone()?)
}

impl<W: std::io::Write, R: std::io::Read> Client<W, R> {
    pub fn new(w: W, r: R) -> Result<Self> {
        let mut client = Self { w, r };

        client.write(WORKER_MAGIC_1)?;

        let magic: u64 = client.read()?;
        if magic != WORKER_MAGIC_2 {
            panic!("protocol magic mismatch");
        }

        let version: u64 = client.read()?;
        if protocol_version_major(version) != protocol_version_major(PROTOCOL_VERSION) {
            panic!("protocol major version mismatch")
        }
        if protocol_version_minor(version) < 33 {
            panic!("protocol minor version too low")
        }

        client.write(PROTOCOL_VERSION)?;
        client.write(0u64)?; // obsolete CPU affinity
        client.write(false)?; // obsolete reserve space

        let version: String = client.read()?;
        println!("daemon version string: {}", version);
        client.process_stderr()?;
        Ok(client)
    }
    pub fn write<T: Serialize>(&mut self, value: T) -> Result<()> {
        value.serialize(&mut Serializer::new(&mut self.w))?;
        Ok(())
    }
    pub fn read<'a, T: Deserialize<'a>>(&'a mut self) -> Result<T> {
        Ok(T::deserialize(&mut Deserializer::new(&mut self.r))?)
    }
    pub fn process_stderr(&mut self) -> Result<()> {
        loop {
            let msg: u64 = self.read()?;
            match msg {
                STDERR_WRITE => {
                    let s: String = self.read()?;
                    eprint!("{}", s);
                }
                STDERR_NEXT => unimplemented!(),
                STDERR_READ => unimplemented!(),
                STDERR_LAST => return Ok(()),
                STDERR_ERROR => unimplemented!(),
                STDERR_START_ACTIVITY => unimplemented!(),
                STDERR_STOP_ACTIVITY => unimplemented!(),
                STDERR_RESULT => unimplemented!(),
                _ => unimplemented!(),
            }
        }
    }
    pub fn query_path_info(&mut self, path: &str) -> Result<ValidPathInfo> {
        self.write(Op::QueryPathInfo)?;
        self.write(path)?;
        self.process_stderr()?;
        let valid: bool = self.read()?;
        if valid {
            let deriver: String = self.read()?;
            let hash: String = self.read()?;
            let references: Vec<String> = self.read()?;
            let registration_time: u64 = self.read()?;
            let nar_size: u64 = self.read()?;
            let ultimate: bool = self.read()?;
            let sigs: Vec<String> = self.read()?;
            let ca: String = self.read()?;
            Ok(ValidPathInfo {
                path: path.to_string(),
                deriver: Some(deriver),
                hash,
                nar_size,
                id: 0,
                ca: Some(ca),
                references,
                sigs,
                ultimate,
                registration_time,
            })
        } else {
            Err(ClientError::Generic(String::from("invalid path")))
        }
    }
}

#[cfg(test)]
mod test {
    use super::daemon;

    #[test]
    fn test_daemon() {
        let mut client = daemon().unwrap();
    }
}
