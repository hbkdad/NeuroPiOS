//! Ed25519 identity signing, using the audited `ed25519-dalek` crate --
//! no hand-rolled cryptography, per AGENTS.md's explicit prohibition.
//! This is the P2P identity keypair described in docs/ARCHITECTURE.md's
//! Identity Separation section (distinct from the secp256k1 wallet
//! identity, which belongs with the chosen L2's account-abstraction
//! tooling rather than this crate -- see docs/adr/0001-settlement-layer.md).

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand_core::OsRng;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SignatureError {
    #[error("signature verification failed")]
    InvalidSignature,
    #[error("malformed public key bytes")]
    MalformedPublicKey,
}

/// A P2P identity keypair. The signing key never leaves this struct.
pub struct Identity {
    signing_key: SigningKey,
}

impl Identity {
    pub fn generate() -> Self {
        Self {
            signing_key: SigningKey::generate(&mut OsRng),
        }
    }

    pub fn from_bytes(secret_bytes: &[u8; 32]) -> Self {
        Self {
            signing_key: SigningKey::from_bytes(secret_bytes),
        }
    }

    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.signing_key.verifying_key().to_bytes()
    }

    pub fn sign(&self, payload: &[u8]) -> Signature {
        self.signing_key.sign(payload)
    }
}

/// Verify a signature against a raw public key, independent of any live
/// `Identity` instance -- this is what a peer receiving a signed message
/// actually calls.
pub fn verify(
    public_key_bytes: &[u8; 32],
    payload: &[u8],
    signature: &Signature,
) -> Result<(), SignatureError> {
    let verifying_key = VerifyingKey::from_bytes(public_key_bytes)
        .map_err(|_| SignatureError::MalformedPublicKey)?;
    verifying_key
        .verify(payload, signature)
        .map_err(|_| SignatureError::InvalidSignature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_then_verify_roundtrip_succeeds() {
        let identity = Identity::generate();
        let payload = b"job-payload-bytes";
        let sig = identity.sign(payload);
        assert!(verify(&identity.public_key_bytes(), payload, &sig).is_ok());
    }

    #[test]
    fn tampered_payload_fails_verification() {
        let identity = Identity::generate();
        let sig = identity.sign(b"original payload");
        let result = verify(&identity.public_key_bytes(), b"tampered payload", &sig);
        assert_eq!(result, Err(SignatureError::InvalidSignature));
    }

    #[test]
    fn signature_from_a_different_identity_fails_verification() {
        let alice = Identity::generate();
        let bob = Identity::generate();
        let payload = b"same payload";
        let sig = alice.sign(payload);
        // Verifying alice's signature against bob's public key must fail.
        let result = verify(&bob.public_key_bytes(), payload, &sig);
        assert_eq!(result, Err(SignatureError::InvalidSignature));
    }

    #[test]
    fn deterministic_key_reconstruction_from_seed_bytes() {
        let seed = [7u8; 32];
        let a = Identity::from_bytes(&seed);
        let b = Identity::from_bytes(&seed);
        assert_eq!(a.public_key_bytes(), b.public_key_bytes());
    }
}
