//! Gates `aion_storage::ContentStore` usage through `ResourceCaps`'s
//! `max_storage_mb`, per docs/ARCHITECTURE.md's Node Safety principle: "a
//! node never silently consumes host resources." Closes the same class of
//! gap `ResourceCaps::to_connection_limits()` (see `resource_caps.rs`)
//! already closed for P2P connection limits -- a node's storage cap and
//! its actual `ContentStore` disk usage used to be two independent,
//! unconnected knobs; this module connects them.
//!
//! Unlike the CPU/memory checks in `ResourceCaps::allows_new_work` (which
//! take a caller-observed current usage snapshot), storage usage here is
//! measured directly and authoritatively from the store itself via
//! `ContentStore::disk_usage_bytes` -- a real filesystem sum, not a
//! separately-tracked counter that could drift from what's actually on
//! disk.

use aion_storage::{ContentStore, StorageError};
use cid::Cid;
use std::path::PathBuf;
use thiserror::Error;

use crate::resource_caps::ResourceCaps;

/// Decimal MB, matching this codebase's existing convention of
/// documented-but-approximate unit assumptions elsewhere (e.g.
/// `resource_caps.rs`'s `ASSUMED_MBPS_PER_CONNECTION`) rather than
/// claiming a precise MiB/MB distinction the field name itself doesn't
/// specify.
const BYTES_PER_MB: u64 = 1_000_000;

#[derive(Debug, Error)]
pub enum CappedStorageError {
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error(
        "storing {attempted_bytes} more bytes would bring total usage to {would_be_bytes} bytes, exceeding this node's configured max_storage_mb cap of {cap_mb} MB ({cap_bytes} bytes) -- operator must raise the cap before this content can be accepted"
    )]
    WouldExceedStorageCap {
        attempted_bytes: u64,
        would_be_bytes: u64,
        cap_mb: u64,
        cap_bytes: u64,
    },
}

/// A `ContentStore` wrapped with mandatory `ResourceCaps` enforcement.
/// `put` is rejected outright -- no partial write, no silent truncation --
/// if it would push total disk usage over `caps.max_storage_mb`. `get`
/// (reading) is never gated, matching `resource_caps.rs`'s existing
/// principle that caps constrain accepting NEW work/resource commitment,
/// not access to what's already stored.
pub struct CappedContentStore {
    store: ContentStore,
    caps: ResourceCaps,
}

impl CappedContentStore {
    pub fn open(root: impl Into<PathBuf>, caps: ResourceCaps) -> Result<Self, CappedStorageError> {
        Ok(Self {
            store: ContentStore::open(root)?,
            caps,
        })
    }

    /// Stores `data`, rejecting the write entirely if it would bring this
    /// store's total disk usage above `caps.max_storage_mb`. At
    /// `ResourceCaps::locked_down()`'s default (`max_storage_mb: 0`),
    /// every `put` is rejected -- the same "accept no new work until the
    /// operator explicitly widens the cap" default `allows_new_work`
    /// already enforces for CPU/memory, extended here to storage.
    pub fn put(&self, data: &[u8]) -> Result<Cid, CappedStorageError> {
        let cap_bytes = self.caps.max_storage_mb * BYTES_PER_MB;
        let current_bytes = self.store.disk_usage_bytes()?;
        let attempted_bytes = data.len() as u64;
        let would_be_bytes = current_bytes + attempted_bytes;
        if would_be_bytes > cap_bytes {
            return Err(CappedStorageError::WouldExceedStorageCap {
                attempted_bytes,
                would_be_bytes,
                cap_mb: self.caps.max_storage_mb,
                cap_bytes,
            });
        }
        Ok(self.store.put(data)?)
    }

    /// Retrieval is never capped -- reading existing content isn't new
    /// resource commitment.
    pub fn get(&self, cid: &Cid) -> Result<Vec<u8>, CappedStorageError> {
        Ok(self.store.get(cid)?)
    }

