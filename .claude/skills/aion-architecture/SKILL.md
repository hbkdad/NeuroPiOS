---
name: aion-architecture
description: Design or revise AION system architecture, protocol boundaries, or component responsibilities. Use when asked to change docs/ARCHITECTURE.md, add a new component, or resolve an architectural question spanning multiple subsystems.
---

## Purpose
Keep `docs/ARCHITECTURE.md` and the ADR set internally consistent as AION's design evolves.

## Trigger conditions
A request to add/change a system component, resolve a cross-cutting design question, or reconcile two docs that now disagree.

## Allowed tools
Read, Grep, Glob, Write, Edit, Bash (for repo-wide consistency checks).

## Preferred sources
Existing `docs/adr/*.md` decisions first (don't contradict them silently); `docs/research/` for grounding; external research via the aion-research skill/agent when a genuinely new question arises.

## Workflow
1. Read all existing ADRs before proposing a change.
2. If the change contradicts an existing ADR, write a new ADR that explicitly supersedes it (state which one, and why).
3. Update `docs/ARCHITECTURE.md`'s layer diagram/component table to match.
4. Cross-check `docs/PROTOCOL.md`, `docs/security/THREAT-MODEL.md`, and `docs/ROADMAP.md` for consistency with the change.

## Required checks
No silent architectural drift — every deviation from a prior ADR is itself a new ADR.

## Expected outputs
Updated `docs/ARCHITECTURE.md` and/or a new `docs/adr/NNNN-*.md`.

## Failure conditions
Changing architecture without recording why; leaving `docs/ARCHITECTURE.md` inconsistent with the ADRs that actually govern the system.
