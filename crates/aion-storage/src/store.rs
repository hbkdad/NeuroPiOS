//! A real, disk-backed content-addressed store: `put` writes a blob under
//! a filename derived from its own [`Cid`](cid::Cid) (see `content_id.rs`),
//! and `get` re-derives the CID from the bytes actually read back off
//! disk and rejects a mismatch rather than silently returning corrupted
//! or tampered content under a CID that no longer describes it. This is
//! the "integrity verification" this crate's original README stub named
//! as in-scope, now real rather than aspirational.
//!
//! Honest scope note: this is a *local* store only. No peer
//! seeding/network distribution (Bitswap-equivalent) exists here --
//! that half of `crates/aion-storage`'s eventual scope needs a real
//! multi-host network to build against meaningfully, which this dev
//! container doesn't have (same reasoning documented for
//! `crates/aion-p2p`'s NAT-traversal testing). No sharded directory
//! layout either (a real large-scale store would likely shard by CID
//! prefix to avoid one huge flat directory) -- not needed at the scale
//! this milestone tests against, and not built ahead of an actual need.

use crate::content_id::compute_cid;
use cid::Cid;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("I/O error accessing the content store: {0}")]
    Io(#[from] std::io::Error),
    #[error("no content stored under CID {0}")]
    NotFound(Cid),
    #[error(
        "integrity check failed reading CID {expected}: bytes on disk actually hash to {actual} -- the stored blob is corrupted or was tampered with"
    )]
    IntegrityMismatch {
        // Boxed per clippy::result_large_err: Cid embeds a full multihash
        // buffer, so two of them inline would make every StorageError
        // (and therefore every Result<_, StorageError>) needlessly large
        // to move around, even for the common non-error path.
        expected: Box<Cid>,
        actual: Box<Cid>,
    },
}

/// A local, disk-backed, content-addressed store rooted at a single
/// directory. Every stored file's name IS its CID's string form, so a
/// CID can always be turned into a path without a separate index.
pub struct ContentStore {
    root: PathBuf,
}

impl ContentStore {
    /// Opens (creating if necessary) a content store rooted at `root`.
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let root = root.into();
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    fn path_for(&self, cid: &Cid) -> PathBuf {
        self.root.join(cid.to_string())
    }

    /// Stores `data`, returning the [`Cid`] it's now retrievable under.
    /// Storing identical bytes twice is idempotent (the second `put`
    /// overwrites the same path with byte-identical content, not a
    /// duplicate entry) since the CID -- and therefore the filename --
    /// is wholly determined by the bytes themselves.
    pub fn put(&self, data: &[u8]) -> Result<Cid, StorageError> {
        let cid = compute_cid(data);
        let path = self.path_for(&cid);
        fs::write(path, data)?;
        Ok(cid)
    }

    /// Retrieves the bytes stored under `cid`, verifying on every read
    /// that they still actually hash to `cid` before returning them.
    pub fn get(&self, cid: &Cid) -> Result<Vec<u8>, StorageError> {
        let path = self.path_for(cid);
        let data = match fs::read(&path) {
            Ok(data) => data,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(StorageError::NotFound(*cid))
            }
            Err(e) => return Err(e.into()),
        };
        let actual = compute_cid(&data);
        if actual != *cid {
            return Err(StorageError::IntegrityMismatch {
                expected: Box::new(*cid),
                actual: Box::new(actual),
            });
        }
        Ok(data)
    }

    /// Whether `cid` is currently retrievable (does not itself perform
    /// the integrity check `get` does -- a corrupted file still
    /// `contains`, since presence and integrity are different questions).
    pub fn contains(&self, cid: &Cid) -> bool {
        self.path_for(cid).exists()
    }

    /// Root directory backing this store, for tests that need to poke at
    /// the filesystem directly (e.g. simulating corruption).
    pub fn root(&self) -> &Path {
        &self.root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_temp_store() -> (ContentStore, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let store = ContentStore::open(dir.path()).unwrap();
        (store, dir)
    }

    #[test]
    fn put_then_get_round_trips_the_exact_bytes() {
        let (store, _dir) = open_temp_store();
        let data = b"a model weights blob, or a stand-in for one".to_vec();
        let cid = store.put(&data).unwrap();
        let retrieved = store.get(&cid).unwrap();
        assert_eq!(retrieved, data);
    }

    #[test]
    fn putting_identical_content_twice_is_idempotent() {
        let (store, _dir) = open_temp_store();
        let data = b"same content, put twice".to_vec();
        let cid_a = store.put(&data).unwrap();
        let cid_b = store.put(&data).unwrap();
        assert_eq!(cid_a, cid_b);
        assert_eq!(store.get(&cid_a).unwrap(), data);
    }

    #[test]
    fn distinct_content_is_stored_under_distinct_cids_and_both_retrievable() {
        let (store, _dir) = open_temp_store();
        let cid_a = store.put(b"first blob").unwrap();
        let cid_b = store.put(b"second blob").unwrap();
        assert_ne!(cid_a, cid_b);
        assert_eq!(store.get(&cid_a).unwrap(), b"first blob");
        assert_eq!(store.get(&cid_b).unwrap(), b"second blob");
    }

    #[test]
    fn get_on_an_unknown_cid_returns_not_found_rather_than_panicking() {
        let (store, _dir) = open_temp_store();
        let never_stored = compute_cid(b"never actually stored");
        let err = store.get(&never_stored).unwrap_err();
        assert!(matches!(err, StorageError::NotFound(cid) if cid == never_stored));
    }

    #[test]
    fn contains_reflects_presence_without_reading_the_content() {
        let (store, _dir) = open_temp_store();
        let cid = store.put(b"present").unwrap();
        let absent = compute_cid(b"absent");
        assert!(store.contains(&cid));
        assert!(!store.contains(&absent));
    }

    #[test]
    fn a_corrupted_file_on_disk_is_detected_on_read_not_silently_returned() {
        // The real integrity-verification property this store exists to
        // provide: if the bytes on disk no longer hash to the CID they're
        // filed under (bit-rot, truncation, tampering), get() must fail
        // loudly rather than hand back wrong bytes under a CID that no
        // longer describes them.
        let (store, _dir) = open_temp_store();
        let cid = store.put(b"the original, uncorrupted content").unwrap();

        // Simulate corruption: overwrite the stored file's bytes directly
        // on disk, bypassing the store's own put() (which would compute a
        // new, different CID for the new content instead of reusing this
        // one).
        std::fs::write(store.root().join(cid.to_string()), b"corrupted!!").unwrap();

        let err = store.get(&cid).unwrap_err();
        match err {
            StorageError::IntegrityMismatch { expected, actual } => {
                assert_eq!(*expected, cid);
                assert_ne!(*actual, cid);
            }
            other => panic!("expected IntegrityMismatch, got {other:?}"),
        }
    }

    #[test]
    fn opening_a_store_creates_its_root_directory_if_missing() {
        let dir = tempfile::tempdir().unwrap();
        let nested_root = dir.path().join("does/not/exist/yet");
        assert!(!nested_root.exists());
        let store = ContentStore::open(&nested_root).unwrap();
        assert!(nested_root.exists());
        // and it's actually usable, not just present:
        let cid = store.put(b"works after creation").unwrap();
        assert_eq!(store.get(&cid).unwrap(), b"works after creation");
    }
}
