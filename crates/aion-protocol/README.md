# crates/aion-protocol — Wire Protocol Types (planned, Phase 3)

Rust implementation of the job object, model manifest, benchmark object, and job-lifecycle state machine specified in `docs/PROTOCOL.md`, including canonical serialization and signature verification. The Python reference implementation in `simulations/poig_sim/poig_sim/protocol.py` (Phase 2) should be treated as a semantics reference, not a wire-format reference — this crate defines the actual wire encoding. **Status: not started.**
