# PoIG — Proof of Intelligence Gain: Specification

Status: Phase 1 specification, validated so far only by the Phase 2 simulator (`simulations/poig_sim`) on synthetic scenarios. Weights and thresholds below are **starting points for simulation, not tuned production values**.

## What PoIG is (and is not)

PoIG is an **economic contribution-scoring function**, evaluated off-chain (or in coordinator-internal computation), that converts verified, requester-anchored work into a reward amount. It is explicitly **not** a blockchain consensus mechanism — see `docs/ARCHITECTURE.md`, "PoIG is not blockchain consensus."

## The core rule this spec exists to enforce

> Never equate "computation occurred" with "useful AI computation occurred." A valid reward must be linked to a legitimate network job, committed model/input state, verifiable execution, and measurable outcome.

Every term in the reward function below exists to make this rule mechanically true rather than aspirational.

## Reward function (initial form, to be simulated and re-derived, not assumed correct)

```
Reward =
    BaseValue
  × UsefulWorkScore
  × VerificationConfidence
  × DemandFactor
  × QualityFactor
  × ImprovementFactor
  × ReputationFactor
  × EfficiencyFactor
```

All factors are bounded to `[0, 1]` except `BaseValue` (a job-type-specific price anchor, denominated in the Stage-appropriate ledger unit per `docs/adr/0004-token-development-staging.md`) and `DemandFactor` (bounded `[0, cap]`, capped to prevent unbounded amplification from a thin/manipulable demand signal).

### UsefulWorkScore — the anti-farming gate
`UsefulWorkScore = 0` unless the work traces back to one of:
- a **signed, escrow-backed job** matching the canonical job object in `docs/PROTOCOL.md` (requester signature + funded escrow, both checked before this factor can be nonzero), or
- a **network-issued benchmark** (per the Benchmark object in `docs/PROTOCOL.md`) that the network itself requested, with its own anti-leakage policy.

A worker cannot generate arbitrary computation and self-reward: there is no path to nonzero `UsefulWorkScore` without an external, signed demand signal. This is the mechanical enforcement of the brief's "Useful Work Score" requirement (escrow-backed job / network-generated benchmark / signed commitment / job fee / rate limiting / minimum requester reputation / challenge system — implemented here as escrow + signature + benchmark provenance, with rate limiting and requester-reputation gating specified as additional multipliers below rather than a separate gate, to keep the "is this real demand" check in one place).

### VerificationConfidence
Derived directly from the verification tier actually applied (see `docs/adr/0003-verification-tiering.md`) and its outcome — e.g. L1 alone caps this factor well below 1.0; L4 with no dispute raised within the challenge window raises it; L6 quorum agreement raises it further. Exact mapping is a simulation output, not fixed here.

### DemandFactor
Reflects real signed demand for this job/model/benchmark (e.g., escrow amount relative to `BaseValue`, or benchmark participation rate), capped to prevent a small number of self-dealing "requesters" (see `docs/security/ECONOMIC-ATTACKS.md`, self-dealing jobs) from unboundedly inflating reward via wash demand.

### QualityFactor / ImprovementFactor
For `evaluate`/`fine_tune`/`train`/`compress` job types: derived from **Proof of Improvement** (see below) — a candidate is scored against hidden/rotating benchmark suites with statistical-significance testing, never against a benchmark the candidate's own training process could have seen (anti-overfitting).

### ReputationFactor
Multi-dimensional reputation (see `docs/security/THREAT-MODEL.md` and the brief's Reputation System section) collapsed into a single bounded multiplier for this formula; the underlying reputation store itself remains multi-dimensional (completion reliability, verification reliability, latency, hardware honesty, uptime, dispute history, per-job-type expertise) and is never purchasable or transferable.

### EfficiencyFactor
For workloads where efficiency is the measured contribution (Proof of Efficiency, below), a bounded ratio — not the literal `Quality × Throughput / (Latency × Memory × EnergyCost)` formula suggested as an example in the orchestration brief, which this spec explicitly does not adopt until Phase 2 simulation validates a concrete efficiency metric (the brief itself says not to use that exact equation until simulated).

## Proof of Improvement (model-improvement competitions)

```
baseline model -> candidate contribution -> hidden benchmark suite -> statistical evaluation
  -> independent validation -> improvement confidence interval -> accept/reject -> reward
```

Mandatory anti-overfitting measures: hidden test sets, rotating benchmark rotation schedule, out-of-distribution evaluation slices, multiple independent metrics (not a single scalar), and periodic adversarial test injection. A candidate is only rewarded if the confidence interval on its improvement excludes zero at a pre-registered significance threshold — this threshold is a simulation-tunable parameter, not fixed in this document.

## Proof of Efficiency

Rewards measurable reductions in latency, memory, energy, or model size **at equivalent quality**, or measurable quality gains at equivalent cost. The exact scoring formula is deferred to simulation (`simulations/poig_sim`) rather than assumed — see EfficiencyFactor above.

## Model royalties (contribution DAG attribution)

Per completed job, the fee is split among compute provider, model creator(s) (walking the model's provenance DAG from `docs/PROTOCOL.md`), validator(s), dataset contributor(s), protocol treasury, and an optional burn/sink, per creator-configured revenue-share policy on each `ModelManifest`. DAG attribution is computed off-chain; only the final aggregated per-recipient distribution is settled on-chain, to keep gas cost bounded regardless of DAG depth (see `docs/adr/0005-monorepo-structure.md` note and `docs/PROTOCOL.md` — Model provenance DAG).

## Agent economy constraint

Autonomous agents requesting paid work (per the brief's Agent Economy section) operate under an owner-defined `spendingAuthorization`: per-job limit, daily limit, an explicit allow-list of service types, and owner-revocable at any time. An agent is never issued an unlimited-spend wallet — this is enforced at the job-signing layer (a job signed on an agent's behalf must reference a currently-valid, unexpired `spendingAuthorization` that covers it, or the job is invalid before it ever reaches a worker).

## What Phase 2 actually validates (and what it doesn't yet)

`simulations/poig_sim` implements `UsefulWorkScore`'s escrow/signature gate, a simplified `VerificationConfidence` (from a configurable per-tier confidence table), `ReputationFactor` with decay, and runs attack scenarios (Sybil job self-dealing, fake-benchmark reward farming) to confirm the gate actually blocks the naive versions of those attacks. It does **not** yet validate `QualityFactor`/`ImprovementFactor` against a real hidden benchmark suite (no real models are executed in this container — see `docs/research/CAPABILITY-MATRIX.md`), and does not yet run at the 100/1,000/10,000/100,000-node scale the brief specifies for full economic simulation — that is explicitly future work, not claimed as done here.
