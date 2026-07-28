# AION Threat Model

Status: Phase 1. Structured per assets / actors / trust boundaries / entry points / threats / mitigations / residual risk, as required by the orchestration brief. This is a design-time threat model — no penetration testing, audit, or red-team exercise has been performed against real running infrastructure, because no infrastructure beyond the Phase 2 simulator exists yet. Do not read any claim below as "tested."

## Assets

- Escrowed job payments and staked collateral (once contracts exist, Phase 10+).
- Model weights and provenance/royalty records.
- Requester input data (privacy-classified per job, see brief's Privacy section — not yet implemented).
- Reputation records (economic value: routing priority, reduced verification overhead).
- Node operator wallet keys and P2P identity keys.
- Protocol treasury.
- Benchmark answer keys / hidden test sets (leakage devalues the whole verification layer).

## Actors

- Honest requester, honest worker (compute/storage/validator), honest model creator.
- Malicious requester (fake demand, wash activity, benchmark-answer extraction attempts).
- Malicious worker (fabricated results, capability misrepresentation, model theft).
- Colluding validator set.
- Sybil operator (many fake identities from one actor).
- External attacker with no legitimate network role (infrastructure DoS, key theft, smart-contract exploitation).

## Trust boundaries

1. **Requester ↔ network**: a signed job is trusted only as far as its signature and funded escrow — see `docs/PROTOCOL.md`.
2. **Worker ↔ verifier**: a worker's claimed result is untrusted until the job's declared `verificationPolicy` tier resolves it — see `docs/VERIFICATION.md`.
3. **Validator ↔ PoIG scoring**: a validator's vote is one signal among the applicable tier's quorum, never unilaterally authoritative above L1.
4. **Off-chain PoIG computation ↔ on-chain settlement**: the contract layer must not trust an off-chain reward number without a verifiable derivation path (this boundary is currently unimplemented — flagged as an open risk below).
5. **Node operator ↔ host hardware**: the node process runs with operator-configured resource caps; it must not exceed them regardless of job pressure (Node Safety requirement).

## Entry points

P2P gossip messages, job submission API, model manifest publication, benchmark submission, smart-contract public functions (escrow deposit/release, staking, dispute filing), node registration/capability advertisement, web/desktop UI inputs.

## Threats, mapped to attack-tree categories (see below), with mitigations and residual risk

| Threat | Mitigation (spec'd) | Residual risk |
|---|---|---|
| Arithmetic-only mining (compute occurred, no useful work) | `UsefulWorkScore` gate requires signed escrow-backed job or network benchmark provenance (`POIG-SPEC.md`) | Gate logic itself must be implemented correctly and audited before any real value is at stake; not yet audited |
| Fake job / self-dealing reward farming | Escrow funding requirement + `DemandFactor` cap + requester-reputation gating (`ECONOMIC-MODEL.md`) | Cap parameters are simulation-tunable and could be set wrong initially; requires ongoing calibration against real demand data |
| Sybil identity farming | Bonding cost for high-trust roles, max verification rate + reduced routing weight for new identities, reputation non-transferability, decay | Sybil resistance without KYC is inherently probabilistic, not absolute; graph-analysis defense is planned, not yet implemented |
| Validator collusion | Commit/reveal for validator votes; quorum weighting; reputation-floor verification rate that never reaches zero | Small validator sets remain collusion-susceptible regardless of protocol design (see Bittensor competitor analysis) — mitigated but not eliminated by design alone |
| Benchmark leakage / overfitting | Hidden + rotating benchmark suites, multiple independent metrics, statistical-significance gating in Proof of Improvement | A sufficiently patient adversary with many submission attempts can still partially infer a static hidden set over time; rotation cadence must stay ahead of this |
| Replay of a signed job/result | Per-requester monotonic nonce (`PROTOCOL.md`) | None identified beyond correct implementation |
| Double settlement of one job | Terminal-state invariant (`SETTLED`/`SLASHED` reject further settlement instructions) enforced at the contract layer | Not yet implemented (no contracts exist yet); this is a Phase 10 must-have, tracked here so it isn't dropped |
| Model weight theft / unauthorized redistribution | Content-addressed manifests + license metadata; does not itself prevent copying (content addressing does not equal DRM) | Fundamentally a P2P/open-network limitation — mitigations are economic (royalty attribution) and social (license terms), not technically airtight; document this honestly rather than overclaiming protection |
| Node resource abuse (silent overconsumption) | Node Safety: mandatory operator-set resource caps, idle-only mode, working hours (`ARCHITECTURE.md`) | Enforcement quality depends on `aion-node` implementation correctness — not yet implemented |
| Off-chain PoIG number trusted blindly on-chain | **Resolved at the design level in `docs/adr/0007-reward-settlement-trust-model.md`** (milestone 2): optimistic Merkle-batch reward commitments, bonded attestation committee, permissionless on-chain-checkable fraud proofs during a challenge window, pull-based claiming. Requires every verification tier and the reputation store to publish hash-referenceable outcomes (new cross-cutting requirement, see `docs/VERIFICATION.md`). | Medium — design resolved; implementation (Phase 10 `RewardDistributor` contract) and economic parameter tuning (challenge window length, committee bonding) still pending |
| Smart contract exploitation (once contracts exist) | OpenZeppelin primitives, reentrancy guards, checks-effects-interactions, pull payments, timelock + multisig admin, mandatory Slither/Echidna/fuzz/invariant testing before any testnet deployment | Entirely unaddressed today since no contracts exist yet; tracked for Phase 10 |
| Key theft (operator wallet / P2P identity) | Identity separation (P2P vs wallet vs reputation, `ARCHITECTURE.md`); smart-account session-key scoping (ADR-0001) limits blast radius of a stolen session key | Root wallet key compromise remains a fundamental risk any account-abstraction session-key model only partially contains |
| Infrastructure DoS (gossip flood, storage exhaustion) | libp2p peer scoring, connection/resource limits (ADR-0002); rate limiting at API layer | Not yet load-tested; chaos/load testing is future-phase work |

## Attack tree

See `docs/security/ECONOMIC-ATTACKS.md` for the economic-attack-specific tree (self-dealing, Sybil farming, validator bribery, price manipulation, treasury drain, etc.). Technical attack tree:

```
STEAL FUNDS
├── exploit contract           -> mitigated by: OZ primitives, fuzz/invariant testing, timelock (Phase 10, not yet built)
├── compromise admin           -> mitigated by: multisig + timelock, least-privilege roles (Phase 10, not yet built)
├── replay transaction         -> mitigated by: nonce (PROTOCOL.md, spec'd)
└── manipulate settlement      -> mitigated by: terminal-state invariant (spec'd, not yet built)

FAKE COMPUTE
├── fake job                   -> mitigated by: UsefulWorkScore gate (spec'd, simulated in Phase 2)
├── fake model                 -> mitigated by: content-addressed manifest + provenance DAG (spec'd)
├── copy worker (free-ride another's result) -> mitigated by: commit/reveal (spec'd)
├── validator collusion        -> mitigated by: quorum weighting + verification floor (spec'd)
└── arithmetic-only mining     -> mitigated by: UsefulWorkScore gate (spec'd, simulated in Phase 2)

CORRUPT AI
├── dataset poisoning          -> mitigated by: dataset governance (license/hash/provenance metadata), not yet enforced in tooling
├── model poisoning            -> mitigated by: Proof of Improvement statistical gating (spec'd, not simulated against real models)
├── benchmark attack           -> mitigated by: hidden/rotating benchmarks (spec'd)
└── malicious adapter          -> mitigated by: provenance DAG + verification tiers on evaluate jobs (spec'd)

TAKE DOWN NETWORK
├── Sybil                      -> mitigated by: bonding + verification-rate + reputation decay (spec'd, simulated in Phase 2)
├── DDoS                       -> not yet addressed (no live infrastructure)
├── gossip spam                -> mitigated by: libp2p peer scoring / resource manager (spec'd, not yet built)
├── storage exhaustion         -> not yet addressed (Phase 6)
└── validator griefing         -> mitigated by: quorum design tolerating some nonresponsive validators (spec'd, not load-tested)
```

## Residual risk summary

Milestone 1's single highest-priority unresolved item — **off-chain PoIG trust in on-chain settlement** — is now resolved at the design level (`docs/adr/0007-reward-settlement-trust-model.md`, milestone 2). The next-highest-priority residual risks, in order, are: (1) smart contract exploitation (entirely unaddressed — no contracts exist yet, Phase 10), (2) Sybil resistance beyond the two scenarios actually simulated so far (validator collusion, price manipulation — targeted in milestone 2's simulator extension), and (3) infrastructure DoS/load testing (no live infrastructure to test yet).
