# Slashing Specification

Status: Phase 1 specification. Slashing must be conservative — this document exists specifically to prevent honest workers from being punished for ambiguous failures, per the orchestration brief.

## Failure taxonomy

| Failure class | Description | Slashable? | Reward outcome |
|---|---|---|---|
| Malicious result | Result diverges from verified-correct output in a way inconsistent with any plausible honest-execution explanation (e.g., fails a commit/reveal check, contradicts an L4 bisection outcome against the disputing party) | **Yes** | Slashed; no reward |
| Runtime error | Worker's execution environment threw an error and the worker reported it honestly (job marked failed, not silently dropped) | No | No reward, no slash — job requeued |
| Hardware failure | Node hardware failed mid-job (verifiable via node's own health telemetry / abrupt disconnect pattern) | No | No reward, no slash — job requeued; repeated pattern feeds `hardwareHonesty` reputation dimension, not a slash |
| Network timeout | Job did not complete within deadline due to network conditions, no evidence of dishonesty | No | No reward, no slash — job requeued |
| Numerical nondeterminism | Result differs from a reference execution within a declared numeric-tolerance band (see below), consistent with known floating-point non-associativity across hardware/kernel versions | No | Reward at reduced `VerificationConfidence`, no slash |
| Model incompatibility | Worker's advertised capability didn't actually match the job's `executionProfile` requirements | Depends: **slashable** if the worker knowingly misrepresented capability at registration; **not slashable** if the mismatch stems from an ambiguous/underspecified job spec | Job requeued; capability-misrepresentation cases also degrade `hardwareHonesty` reputation |
| Software bug | AION's own execution harness (not the worker) is at fault, confirmed via reproduction on a second worker with the same result deviation | No — this is a protocol/software defect, not a worker failure | No reward, no slash; treated as an incident, not a dispute |

## Numeric-tolerance band

L2/L3/L4 comparisons use a declared per-job-type tolerance (e.g., relative error bound on output logits/embeddings, or exact-match for deterministic non-floating-point tasks) rather than bit-exact equality, because bit-exact reproducibility across heterogeneous GPU hardware is not achievable for most real ML workloads (see `docs/research/competitors/gensyn.md` — this is the same problem Gensyn's Verde/REE research explicitly targets). The tolerance value is a per-`jobType` configuration, not a single global constant, and must be set conservatively (wide enough to not punish honest hardware variance, narrow enough to catch actually-wrong results) — initial values are simulation-tunable, not fixed in this document.

## Burden and process

1. A slash requires a **positive finding** from the applicable verification tier (an L4 bisection result identifying the disputed step, an L2 majority-disagreement beyond tolerance with no plausible nondeterminism explanation, or an L6 quorum majority) — never a unilateral accusation.
2. The accused worker has the ability to contest via the dispute/challenge mechanism (L4) before a slash is finalized, where the job's verification tier supports disputes.
3. Slashed funds follow a defined policy: a portion compensates the disputing/harmed party (requester and/or challenger), a portion goes to protocol treasury, per-job-type configuration — exact split is an economic-simulation output, not fixed here.
4. Ambiguous cases (evidence consistent with more than one failure class) default to the **non-slashable** classification — the burden is on the evidence to clearly establish malice, not on the worker to prove innocence.

## Core invariant

A worker is never slashed for a failure class in the "No" column above, regardless of how many times it recurs — repeated non-slashable failures degrade reputation (which affects future routing and verification rate) but never trigger a stake slash. This separation (reputation consequence vs. stake consequence) is deliberate and must be preserved in any future implementation of `crates/aion-reputation` and the on-chain `StakeManager`/`DisputeManager` contracts.
