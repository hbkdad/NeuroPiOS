# Competitor Analysis: Allora Network

## Architecture
Sovereign Cosmos SDK Layer 1 (mainnet since November 2025), branded a "Model Coordination Network" (MCN). Organizes participants into **Topics** (55+ live as of 2026) — analogous to Bittensor subnets — where many independent models submit inferences/forecasts on a shared prediction task and are combined/scored together rather than competing 1:1.

## Consensus / verification mechanism
Claims to integrate **zkML** to cryptographically verify model-execution integrity without exposing proprietary weights, branded internally as "Proof of Intelligence." As of this research pass, the depth of zkML coverage (which model classes, what proving cost) is not independently detailed in available sources — treat the "auditable AI via zkML" claim as a marketing-level claim to verify against Allora's own technical docs before relying on it architecturally.

## Worker model
288,000+ active "workers" (per Feb 2026 figures) submit inferences per Topic; a **reputer** layer (validators, in Allora's terms) scores worker submissions for accuracy against ground truth or consensus, feeding a combined "network inference" per Topic.

## Reward system
Emission weighted by scored accuracy per Topic, with reputers themselves scored on how well their evaluations track eventual ground truth — a proof-of-accuracy design, conceptually the closest existing system to AION's "Proof of Improvement" (statistical, ground-truth-anchored evaluation) rather than raw-compute proof.

## Strengths
- Reported real usage scale (692M+ inferences by Feb 2026, 130+ protocol/enterprise integrations by mid-2026) — the largest publicly reported inference volume among the decentralized-AI competitors researched here.
- Topic-based model coordination (many independent models scored on the same task, combined into a network-level output) is close in spirit to AION's benchmark-registry + hidden-test-set model-improvement loop.
- Explicit "accuracy against ground truth," not just "agreement with other validators," is a stronger anti-collusion property than Bittensor's Yuma design *if* the ground-truth source is itself trustworthy and not gameable.

## Weaknesses / open risks
- The zkML "auditable AI" claim needs independent technical verification — proving full model-execution integrity in zero knowledge at the scale of hundreds of millions of inferences is not consistent with current zkML proving-cost benchmarks (EZKL, Bionetta, NANOZK research all still describe ZK LLM-inference proving as economically impractical at scale in 2026). AION should not assume Allora has actually solved general zkML inference verification — more likely a narrower proof is in use (e.g., proving cheap post-hoc checks, not full inference) or the claim is aspirational.
- Ground-truth availability is workload-dependent: prediction-market-style Topics have a natural eventual ground truth (did the price move as predicted); many useful AI tasks (open-ended generation, code review) do not have an equally clean ground truth, limiting how far this design generalizes.
- Sovereign L1 choice (own consensus/validator set to secure) is a heavier infrastructure commitment than settling on an existing rollup — directly relevant to AION's Stage-0 hypothesis that L2 settlement is preferable to a custom chain (see ADR 0001).

## What AION should borrow
- Ground-truth-anchored accuracy scoring (Proof of Improvement in `POIG-SPEC.md`) over pure agreement-based scoring, for task classes where an eventual ground truth exists or can be constructed (hidden benchmarks, rotating test sets).
- Topic/subnet-style task partitioning so unrelated task types don't share one global scoring function.

## What AION should avoid
- Claiming a verification guarantee (full zkML execution integrity) that current proving-cost research does not support at the claimed inference volume — AION's `AI-VERIFICATION-MATRIX.md` must state proving cost and model-size limits honestly per method, and avoid the same category of unverifiable claim.
- Building on a brand-new sovereign L1 before demonstrating that existing settlement infra is insufficient (this is directly the question ADR 0001 answers for AION).

Sources: [BTCC — What Is Allora](https://www.btcc.com/en-CA/academy/crypto-wiki/altcoin/what-is-allora-network-allo), [Messari — Understanding Allora Network](https://messari.io/report/understanding-allora-network-a-comprehensive-overview), [KuCoin — 2026 AI+Crypto Landscape](https://www.kucoin.com/blog/deep-dive-to-AI-crypto-future), zkML proving-cost cross-reference: [EZKL Benchmarks](https://blog.ezkl.xyz/post/benchmarks/), [Bionetta arXiv](https://arxiv.org/pdf/2510.06784), [NANOZK arXiv](https://arxiv.org/pdf/2603.18046)
