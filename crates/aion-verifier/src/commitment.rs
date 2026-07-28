//! L1 tier: commitment/hash validation, per docs/VERIFICATION.md. Thin
//! wrapper over `aion_crypto::commit_reveal` -- the cryptographic
//! primitive already lives there (audited `sha2`-based commit/reveal);
//! this module is just the L1-tier-specific naming/entry-point so callers
//! working at the verification-tier level don't need to know the crypto
//! crate exists.

pub use aion_crypto::{commit, verify_reveal, Commitment};

/// Runs the L1 check: does (value, salt) actually reveal `commitment`?
/// Returns the L1 `VerificationConfidence` contribution directly (0.0 on
/// failure, `Tier::L1.base_confidence()` on success), per
/// `crate::tier::verification_confidence`.
pub fn check_l1(commitment: &Commitment, value: &[u8], salt: &[u8]) -> f64 {
    crate::tier::verification_confidence(
        crate::tier::Tier::L1,
        verify_reveal(commitment, value, salt),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_reveal_yields_l1_base_confidence() {
        let c = commit(b"result-hash", b"salt");
        assert_eq!(
            check_l1(&c, b"result-hash", b"salt"),
            crate::tier::Tier::L1.base_confidence()
        );
    }

    #[test]
    fn mismatched_reveal_yields_zero_confidence() {
        let c = commit(b"result-hash", b"salt");
        assert_eq!(check_l1(&c, b"different-result", b"salt"), 0.0);
    }
}
