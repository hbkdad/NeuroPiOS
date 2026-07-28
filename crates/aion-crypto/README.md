# crates/aion-crypto — Cryptographic Primitives (planned, Phase 3+)

Ed25519 (P2P identity), secp256k1 (wallet identity), commitment schemes (commit/reveal per `docs/VERIFICATION.md`), replay-protection nonce handling. Uses reviewed, audited upstream crates only (e.g. `ed25519-dalek`, `k256`) — no hand-rolled cryptography, per the orchestration brief's explicit prohibition. **Status: not started.**
