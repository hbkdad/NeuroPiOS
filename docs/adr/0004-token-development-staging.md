# ADR 0004: Token Development Staging (A–G), No Tradable Token Now

## Status
Accepted

## Context
The orchestration brief is explicit: no tradable token during initial development, and no production token deployment without explicit operator authorization. AION must not be built or described as a speculative cryptocurrency with AI branding.

## Decision
Follow the staged token policy exactly as specified:

- **Stage A** — internal points ledger (off-chain, in the Phase 2 simulator and later `services/coordinator`), no monetary value, used purely to validate PoIG scoring logic.
- **Stage B** — local simulation token (still off-chain, used in Monte Carlo / agent-based economic simulations under `simulations/`).
- **Stage C** — testnet token (on the L2 testnet chosen in ADR-0001, zero real value, faucet-distributed).
- **Stage D** — closed economic testnet (invited participants, adversarial testing of the mechanisms in `docs/security/ECONOMIC-ATTACKS.md`).
- **Stage E** — public testnet.
- **Stage F** — legal/security review (external; explicitly out of scope for this agent to perform or claim).
- **Stage G** — production economic network — **requires explicit operator authorization**, not a decision this agent makes autonomously under any circumstance (per the brief's Autonomous Decision Policy: "mainnet token deployment" is a listed reason to stop and ask a human).

## Reasoning
- Matches the brief's non-negotiable staging and its explicit list of things that require human intervention.
- Keeps every economic mechanism (PoIG, slashing, royalties) testable and falsifiable in simulation (Stage A/B) before any value is at risk (Stage C+), which is also just good practice for a mechanism-design problem this complex.

## Consequences
- Current repo state (this milestone) is Stage A only: the `simulations/poig_sim` Python package uses an in-memory points ledger with no blockchain dependency at all.
- Any future PR that introduces a real token contract, a mainnet RPC endpoint, or a production deployment script must be treated as a Stage F/G action and explicitly flagged to the operator before merging — this is a standing constraint on all future work on this repo, not just this milestone.
