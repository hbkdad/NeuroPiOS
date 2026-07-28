# AION Roadmap

Phases as defined by the orchestration brief. Status reflects reality as of milestone 2 (2026-07-28) — see PR #1 and its follow-up commits for the authoritative "what actually happened" record.

| Phase | Name | Status |
|---|---|---|
| 0 | Research | **Done** — capability matrix (incl. real toolchain inventory: Rust/Cargo, Node, Docker all confirmed available), 5 competitor analyses, verification matrix, research log |
| 1 | Specification | **Done** — ARCHITECTURE, PROTOCOL, POIG-SPEC, ECONOMIC-MODEL, VERIFICATION, SLASHING-SPEC, THREAT-MODEL, ECONOMIC-ATTACKS, 7 ADRs (added ADR-0007: reward settlement trust model, milestone 2) |
| 2 | Local simulator (no blockchain) | **Substantially done** — `simulations/poig_sim`: 14 pytest tests covering 5 attack scenarios (self-dealing, Sybil, validator collusion x2, price manipulation), full 100/1,000/10,000/100,000-actor Monte Carlo sweep completed (~7.6s wall-clock, anti-fraud gate held at every scale). NOT done: quality/improvement-factor validation against real models (no real inference exists) |
| 3 | Real P2P node | **Substantially started** — `crates/aion-protocol` (job object, nonce replay protection, no-double-settle state machine), `crates/aion-crypto` (Ed25519 identity, commit/reveal), `crates/aion-node` (real hardware detection via `sysinfo`, Node Safety resource caps defaulting to locked-down, composable roles), and `crates/aion-p2p` (real GossipSub+Identify swarm over TCP+Noise+Yamux, working two-peer publish/receive integration test) — all real Rust, 43/43 `cargo test` passing, clippy-clean, fmt-clean. NOT done: Kademlia peer discovery, AutoNAT/relay, peer scoring/anti-spam policy, wiring `aion-node` to actually own a running `aion-p2p` swarm |
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

## Milestone 3 changes (this update)

- `crates/aion-node`: real hardware detection (`sysinfo`), Node Safety resource caps (mandatory locked-down default, working-hours window incl. midnight wraparound), composable role flags. 13/13 tests.
- `crates/aion-p2p`: real GossipSub+Identify swarm over TCP+Noise+Yamux, with a genuine two-peer integration test proving a published message is actually received by another peer (not just that the code compiles). 4/4 tests, including one real network integration test.
- Total test count across the repo: 57 (14 Python + 43 Rust), all passing, clippy-clean, fmt-clean.

## Next highest-priority task (per COMPLETION RULE)

Wire `crates/aion-node` to actually own and drive an `aion-p2p` swarm (currently the two crates are independent), and add Kademlia-based peer discovery to `aion-p2p` so peers don't need a manually-known dial address — this closes the gap toward the Phase 3 exit criterion "three machines discover each other" from the MVP definition below.

Second-priority: extend `docs/security/ECONOMIC-ATTACKS.md`'s remaining "No" rows (benchmark leakage, treasury drain modeling, stake concentration) with real simulation coverage where feasible in `simulations/poig_sim`.

## Definition of MVP (unchanged from brief, restated for tracking)

Three machines discover each other; a requester creates a signed job; a worker executes it; another node verifies it; the model artifact is retrievable by content address; all messages are authenticated; a simulated/testnet reward settles; the dashboard shows real telemetry; bad results can be challenged; a job cannot be paid twice. Progress so far: signed jobs and the no-double-settle invariant now exist as real, tested Rust code (`crates/aion-protocol`); identity signing exists (`crates/aion-crypto`). Still missing: actual P2P discovery, actual job execution, actual verification, actual settlement, and the dashboard.
