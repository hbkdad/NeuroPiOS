# AION Roadmap

Phases as defined by the orchestration brief. Status reflects reality as of milestone 2 (2026-07-28) — see PR #1 and its follow-up commits for the authoritative "what actually happened" record.

| Phase | Name | Status |
|---|---|---|
| 0 | Research | **Done** — capability matrix (incl. real toolchain inventory: Rust/Cargo, Node, Docker all confirmed available), 5 competitor analyses, verification matrix, research log |
| 1 | Specification | **Done** — ARCHITECTURE, PROTOCOL, POIG-SPEC, ECONOMIC-MODEL, VERIFICATION, SLASHING-SPEC, THREAT-MODEL, ECONOMIC-ATTACKS, 7 ADRs (added ADR-0007: reward settlement trust model, milestone 2) |
| 2 | Local simulator (no blockchain) | **Substantially done** — `simulations/poig_sim`: 14 pytest tests covering 5 attack scenarios (self-dealing, Sybil, validator collusion x2, price manipulation), full 100/1,000/10,000/100,000-actor Monte Carlo sweep completed (~7.6s wall-clock, anti-fraud gate held at every scale). NOT done: quality/improvement-factor validation against real models (no real inference exists) |
| 3 | Real P2P node | **Started** — `crates/aion-protocol` (job object, canonical serialization, nonce replay protection, job-lifecycle state machine with the no-double-settle invariant enforced in the type system) and `crates/aion-crypto` (Ed25519 identity signing, SHA-256 commit/reveal) implemented in real Rust, 26/26 `cargo test` passing, clippy-clean, fmt-clean. NOT done: `crates/aion-p2p` (libp2p integration), `crates/aion-node` (hardware detection, resource caps), actual P2P networking |
| 4 | AI execution | Not started (no GPU/runtime in this dev container; needs a real environment) |
| 5 | Verification | Design complete (`docs/VERIFICATION.md`, now including the ADR-0007 outcome-publication requirement); `crates/aion-verifier` not implemented |
| 6 | Storage | Not started |
| 7 | Marketplace | Not started |
| 8 | Web dashboard | Not started |
| 9 | Desktop node | Not started |
| 10 | Testnet settlement | Not started — blocked on operator-provided testnet RPC/funded deployer key (credentials stop-condition); design for the reward-settlement trust model is now specified (ADR-0007) so implementation can start once this phase is reached |
| 11 | ZKML experiment | Not started |
| 12 | Economic testnet | Not started |
| 13 | Security review | Not started — requires external audit, cannot be self-certified by this agent |

## Milestone 2 changes (this update)

- Closed the top open architectural risk from milestone 1 (off-chain PoIG trust in on-chain settlement) via ADR-0007, with ripple-through updates to `docs/security/THREAT-MODEL.md`, `docs/VERIFICATION.md` (new outcome-publication requirement), and `docs/POIG-SPEC.md`.
- Extended the Phase 2 simulator: `quorum.py` (commit/reveal + weighted quorum, including an honestly-documented majority-collusion residual risk) and `routing.py` (deterministic Routing Score, price-manipulation resistance proof).
- Ran the full 100→100,000-actor Monte Carlo sweep specified in the orchestration brief (previously deferred as out of scope).
- Corrected the milestone-1 capability matrix: Rust/Cargo, Node.js/npm, and Docker are all real in this environment (not previously checked) — used this milestone to actually implement `crates/aion-protocol` and `crates/aion-crypto` in Rust rather than leaving them as README stubs, with dependencies fetched live from crates.io (ed25519-dalek, sha2, thiserror) and pinned via `Cargo.lock`.

## Next highest-priority task (per COMPLETION RULE)

Begin `crates/aion-node` and `crates/aion-p2p` (Phase 3) — node identity (composing `aion-crypto`'s Ed25519 identity with hardware auto-detection) and a minimal rust-libp2p integration (peer discovery via Kademlia+Identify, one GossipSub topic for job announcements), enough to get two local processes discovering each other. This is the natural next real-code milestone now that the protocol/crypto foundation compiles and is tested.

Second-priority: extend `docs/security/ECONOMIC-ATTACKS.md`'s remaining "No" rows (benchmark leakage, treasury drain modeling, stake concentration) with real simulation coverage where feasible in `simulations/poig_sim`.

## Definition of MVP (unchanged from brief, restated for tracking)

Three machines discover each other; a requester creates a signed job; a worker executes it; another node verifies it; the model artifact is retrievable by content address; all messages are authenticated; a simulated/testnet reward settles; the dashboard shows real telemetry; bad results can be challenged; a job cannot be paid twice. Progress so far: signed jobs and the no-double-settle invariant now exist as real, tested Rust code (`crates/aion-protocol`); identity signing exists (`crates/aion-crypto`). Still missing: actual P2P discovery, actual job execution, actual verification, actual settlement, and the dashboard.
