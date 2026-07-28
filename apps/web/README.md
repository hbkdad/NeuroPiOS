# apps/web — AION Dashboard (planned, Phase 8)

Target stack (per `docs/adr/` and the orchestration brief; no ADR needed since it matches the brief's justified default and no alternative was required): Next.js, TypeScript, React, shadcn/ui, Tailwind CSS, TanStack Query, Zod.

Will surface: network overview, node/compute/job/model/agent/benchmark explorers, an intelligence index (see `docs/ARCHITECTURE.md`), and settings — all backed by real telemetry once `services/indexer` and `services/api` exist. No mock data will ship labeled as real; any placeholder data used during development must be visibly marked as mock in the UI itself.

**Status: not started.** This is a stub directory reserving the location in the monorepo layout (`docs/adr/0005-monorepo-structure.md`).
