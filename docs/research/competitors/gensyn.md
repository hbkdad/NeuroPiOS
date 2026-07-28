# Competitor Analysis: Gensyn

## Architecture
Ethereum-aligned protocol focused on verifiable, distributed ML compute. Two visible tracks as of mid-2026: **RL Swarm** (open-source collaborative RL training over the internet, currently running the "CodeZero" task domain) and **Delphi** (a permissionless AI-settled prediction-market app, positioned as the first Gensyn-mainnet application). RL-Swarm-hosted nodes were paused mid-2026 while Gensyn focuses mainnet launch work on Delphi.

## Consensus / verification mechanism
The core research contribution is **Verde**, a dispute-arbitration framework for ML programs, plus the **Reproducible Execution Environment (REE)** for verifiable settlement. Verde's approach: don't require every node to agree on bit-exact output (ML execution is not naturally reproducible across heterogeneous hardware/kernels); instead use **lightweight dispute resolution** — a prover executes and commits to a result, and disputes are resolved via a **refereed delegation** game (bisection-style, similar in spirit to optimistic-rollup fraud proofs) rather than requiring full re-execution by every verifier up front.

## Worker model
RL Swarm nodes run open-source training/RL loops (CodeZero replaced "Reasoning Gym" as the active environment in Nov 2025), scored by execution-based or model-based reward functions rather than purely rule-based logic.

## Reward system
Coordinated on-chain via `rl-swarm-contracts`; specifics of mainnet tokenomics are not yet finalized publicly as of this research pass — testnet is explicitly a pre-mainnet proving ground, not the production incentive design.

## Strengths
- Verde/REE directly targets AION's hardest open problem: verifying ML execution across non-deterministic, heterogeneous hardware without requiring full re-execution or expensive ZK proofs for every job.
- Optimistic dispute framing (commit → challenge window → bisection) is cheap in the common case (no dispute) and only pays arbitration cost when something is actually contested — a good fit for AION's tiered verification model (Level 4 in `docs/VERIFICATION.md`).

## Weaknesses / open risks
- Reproducibility across GPU vendors/driver versions/kernel fusion is still a hard unsolved problem generally, not just for Gensyn — floating-point non-associativity means "the same model, same input" can legitimately produce different outputs on different hardware. Any dispute protocol needs a tolerance model, not bit-exact equality.
- Mainnet economics and the fraud-proof bisection game's actual on-chain cost are not yet publicly battle-tested at scale.
- CodeZero-style execution-based reward functions are themselves gameable (reward hacking within the eval harness) unless the harness itself is adversarially maintained.

## What AION should borrow
- The **optimistic commit → dispute-window → bisection** verification pattern for Level 4 ("Challenge Protocol") in AION's verification tiering — cheap by default, expensive only under dispute.
- Explicit acknowledgment that bit-exact reproducibility is not achievable for most real ML workloads; AION's `VERIFICATION.md` and `SLASHING-SPEC.md` define a numerical-tolerance band and distinguish "wrong answer" from "hardware/numerical variance" before slashing.

## What AION should avoid
- Publicly promising a verification mechanism (REE) years before it is proven at mainnet scale under adversarial load — AION's docs should mark verification levels above L2 as experimental until measured.

Sources: [Gensyn Testnet — How It Works](https://docs.gensyn.ai/testnet/rl-swarm/how-it-works), [Gensyn Testnet Overview](https://docs.gensyn.ai/testnet), [gensyn-ai/rl-swarm](https://github.com/gensyn-ai/rl-swarm), [gensyn-ai/rl-swarm-contracts](https://github.com/gensyn-ai/rl-swarm-contracts), [DeSpread — RL Swarm Node Operation Guide](https://research.despread.io/gensyn-guide/)
