use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct PathInfo {
    pub path: String,
    pub deriver: String,
    pub hash: String,
    pub references: Vec<String>,
    pub registration_time: u64,
    pub nar_size: u64,
    pub ultimate: bool,
    pub sigs: Vec<String>,
    pub ca: String,
}
