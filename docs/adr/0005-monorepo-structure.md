# ADR 0005: Monorepo Structure

## Status
Accepted (structure scaffolded; most directories are stubs pending later phases)

## Context
The orchestration brief proposes a monorepo layout (`apps/`, `services/`, `crates/`, `packages/`, `contracts/`, `simulations/`, `benchmarks/`, `protocol/`, `research/`, `docs/`, `scripts/`). The brief explicitly permits deviating from this if research reveals a better structure, with the reason documented.

## Decision
Adopt the proposed structure as-is, with one adjustment: `research/` (top-level, brief's spec) is consolidated into `docs/research/` rather than kept as a separate top-level directory, since ADRs, specs, and research findings are all part of the same documentation lifecycle and a reader should find them in one tree. A stub top-level `research/` directory is kept (per the original spec) with a pointer file, in case future tooling expects that exact path.

## Reasoning
- Avoids two overlapping "docs-ish" top-level directories (`docs/` and `research/`) that reviewers would have to check both of.
- Every other directory in the brief's proposal maps to a real, distinct phase of work (Rust `crates/` for the node/protocol core, `services/` for off-chain coordination services, `apps/` for user-facing surfaces, `packages/` for shared TS code, `contracts/` for Solidity, `simulations/` for the economic model) — no other consolidation was justified.

## Consequences
- Most directories created this milestone (`apps/*`, `services/*`, `crates/*`, `packages/*`, `contracts/*`) are intentionally stub `README.md` files describing planned contents, not implementations — implementing them is later-phase work (Phases 3–10). Only `simulations/poig_sim` has working code in this milestone (Phase 2).
- Filling a stub is not itself an architectural decision requiring a new ADR; changing which top-level directory something lives in would be.
