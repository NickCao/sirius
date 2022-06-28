use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize_repr, Deserialize_repr, Debug)]
#[repr(u64)]
pub enum BuildStatus {
    Built,
    Substituted,
    AlreadyValid,
    PermanentFailure,
    InputRejected,
    OutputRejected,
    TransientFailure, // possibly transient
    CachedFailure,    // no longer used
    TimedOut,
    MiscFailure,
    DependencyFailed,
    LogLimitExceeded,
    NotDeterministic,
}