    pub fn contains(&self, cid: &Cid) -> bool {
        self.store.contains(cid)
    }

    pub fn disk_usage_bytes(&self) -> Result<u64, CappedStorageError> {
        Ok(self.store.disk_usage_bytes()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn caps_with_storage_mb(max_storage_mb: u64) -> ResourceCaps {
        ResourceCaps {
            max_storage_mb,
            ..ResourceCaps::locked_down()
        }
    }

    #[test]
    fn locked_down_default_rejects_every_put_since_max_storage_mb_is_zero() {
        let dir = tempfile::tempdir().unwrap();
        let store = CappedContentStore::open(dir.path(), ResourceCaps::locked_down()).unwrap();
        let err = store.put(b"any content at all").unwrap_err();
        assert!(matches!(
            err,
            CappedStorageError::WouldExceedStorageCap { .. }
        ));
    }

    #[test]
    fn a_put_within_the_configured_cap_succeeds_and_is_retrievable() {
        let dir = tempfile::tempdir().unwrap();
        let store = CappedContentStore::open(dir.path(), caps_with_storage_mb(1)).unwrap(); // 1 MB
        let data = b"small enough to fit easily";
        let cid = store.put(data).unwrap();
        assert_eq!(store.get(&cid).unwrap(), data);
    }

    #[test]
    fn a_put_that_would_exceed_the_cap_is_rejected_without_writing_anything() {
        let dir = tempfile::tempdir().unwrap();
        // Cap small enough that a single large put clearly can't fit.
        let store = CappedContentStore::open(dir.path(), caps_with_storage_mb(0)).unwrap();
        let data = vec![0u8; 1024];
        let err = store.put(&data).unwrap_err();
        match err {
            CappedStorageError::WouldExceedStorageCap {
                attempted_bytes,
                cap_bytes,
                ..
            } => {
                assert_eq!(attempted_bytes, 1024);
                assert_eq!(cap_bytes, 0);
            }
            other => panic!("expected WouldExceedStorageCap, got {other:?}"),
        }
        // Confirm nothing was actually written: usage stays at zero.
        assert_eq!(store.disk_usage_bytes().unwrap(), 0);
    }

    #[test]
    fn the_cap_is_enforced_cumulatively_across_multiple_puts_not_just_per_call() {
        // A cap of 1 MB (1_000_000 bytes). Two puts of 600_000 bytes each
        // individually fit under the cap, but together they don't -- the
        // second put must be rejected even though it alone would be fine.
        let dir = tempfile::tempdir().unwrap();
        let store = CappedContentStore::open(dir.path(), caps_with_storage_mb(1)).unwrap();

        let first = vec![1u8; 600_000];
        store.put(&first).unwrap();

        let second = vec![2u8; 600_000]; // distinct content -> distinct CID, not deduped
        let err = store.put(&second).unwrap_err();
        assert!(matches!(
            err,
            CappedStorageError::WouldExceedStorageCap { .. }
        ));

        // The first put's content is still intact and the store's usage
        // wasn't corrupted by the rejected second attempt.
        assert_eq!(store.disk_usage_bytes().unwrap(), 600_000);
    }

    #[test]
    fn get_is_never_gated_by_the_storage_cap() {
        // Store something while the cap still has room, then shrink the
        // cap below current usage (simulating an operator tightening
        // caps after content already exists) -- get() must still work;
        // only NEW puts are gated.
        let dir = tempfile::tempdir().unwrap();
        let store = CappedContentStore::open(dir.path(), caps_with_storage_mb(10)).unwrap();
        let data = b"stored while there was room";
        let cid = store.put(data).unwrap();

        let tightened = CappedContentStore {
            store: ContentStore::open(dir.path()).unwrap(),
            caps: caps_with_storage_mb(0),
        };
        assert_eq!(tightened.get(&cid).unwrap(), data);
    }
}
