# AION Verification Specification

Status: Phase 1 specification. Concrete tiering decision recorded in `docs/adr/0003-verification-tiering.md`; method-level tradeoffs in `docs/research/AI-VERIFICATION-MATRIX.md`. The Phase 2 simulator models verification tiers abstractly (a confidence value per tier) rather than implementing real replicated execution or ZK proving — no GPUs or model weights exist in this development container (see `docs/research/CAPABILITY-MATRIX.md`).

## Levels

| Level | Name | Reward eligibility | Typical use |
|---|---|---|---|
| L0 | None | No financial reward | Local development jobs only |
| L1 | Commitment/hash validation | Low-confidence reward only | Cheap sanity check, paired with spot L2 checks |
| L2 | Random replicated execution | Standard reward | General inference/training jobs, numeric-tolerance comparison across N replicas |
| L3 | Deterministic/reproducible execution | Standard-to-high reward | High-value jobs where requester accepts a restricted hardware pool |
| L4 | Challenge protocol (optimistic commit + dispute window + bisection) | Standard reward, default tier | Default for paid inference/training where L3's hardware restriction is unacceptable |
| L5 | ZK/cryptographic proof (ZKML) | High reward, narrow scope | Small models / sub-components only (Phase 11 proof-of-concept) |
| L6 | Multi-validator quorum | Highest reward tier, layered on L2/L4 | High-value jobs, combined with a lower-tier mechanism, not standalone |

## Non-negotiable rule

Reputation never fully eliminates verification. Even an established, high-reputation worker is subject to a nonzero random-check rate — the check rate decreases with proven track record but never reaches zero. This is enforced in `crates/aion-verifier` (target) and modeled in `simulations/poig_sim` (current) via a floor parameter on the verification-rate function.

## Distinguishing failure classes before any penalty

Verification failure is not automatically treated as fraud. See `docs/protocol/SLASHING-SPEC.md` for the full failure taxonomy (malicious result vs. runtime error vs. hardware failure vs. network timeout vs. numerical nondeterminism vs. model incompatibility vs. software bug) and which failure classes are slashable versus merely non-rewarded.

## Commit/reveal

Used wherever a party's answer could otherwise be copied from another party's already-visible answer: benchmark answers, validator decisions, worker bids, and challenge responses all follow a commit-then-reveal pattern (commit = hash of answer + salt, reveal = answer + salt, checked against the earlier commit) so that, e.g., a validator cannot simply copy another validator's on-chain-visible score before submitting their own.

## Outcome publication requirement (added by ADR-0007)

Every verification tier's outcome (pass/fail, and the confidence/quorum detail behind it) must be published somewhere hash-referenceable — a commitment, a dispute-window result, an aggregated quorum signature set, or a ZK proof — never held only in private coordinator memory. This is what makes the reward-settlement fraud-proof mechanism in `docs/adr/0007-reward-settlement-trust-model.md` possible: a challenger recomputing a disputed reward must be able to independently confirm which verification outcome actually backed it, without trusting the coordinator's say-so. `crates/aion-verifier` must satisfy this from its first implementation (Phase 5), not retrofit it later.

## Adaptive quorum sizing

Quorum/verification-rate scales with both job value and worker track record:
- New/unproven worker → high verification rate regardless of job value.
- Established worker, low-value job → lower (but nonzero) verification rate.
- Any worker, high-value job → higher quorum (L6) regardless of individual track record.

Exact rate curves are simulation-tunable parameters (`simulations/poig_sim/poig_sim/verification.py`), not fixed constants in this document, since the right curve depends on measured fraud rates this repo does not have real data for yet.
