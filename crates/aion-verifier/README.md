# crates/aion-verifier — Verification Tiers L0-L6

Implements the tiers from `docs/VERIFICATION.md` / `docs/adr/0003-verification-tiering.md`. Produces `VerificationConfidence` for `docs/POIG-SPEC.md`.

**Status: real implementation (partial), milestone 9.** `cargo test -p aion-verifier` → 16/16 passing:

- `tier.rs` — the L0-L6 base-confidence table and adaptive verification-rate function, ported from `simulations/poig_sim/poig_sim/verification.py` (that Python module is the validated reference this crate must not silently diverge from — one test pins the confidence table values directly against it). Confirms L0 is never reward-eligible, the verification-rate floor is never zero even for a perfect reputation, and high-value jobs force a high check rate regardless of reputation.
- `commitment.rs` — L1 tier: a thin, tier-aware wrapper over `aion_crypto::commit_reveal` (the actual cryptographic primitive lives there; this module just maps its result into `VerificationConfidence`).
- `replicated.rs` — L2 tier: clusters N workers' results for the same job by mutual numeric tolerance and returns the largest cluster as consensus, with outliers flagged separately. Distinguishes "numerical nondeterminism" (small relative difference, e.g. hardware/kernel floating-point variance — not flagged) from "diverging/wrong result" (an outlier — flagged, but per `docs/protocol/SLASHING-SPEC.md` outlier status alone is explicitly NOT sufficient grounds for a slash; this module only identifies divergence, it does not decide fault).

**Not yet done:** L3 (deterministic-execution harness), L4 (optimistic dispute/bisection — the design is specified in `docs/adr/0003-verification-tiering.md` and `docs/adr/0007-reward-settlement-trust-model.md`, not yet implemented as code), L5 (ZKML, out of scope until the Phase 11 proof-of-concept), L6 (multi-validator quorum aggregation — already implemented in Python at `simulations/poig_sim/poig_sim/quorum.py`, including commit/reveal and the honestly-documented majority-collusion residual risk, but not yet ported to Rust). No wiring yet into `crates/aion-node` or a real job-execution pipeline (neither exists yet either).
