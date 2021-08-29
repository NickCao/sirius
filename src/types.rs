use serde::{Deserialize, Serialize};

pub type DrvOutputs = std::collections::HashMap<DrvOutput, Realisation>;

#[derive(
    Deserialize, Serialize, Clone, Debug, std::cmp::Eq, std::cmp::PartialEq, std::hash::Hash,
)]
pub struct Hash {
    pub hash_size: u64,
    pub hash: Vec<u8>, // should be of length 64
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct StorePath {
    pub base_name: String,
}

#[derive(
    Deserialize, Serialize, Clone, Debug, std::cmp::Eq, std::cmp::PartialEq, std::hash::Hash,
)]
pub struct DrvOutput {
    pub drv_hash: Hash,
    pub output_name: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Realisation {
    pub id: DrvOutput,
    pub out_path: StorePath,
    pub signature: Vec<String>,
    pub dependent_realisations: std::collections::HashMap<DrvOutput, StorePath>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PathInfo {
    pub path: String,
    pub info: PathInfoWithoutPath,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PathInfoWithoutPath {
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

#[derive(Deserialize, Debug)]
pub struct ClientSettings {
    pub keep_failed: bool,
    pub keep_going: bool,
    pub try_fallback: bool,
    pub verbosity: u64,
    pub max_build_jobs: u64,
    pub max_silent_time: u64,
    pub use_build_hook: u64, // obsolete
    pub verbose_build: u64,
    pub log_type: u64,          // obsolete
    pub print_build_trace: u64, // obsolete
    pub build_cores: u64,
    pub use_subsitutes: bool,
    pub overrides: Vec<(String, String)>,
}
