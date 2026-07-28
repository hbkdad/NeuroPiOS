//! AION content-addressed storage, per docs/PROTOCOL.md's model manifest
//! CID fields (`weightsCid`, `tokenizerCid`, `configCid`, `licenseCid`,
//! `benchmarkManifestCid`) and Phase 6 of docs/ROADMAP.md.
//!
//! Status: `content_id` (real IPFS-compatible CIDv1 generation) and
//! `store` (a disk-backed content store with on-read integrity
//! verification) are real, tested Rust. NOT done: peer seeding/network
//! distribution (the Bitswap-equivalent half of this crate's eventual
//! scope, per the original README stub) -- that needs a real multi-host
//! network to build meaningfully against, which this dev container
//! doesn't have. No wiring yet into `crates/aion-protocol`'s
//! `ModelManifest` (whose CID fields are still placeholder `String`s, not
//! real CIDs) or `crates/aion-node`'s Node Safety resource caps (gating
//! how much storage/bandwidth a node commits to seeding).

pub mod content_id;
pub mod store;

pub use content_id::compute_cid;
pub use store::{ContentStore, StorageError};
