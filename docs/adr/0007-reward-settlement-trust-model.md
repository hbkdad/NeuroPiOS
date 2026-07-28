# ADR 0007: Reward Settlement Trust Model — Optimistic Merkle Batch Commitments with Fraud Proofs

## Status
Accepted (design-level; not yet implemented — no contracts exist until Phase 10). This ADR closes the top open risk flagged in `docs/security/THREAT-MODEL.md` milestone 1: "off-chain PoIG number trusted blindly on-chain."

## Context

PoIG (`docs/POIG-SPEC.md`) is deliberately computed off-chain (or in coordinator-internal computation) — re-litigated and reaffirmed in ADR-0003/`docs/ARCHITECTURE.md`'s "PoIG is not blockchain consensus" section, because re-running the full reward pipeline (verification-tier outcomes, reputation lookups, demand aggregation) inside EVM gas limits is neither necessary nor economical. But that leaves a real, previously-unresolved gap: what stops the coordinator (or whoever computes rewards) from simply submitting a wrong number to the settlement contract, and what stops a `JobEscrow`/`RewardDistributor` contract from paying it out on blind trust?

This is a different, and easier, problem than AI-execution verification (L0–L6, ADR-0003). The PoIG formula itself is cheap, bounded, deterministic arithmetic — `BaseValue × sevenBoundedFactors`. The hard part was never re-running the arithmetic; it's making sure every *input* to that arithmetic (was escrow actually funded, did verification actually pass at the claimed tier, what is this worker's actual reputation) is itself something a third party can check without trusting the coordinator's word for it.

## Decision

Adopt an **optimistic batch-commitment model**, structurally the same optimistic-dispute pattern already chosen for AI-execution verification at L4 (ADR-0003) — cheap by default, expensive only when actually disputed:

1. **Every input to the reward formula must be independently checkable, not just held in coordinator memory.** This is a new cross-cutting requirement on the rest of the protocol, not just a settlement-layer add-on:
   - `UsefulWorkScore`'s escrow-funded check: verifiable directly from the `JobEscrow` contract's own on-chain state/events — this one was already inherently checkable, no change needed.
   - `UsefulWorkScore`'s signature/nonce check: verifiable from the signed job object itself (already content-addressable per `docs/PROTOCOL.md`).
   - `VerificationConfidence`: the outcome of whichever tier (L1–L6) actually ran must be published somewhere hash-referenceable (a commitment, a dispute-window's pass/fail outcome, a quorum's aggregated signatures, a ZK proof) — this is a **new requirement added to `docs/VERIFICATION.md`** in this milestone: no verification tier's outcome may live only in private coordinator state.
   - `ReputationFactor`: derived from a reputation store whose update history is itself an append-only, hash-chained (or on-chain-anchored) log, so a claimed reputation value at epoch N is reproducible from public history, not assertable at will.

2. **Coordinator publishes reward batches, not individual settlements.** At the end of each epoch, the off-chain reward aggregator (`services/coordinator`, future) computes rewards for every job settled that epoch, and publishes: (a) a Merkle root over `(workerId, epochId, rewardAmount)` leaves, submitted on-chain to a `RewardDistributor` contract; (b) the full leaf set plus every input referenced in step 1, to public content-addressed storage (a CID), so anyone can independently recompute the batch offline.

3. **A bonded attestation step, not blind trust.** A small, initially-permissioned attestation committee (multisig, expanding toward a broader validator set as the network matures — an explicit centralization-to-decentralization glide path, not a permanent design) must co-sign a batch root, and must have independently recomputed it before signing (not just relayed the coordinator's claim). The committee posts a bond that is slashable on a proven fraud proof (item 4).

4. **Challenge window with cheap, on-chain-checkable fraud proofs.** After a batch root is posted, a fixed challenge window (parameter, not fixed in this ADR — simulation/economic-tunable, mirroring L4's dispute window) is open during which anyone can submit a fraud proof for a specific leaf: recompute that one worker's reward from the public inputs (step 1) and show it disagrees with the committed amount. Because the PoIG formula itself is cheap bounded arithmetic (not ML execution), **this recomputation is genuinely cheap to verify on-chain directly** — this is the property that makes the optimistic model economical here, unlike AI-execution disputes which need bisection because re-execution is expensive. A successful fraud proof: rejects/corrects the disputed leaf, slashes the attestation committee's bond (split per `docs/protocol/SLASHING-SPEC.md`'s slashed-funds policy), and does not require the whole batch to be redone.

5. **Claim, don't push.** Workers claim their reward from `RewardDistributor` after the challenge window closes, by submitting a standard Merkle inclusion proof against the committed root — a pull-payment pattern (per the Smart Contract Security Requirements in `AGENTS.md`/orchestration brief), not the contract pushing funds proactively.

## Reasoning

- Reuses the same conceptual pattern (commit → challenge window → cheap arbitration) already adopted for L4 AI-execution verification, rather than inventing a second, different trust mechanism — one fewer novel piece of infrastructure to build and audit.
- Correctly identifies that the PoIG *arithmetic* was never the hard part — the hard part is input integrity — and solves that by pushing a publish-everything-checkable requirement down into `docs/VERIFICATION.md` and the reputation store, rather than trying to make the settlement contract itself smarter.
- The bonded-attestation-committee step is an honestly-labeled interim centralization: fully permissionless fraud-proof submission (item 4) is the actual trust anchor; the committee exists so batches don't sit unposted waiting for a fully decentralized validator set to exist, and it is explicitly bonded/slashable rather than unaccountable.
- Pull-based claiming avoids a whole class of push-payment reentrancy/failure-handling bugs at the settlement layer.

## Consequences

- `docs/VERIFICATION.md` gains a new cross-cutting requirement (every tier must publish a hash-referenceable outcome) — this affects `crates/aion-verifier` design from Phase 5 onward, not just the Phase 10 contracts.
- `crates/aion-reputation`'s store must be append-only/hash-chained (or periodically anchored) rather than a freely-mutable table, once implemented beyond the Phase 2 Python reference.
- New contract surface for Phase 10: `RewardDistributor` needs Merkle-proof claim verification and a fraud-proof-checking function that re-runs the (cheap) PoIG arithmetic — both must be fuzz/invariant tested per `AGENTS.md`'s smart-contract testing requirements before any testnet deployment.
- The attestation committee's bonding/rotation/slashing parameters, and the challenge-window length, are economic-simulation-tunable parameters for the `aion-economics` skill/agent to derive, not fixed by this ADR.
- This ADR does not eliminate all trust — it eliminates *blind* trust, replacing it with a bonded, challengeable, permissionlessly-auditable commitment. Full trustlessness would require either the attestation committee's elimination (broader validator set, later) or moving the entire reward computation on-chain (rejected in `docs/ARCHITECTURE.md`'s "PoIG is not blockchain consensus" — the cost/complexity is not justified when the cheaper fraud-proof path is available).

## Status of the threat-model risk this closes

`docs/security/THREAT-MODEL.md`'s "off-chain PoIG number trusted blindly on-chain" row is updated from **Residual risk: High until resolved** to **Residual risk: Medium — design resolved (this ADR), implementation and economic-parameter tuning still pending** (Phase 10 work, not yet done).
