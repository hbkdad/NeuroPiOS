# AION Roadmap

Phases as defined by the orchestration brief. Status reflects reality as of milestone 2 (2026-07-28) — see PR #1 and its follow-up commits for the authoritative "what actually happened" record.

| Phase | Name | Status |
|---|---|---|
| 0 | Research | **Done** — capability matrix (incl. real toolchain inventory: Rust/Cargo, Node, Docker all confirmed available), 5 competitor analyses, verification matrix, research log |
| 1 | Specification | **Done** — ARCHITECTURE, PROTOCOL, POIG-SPEC, ECONOMIC-MODEL, VERIFICATION, SLASHING-SPEC, THREAT-MODEL, ECONOMIC-ATTACKS, 7 ADRs (added ADR-0007: reward settlement trust model, milestone 2) |
| 2 | Local simulator (no blockchain) | **Substantially done** — `simulations/poig_sim`: 22 pytest tests covering 7 attack scenarios (self-dealing, Sybil, validator collusion x2, price manipulation, benchmark leakage, stake concentration), full 100/1,000/10,000/100,000-actor Monte Carlo sweep completed (~7.6s wall-clock, anti-fraud gate held at every scale). NOT done: quality/improvement-factor validation against real models (no real inference exists) |
| 3 | Real P2P node | **Substantially done** — `crates/aion-protocol` (job object, nonce replay protection, no-double-settle state machine), `crates/aion-crypto` (Ed25519 identity, commit/reveal, cross-crate identity export), `crates/aion-node` (real hardware detection via `sysinfo`, Node Safety resource caps defaulting to locked-down, composable roles, owns and drives a real `aion-p2p` swarm built from its own identity), and `crates/aion-p2p` (real GossipSub+Identify+Kademlia+ConnectionLimits swarm over TCP+Noise+Yamux, peer scoring and connection-limit enforcement both active and tested, working two-peer gossip and Kademlia FIND_NODE integration tests) — all real Rust, 56/56 `cargo test` passing, clippy-clean, fmt-clean. NOT done: AutoNAT/relay (NAT traversal), 3+ node indirect Kademlia discovery (attempted, root-caused, documented as a known issue in `crates/aion-p2p/README.md`) |
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

## Milestone 4 changes (this update)

- `crates/aion-crypto`: added `Identity::to_bytes()` for deliberate cross-crate identity export.
- `crates/aion-p2p`: added Kademlia (`kad::Behaviour`) on a custom protocol name, `keypair_from_identity` to bridge an `aion_crypto::Identity` into a `libp2p::identity::Keypair` (verified byte-for-byte, not just deterministically), and `feed_identify_into_kademlia` for the standard Identify→Kademlia integration pattern. New `tests/kademlia_discovery.rs` proves a real FIND_NODE round-trip over the wire.
- `crates/aion-node`: `Node::build_swarm()` now actually builds a P2P swarm from the node's own identity. New `tests/node_gossip.rs` proves two `Node`s gossip using their own identities end-to-end, with the swarm's PeerID checked against the node's `aion_crypto::Identity` public key.
- Total test count: 62 (14 Python + 48 Rust), all passing, clippy-clean, fmt-clean.

## Milestone 5 changes (this update)

- `crates/aion-p2p`: activated real GossipSub peer scoring (`with_peer_score`) with an explicit, deliberately-tuned `TopicScoreParams` for the job-announcements topic (rewards mesh tenure and first-delivery, penalizes under-delivery and — heavily, quadratically — invalid messages), satisfying `docs/adr/0002-p2p-stack.md`'s "every topic gets its own anti-spam policy" requirement instead of leaving library defaults untouched. 4 new tests validating the params against gossipsub's own validation rules and confirming they're genuinely non-default.
- Attempted a 3-node relayed Kademlia discovery test (the actual "three machines discover each other" proof). It does not currently work — root-caused as far as reasonable within this session (the responder computes the correct closer-peer list, confirmed via `Event::InboundRequest`, but the requester's query result and routing table never reflect it, even under retry) and documented as a known issue in `crates/aion-p2p/README.md` with the diagnostic trail preserved, rather than either faked as passing or silently dropped.
- Total test count: 68 (14 Python + 54 Rust), all passing, clippy-clean, fmt-clean.

## Milestone 6 changes (this update)

- `crates/aion-p2p`: added real, enforced connection limits (`connection_limits::Behaviour`) with explicit defaults covering pending/established/per-peer/total caps (eclipse-attack-aware: incoming capped independently from other limits). `tests/connection_limits.rs` proves real enforcement — a peer configured with zero allowed inbound connections actually rejects a live dial attempt — with a control test confirming the same dial succeeds under normal limits. 2 new tests.
- `simulations/poig_sim`: added `benchmark.py` (hidden/rotating benchmark model + per-identity rate limiting) and `tests/test_benchmark_leakage.py`, giving `docs/security/ECONOMIC-ATTACKS.md`'s "Benchmark leakage" row real simulation coverage. A real aggregate-score-only bit-flip probing attack fully recovers a 30-item hidden answer key when unmitigated (100% accuracy across all tested seeds, confirming the attack is genuinely effective) but is held to chance-level accuracy when rate limiting + rotation are active. 5 new tests.
- Total test count: 75 (19 Python + 56 Rust), all passing, clippy-clean, fmt-clean.

## Milestone 7 changes (this update)

- `simulations/poig_sim`: added `tests/test_stake_concentration.py`, giving `docs/security/ECONOMIC-ATTACKS.md`'s "Stake concentration" row real simulation coverage by reusing `quorum.py`'s existing weighted aggregation. Isolates weight concentration from headcount: a 2-of-10 dishonest minority with concentrated stake (weight 40 vs. the honest majority's 8) dominates the quorum outcome; the identical 2-of-10 headcount split under equal weights does not; a weight sweep confirms the flip point tracks the whales' aggregate weight share crossing 50%. Confirms this remains a genuinely open design gap, honestly consistent with the mitigation column's existing "not fully resolved" framing. 3 new tests.
- Total test count: 78 (22 Python + 56 Rust), all passing, clippy-clean, fmt-clean.

## Next highest-priority task (per COMPLETION RULE)

Resolve the 3-node relayed Kademlia discovery issue documented in `crates/aion-p2p/README.md` — this is the concrete remaining gap toward the Phase 3 MVP exit criterion "three machines discover each other," and the diagnostic trail already narrows it to response handling between the responder sending a correct closer-peer list and the requester's query algorithm recording it. Needs `libp2p-kad` protocol-level tracing to pin down definitively.

Second-priority: extend `docs/security/ECONOMIC-ATTACKS.md`'s remaining "No" rows (treasury drain modeling, fake demand/wash activity, front-running, griefing) with real simulation coverage where feasible in `simulations/poig_sim`.

## Definition of MVP (unchanged from brief, restated for tracking)

Three machines discover each other; a requester creates a signed job; a worker executes it; another node verifies it; the model artifact is retrievable by content address; all messages are authenticated; a simulated/testnet reward settles; the dashboard shows real telemetry; bad results can be challenged; a job cannot be paid twice. Progress so far: signed jobs and the no-double-settle invariant exist as real, tested Rust code (`crates/aion-protocol`); identity signing exists (`crates/aion-crypto`); two real nodes can discover each other via manual dial and gossip authenticated messages with active anti-spam scoring and connection-limit enforcement (`crates/aion-node` + `crates/aion-p2p`); a Kademlia DHT protocol round-trip works. Still missing: 3+ node indirect discovery (blocked, documented), actual job execution, actual verification, actual settlement, and the dashboard.
