//! SHA-256 based commit/reveal, per docs/VERIFICATION.md's Commit/reveal
//! section: prevents a validator (or benchmark submitter, or bidder) from
//! copying another party's already-visible answer before submitting its
//! own. Uses the audited `sha2` crate, not a hand-rolled hash construction.

use sha2::{Digest, Sha256};

pub type Commitment = [u8; 32];

/// Commit to `value` using `salt` as a per-commitment nonce (required --
/// reusing a salt across different values leaks information and, worse,
/// reusing the same (value, salt) pair for two different commitments makes
/// them trivially linkable).
pub fn commit(value: &[u8], salt: &[u8]) -> Commitment {
    let mut hasher = Sha256::new();
    hasher.update((value.len() as u64).to_le_bytes());
    hasher.update(value);
    hasher.update((salt.len() as u64).to_le_bytes());
    hasher.update(salt);
    hasher.finalize().into()
}

/// Check that (value, salt) actually reveals the given commitment.
pub fn verify_reveal(commitment: &Commitment, value: &[u8], salt: &[u8]) -> bool {
    commit(value, salt) == *commitment
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_reveal_verifies() {
        let c = commit(b"pass", b"salt-123");
        assert!(verify_reveal(&c, b"pass", b"salt-123"));
    }

    #[test]
    fn mismatched_value_fails_to_verify() {
        let c = commit(b"pass", b"salt-123");
        assert!(!verify_reveal(&c, b"fail", b"salt-123"));
    }

    #[test]
    fn mismatched_salt_fails_to_verify() {
        let c = commit(b"pass", b"salt-123");
        assert!(!verify_reveal(&c, b"pass", b"different-salt"));
    }

    #[test]
    fn commitment_does_not_trivially_reveal_the_value() {
        // Length-prefixed encoding prevents the classic ("ab","c") vs
        // ("a","bc") concatenation-ambiguity bug in commit schemes.
        let c1 = commit(b"ab", b"c");
        let c2 = commit(b"a", b"bc");
        assert_ne!(c1, c2);
    }

    #[test]
    fn same_value_different_salt_produces_different_commitments() {
        let c1 = commit(b"pass", b"salt-1");
        let c2 = commit(b"pass", b"salt-2");
        assert_ne!(c1, c2);
    }
}
