# ADR 0001: Settlement Layer — L2 Testnet over Custom L1

## Status
Accepted (development-stage decision; revisit before any mainnet commitment)

## Context
The orchestration brief's initial hypothesis is that L2/testnet settlement is preferable to a custom L1, and that AION must NOT build a custom L1 unless research conclusively shows existing settlement infrastructure cannot satisfy AION's requirements. Requirements: cheap/frequent micropayment settlement, escrow, staking/slashing, smart-account wallet UX (no manual gas management for ordinary users), and eventual governance.

## Options considered
1. **Ethereum L1** — strongest security/decentralization, but gas cost and block time are a poor fit for high-frequency micropayment settlement of AI job rewards.
2. **Existing L2 (Base, Arbitrum, Optimism, or an OP-Stack/Arbitrum-Orbit rollup AION could later deploy itself)** — inherits Ethereum's security via fraud/validity proofs, low fees, and — critically — mature ERC-4337/EIP-7702 account-abstraction tooling already in production (30M+ smart accounts, production paymaster infra across these chains as of mid-2026).
3. **Cosmos SDK custom app-chain** (the path Akash and Allora both took) — full sovereign control over consensus/economics, but requires bootstrapping and securing an independent validator set from day one, and forfeits Ethereum's existing wallet/tooling ecosystem.
4. **Custom L1 from scratch** — maximal control, maximal cost and risk; no requirement identified that existing infra cannot satisfy.

## Decision
Start on an **existing L2 testnet** (Base Sepolia or OP Sepolia — both post-Pectra, both with EIP-7702 support and mature ERC-4337 bundler/paymaster infrastructure). Do not build a custom chain or appchain in Phases 0–9. Re-evaluate only if Phase 10+ testnet operation surfaces a concrete requirement (e.g., settlement throughput/cost) that the chosen L2 cannot meet.

## Reasoning
- Directly satisfies the "Zero-Cost Development Priority" and "prefer battle-tested primitives" principles: no validator set to bootstrap, no consensus code to write or audit, free public testnet RPCs exist today.
- ERC-4337 + EIP-7702 maturity on Base/Optimism directly solves the Wallet UX requirement (gas sponsorship, session keys, batched transactions) without AION building any of that itself.
- Both Akash and Allora chose sovereign Cosmos chains and both carry the ongoing cost of securing an independent validator set — a cost AION has no demonstrated need to take on at this stage.
- Reversible: an L2-based system's core contracts (registry, escrow, staking) can be redeployed to a different L2 or a future AION-controlled rollup later without protocol-level redesign, since the job/verification/PoIG protocol layers are chain-agnostic by design (see `docs/PROTOCOL.md`).

## Consequences
- AION depends on the chosen L2's uptime/decentralization properties; this is an accepted tradeoff versus building sovereign infrastructure.
- Smart contracts must be written chain-agnostically (standard EVM, OpenZeppelin primitives) so a future migration or multi-chain deployment is not blocked by chain-specific code.
- No blockchain deployment happens until Phase 10, and no mainnet deployment happens without explicit operator authorization (per Token Development Policy, Stage A–G).

## Open questions for Phase 10
- Base Sepolia vs. OP Sepolia vs. Arbitrum Sepolia — final pick should be made against actual measured gas cost for AION's specific escrow/settlement transaction shapes, not this document alone.
