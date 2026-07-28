//! AION protocol types: canonical job object, model manifest, nonce-based
//! replay protection, and the job-lifecycle state machine. Wire-format
//! authority for docs/PROTOCOL.md -- see that document for the human
//! -readable spec this crate implements.

pub mod job;
pub mod lifecycle;
pub mod model_manifest;
pub mod nonce;

pub use job::{Job, JobType, VerificationPolicy, PROTOCOL_VERSION_MAJOR, PROTOCOL_VERSION_MINOR};
pub use lifecycle::{JobLifecycle, JobState, TransitionError};
pub use model_manifest::{ModelManifest, ModelManifestMetadata};
pub use nonce::{NonceTracker, ReplayError};
