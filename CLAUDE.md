# CLAUDE.md

This repository contains two unrelated things:

1. **NeuroPiOS** (repo root: `index.html`, `releases/`) — a static landing page for an AI-first Raspberry Pi OS product, with a download button wired to GitHub Releases. Treat this as a live, shipped product; don't modify it as a side effect of AION work.
2. **AION** (`docs/`, `crates/`, `services/`, `contracts/`, `apps/`, `packages/`, `simulations/`, `.claude/agents/`, `.claude/skills/`) — a large, in-progress, multi-phase decentralized AI marketplace project. See `AGENTS.md` for the rules governing this subtree and `docs/ARCHITECTURE.md` / `docs/ROADMAP.md` for what it actually is and its current status.

If you were invoked to work on AION, read `AGENTS.md` first — it has the non-negotiable rules (no mainnet without authorization, no fabricated test/audit claims, ADR discipline, tiered verification, conservative slashing). If you were invoked to work on the NeuroPiOS landing page, `AGENTS.md`'s AION-specific rules don't apply — just edit `index.html`/`releases/` normally.

Claude Code subagents for AION specialist roles live in `.claude/agents/aion-*.md`; project skills live in `.claude/skills/aion-*/SKILL.md`. Both are scoped to this repo only.
