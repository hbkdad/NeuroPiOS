# AGENTS.md — AION project (this repo also hosts the unrelated NeuroPiOS landing page, see README.md)

This file configures coding agents (Claude Code, and any Codex/AGENTS.md-compatible tool) working on the **AION** subtree of this repository. It does not describe the NeuroPiOS static site (`index.html`, `releases/`), which is a separate, unrelated product living in the same repo.

## What AION is

See `docs/ARCHITECTURE.md` for the full picture. Short version: decentralized AI compute/model/agent marketplace, economically coordinated by PoIG (Proof of Intelligence Gain), built incrementally per `docs/ROADMAP.md`. Read `docs/ARCHITECTURE.md`, `docs/PROTOCOL.md`, and `docs/POIG-SPEC.md` before making any change to `crates/`, `services/`, `contracts/`, or `simulations/`.

## Non-negotiable rules for anyone (human or agent) working in this subtree

1. **No mainnet, no real token, no production blockchain deployment** without explicit human authorization — see `docs/adr/0004-token-development-staging.md`. This is a hard stop condition, not a judgment call.
2. **Never equate "computation occurred" with "useful AI computation occurred."** Any change to `POIG-SPEC.md` or `crates/aion-reputation` must preserve the `UsefulWorkScore` gate (escrow-backed signed job or network benchmark provenance) — see `docs/POIG-SPEC.md`.
3. **Don't claim "tested," "audited," "secure," or "production-ready"** unless that action actually happened in this session and its output is shown. Simulation results must show real pass/fail output, not assumed success.
4. **New architectural decisions get an ADR** (`docs/adr/NNNN-*.md`) — context, options considered, decision, reasoning, consequences. Don't silently deviate from an existing ADR; supersede it explicitly if research changes the answer.
5. **Verification is tiered, never one-size-fits-all** — see `docs/VERIFICATION.md`. Don't add a code path that pays a reward without going through a declared verification tier.
6. **Slashing is conservative** — see `docs/protocol/SLASHING-SPEC.md`. Ambiguous failures (hardware, network, nondeterminism) are never slashable.

## Where things live

- `docs/` — specs, ADRs, research, security docs. Read before you write code.
- `crates/` — Rust: node, P2P, protocol, crypto, runtime, verifier, storage, reputation, CLI. Not yet implemented beyond stubs.
- `services/` — off-chain coordination services (Rust/Axum target). Not yet implemented beyond stubs.
- `contracts/` — Solidity (Foundry target, not yet installed in this environment). Not yet implemented.
- `apps/` — web/desktop/explorer/docs-site. Not yet implemented beyond stubs.
- `packages/` — shared TS (`sdk`, `ui`, `types`, `config`). Not yet implemented beyond stubs.
- `simulations/poig_sim` — **the only subtree with real, tested code as of this milestone.** Python, pytest. Run `pytest` from `simulations/poig_sim` before treating any change there as done.
- `.claude/agents/` — specialist subagent definitions for this project (research, protocol architect, distributed systems, AI systems, verification, crypto, smart contracts, economics, security, red team, UI/UX, QA, DevOps).
- `.claude/skills/` — project skills, one per `aion-*` workflow named in the original orchestration brief.

## Autonomous decision policy

Choose the safest technically-justified default and record it as an ADR. Stop and ask a human only for: credentials, financial commitments, irreversible production deployments, legal decisions, mainnet token deployment, destructive actions, or material scope changes (e.g., this repo already hosts an unrelated live product — NeuroPiOS — confirm before any change that would affect it).
