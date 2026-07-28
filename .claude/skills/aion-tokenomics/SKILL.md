---
name: aion-tokenomics
description: Design, simulate, or refine AION's PoIG reward mechanism and token economics - docs/POIG-SPEC.md, docs/ECONOMIC-MODEL.md, simulations/poig_sim.
---

## Purpose
Validate economic mechanisms through simulation before treating any parameter as production-ready.

## Trigger conditions
Changes to the PoIG reward formula, fee-split design, Sybil-resistance parameters, or economic simulation code/scenarios.

## Allowed tools
Read, Grep, Glob, Write, Edit, Bash.

## Preferred sources
`docs/POIG-SPEC.md`, `docs/ECONOMIC-MODEL.md`, `docs/security/ECONOMIC-ATTACKS.md`, competing-network research in `docs/research/competitors/`.

## Workflow
1. Any formula/parameter change must be exercised by a `pytest` run in `simulations/poig_sim` with real output before being described as validated.
2. Preserve the `UsefulWorkScore` gate (escrow-backed signed job or benchmark provenance) — this is the anti-farming invariant the whole reward system depends on; relaxing it needs an explicit ADR.
3. Follow the staged token policy (`docs/adr/0004-token-development-staging.md`) — never introduce real monetary value in simulation code.

## Required checks
Every "mitigated" attack claim in `docs/security/ECONOMIC-ATTACKS.md` is backed by an actual passing test.

## Expected outputs
Updated simulator code + tests, updated spec docs, honest updates to the attack-mitigation table.

## Failure conditions
An unvalidated constant presented as tuned; a weakened UsefulWorkScore gate without an ADR and explicit flag to the operator.
