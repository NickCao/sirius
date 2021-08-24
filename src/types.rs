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

#[derive(Deserialize, Debug)]
pub struct DerivationOutput {
    pub name: String,
    pub path_s: String,
    pub hash_algo: String,
    pub hash: String,
}

#[derive(Deserialize, Debug)]
pub struct BasicDerivation {
    pub name: String, // TODO: parse name from path
    pub outputs: Vec<DerivationOutput>,
    pub input_srcs: Vec<String>,
    pub platform: String,
    pub builder: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
}
