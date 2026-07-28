# ADR 0003: Tiered Verification (L0–L6) Instead of One Universal Mechanism

## Status
Accepted

## Context
No single verification method covers every AION workload at acceptable cost (see `docs/research/AI-VERIFICATION-MATRIX.md`). Competing networks each picked one mechanism and inherited its blind spot: Akash/Render verify nothing about output correctness; Bittensor relies entirely on opaque validator opinion; Allora claims zkML coverage that current proving-cost research does not support at its claimed inference volume; Gensyn's optimistic dispute approach (Verde/REE) is promising but unproven at mainnet scale.

## Decision
Adopt a **7-level verification tiering** (L0–L6, defined in `docs/VERIFICATION.md`), selected per job by the requester's `verificationPolicy` field and the job's declared value:

- L0 — no reward, local/dev only.
- L1 — commitment/hash validation only.
- L2 — random replicated execution (N-of-M, numeric-tolerance comparison).
- L3 — deterministic/reproducible execution (pinned hardware/kernel pool).
- L4 — optimistic commit + dispute window + bisection (Gensyn-Verde-style challenge protocol) — the default for standard paid inference/training jobs.
- L5 — ZKML proof, scoped to small models/sub-components only (Phase 11 proof-of-concept), per current proving-cost limits.
- L6 — multi-validator weighted quorum for high-value jobs, layered on top of L2/L4, not a replacement for them.

Verification quorum size and rate scale adaptively with worker reputation and job value (new/unproven workers get higher random-check rates; established workers get lower rates but never zero — reputation cannot fully eliminate verification, per the orchestration brief).

## Reasoning
- Matches method to workload shape rather than forcing one mechanism (the single biggest lesson from the competitor research: Render's checksum-based render verification doesn't generalize to open-ended inference; Allora's ground-truth accuracy scoring works for prediction-style Topics but not all task types).
- L4 as the default gives most paid jobs a cheap common-case cost (no dispute = no extra compute) while still providing a real economic deterrent (bisection arbitration cost) against dishonest results — directly addressing the brief's core security rule that "computation occurred" must not be equated with "useful AI computation occurred."
- Keeps L5 (ZKML) honest about its actual 2026 limits instead of overclaiming coverage the way Allora's public messaging appears to (per `docs/research/competitors/allora.md`).

## Consequences
- `aion-verifier` crate and `services/verifier` must implement multiple verification strategies rather than one, increasing implementation surface — accepted because a single mechanism would leave entire workload classes either unverified (cheap fraud) or uneconomically over-verified (killing legitimate throughput).
- Every job's `verificationPolicy` becomes a first-class, signed field in the job protocol (`docs/PROTOCOL.md`) — a job cannot silently default to "no verification."
