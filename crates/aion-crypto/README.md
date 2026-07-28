# crates/aion-crypto — Cryptographic Primitives

Uses reviewed, audited upstream crates only — `ed25519-dalek` (RustCrypto), `sha2` — no hand-rolled cryptography, per `AGENTS.md`'s explicit prohibition.

**Status: real implementation (partial), milestone 2.** `cargo test -p aion-crypto` → 9/9 passing:

- `identity.rs` — Ed25519 P2P identity keypair (`docs/ARCHITECTURE.md`'s Identity Separation section): generate, sign, and standalone `verify()` for a received signature. Tests cover the roundtrip, tampered-payload rejection, cross-identity rejection, and deterministic reconstruction from seed bytes.
- `commit_reveal.rs` — SHA-256 commit/reveal (`docs/VERIFICATION.md`'s Commit/reveal section), length-prefixed to avoid the classic concatenation-ambiguity bug in naive commit schemes (`commit("ab","c")` must not collide with `commit("a","bc")` — tested directly).

**Not yet done:** secp256k1 wallet identity (deferred — wallet identity for AION rides on the chosen L2's ERC-4337/EIP-7702 smart-account tooling per `docs/adr/0001-settlement-layer.md`, so this crate may only need to *reference* wallet-signed data rather than implement secp256k1 signing itself; revisit when Phase 10 clarifies the actual integration boundary rather than building it speculatively now). ZKML is out of scope for this crate entirely — see `docs/adr/0003-verification-tiering.md` and the Phase 11 proof-of-concept plan.
