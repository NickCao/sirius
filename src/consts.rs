use serde_repr::{Deserialize_repr, Serialize_repr};

pub const WORKER_MAGIC_1: u64 = 0x6e697863;
pub const WORKER_MAGIC_2: u64 = 0x6478696f;

#[derive(Serialize_repr, Deserialize_repr, Debug)]
#[repr(u64)]
pub enum Op {
    Nop = 0,
    IsValidPath = 1,
    HasSubstitutes = 3,
    QueryReferrers = 6,
    AddToStore = 7,
    BuildPaths = 9,
    EnsurePath = 10,
    AddTempRoot = 11,
    AddIndirectRoot = 12,
    SyncWithGC = 13,
    FindRoots = 14,
    SetOptions = 19,
    CollectGarbage = 20,
    QuerySubstitutablePathInfo = 21,
    QueryAllValidPaths = 23,
    QueryPathInfo = 26,
    QueryPathFromHashPart = 29,
    QuerySubstitutablePathInfos = 30,
    QueryValidPaths = 31,
    QuerySubstitutablePaths = 32,
    QueryValidDerivers = 33,
    OptimiseStore = 34,
    VerifyStore = 35,
    BuildDerivation = 36,
    AddSignatures = 37,
    NarFromPath = 38,
    AddToStoreNar = 39,
    QueryMissing = 40,
    QueryDerivationOutputMap = 41,
    RegisterDrvOutput = 42,
    QueryRealisation = 43,
    AddMultipleToStore = 44,
}
