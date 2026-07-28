//! Real, IPFS-compatible content addressing: SHA2-256 over raw bytes,
//! wrapped as a multihash, wrapped as a CIDv1 with the "raw" codec (model
//! weights/tokenizer/config/license files are opaque binary blobs, not an
//! IPFS DAG-PB/UnixFS structure, so the "raw" codec is the correct choice
//! -- see docs/PROTOCOL.md's `weightsCid`/`tokenizerCid`/etc. fields and
//! the IPFS/Kubo/Bitswap reference noted in docs/adr/0005-monorepo-structure.md's
//! context).
//!
//! Uses the same `cid`/`multihash` crates the wider Rust IPFS/libp2p
//! ecosystem uses (already a transitive dependency via `crates/aion-p2p`'s
//! `libp2p`) rather than hand-rolling a CID encoder -- per AGENTS.md's "no
//! hand-rolled cryptography" rule, and because CID's multibase/multicodec
//! framing is a real interoperability surface, not just an internal
//! implementation detail.

use cid::Cid;
use multihash::Multihash;
use sha2::{Digest, Sha256};

/// The `sha2-256` multicodec code, per the multiformats multicodec table
/// (<https://github.com/multiformats/multicodec>).
const SHA2_256_MULTICODEC: u64 = 0x12;
/// The `raw` multicodec code: content is an opaque binary blob with no
/// further IPLD structure, per the multiformats multicodec table.
const RAW_CODEC: u64 = 0x55;
/// SHA2-256 digests are exactly 32 bytes; `Multihash`'s const generic is
/// the maximum digest size it can hold, not the actual digest length.
const MULTIHASH_MAX_DIGEST_SIZE: usize = 64;

/// Computes the real CIDv1 (raw codec, SHA2-256 multihash) for `data`.
///
/// This is deterministic and content-addressed: identical bytes always
/// produce the identical CID, and the CID's textual form
/// (`bafkrei...`, base32-encoded) is the same one a real IPFS node would
/// compute for the same bytes with `--cid-version 1 --raw-leaves`.
pub fn compute_cid(data: &[u8]) -> Cid {
    let digest = Sha256::digest(data);
    let multihash = Multihash::<MULTIHASH_MAX_DIGEST_SIZE>::wrap(SHA2_256_MULTICODEC, &digest)
        .expect("a 32-byte SHA2-256 digest always fits within a 64-byte multihash buffer");
    Cid::new_v1(RAW_CODEC, multihash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_content_produces_identical_cids() {
        let a = compute_cid(b"the same bytes");
        let b = compute_cid(b"the same bytes");
        assert_eq!(a, b);
    }

    #[test]
    fn different_content_produces_different_cids() {
        let a = compute_cid(b"content one");
        let b = compute_cid(b"content two");
        assert_ne!(a, b);
    }

    #[test]
    fn matches_a_real_independently_verifiable_ipfs_cid() {
        // Golden value: the CIDv1 (raw codec, sha2-256) a real IPFS node
        // computes for these exact bytes with `--cid-version 1
        // --raw-leaves` is bafkreid3y7pm2css2xvjn3lhkdwrlo4tun3usglril7uxcb44otza5mkye --
        // independently reproducible (any correct CIDv1/raw/sha256
        // implementation, not just this one, must produce this exact
        // string for this exact input), proving genuine protocol
        // compatibility rather than an internally-consistent-but-made-up
        // hash scheme.
        let cid = compute_cid(b"hello aion");
        assert_eq!(
            cid.to_string(),
            "bafkreid3y7pm2css2xvjn3lhkdwrlo4tun3usglril7uxcb44otza5mkye"
        );
    }

    #[test]
    fn empty_content_still_produces_a_valid_cid() {
        let cid = compute_cid(b"");
        assert_eq!(cid.codec(), RAW_CODEC);
    }
}
