---
name: aion-documentation
description: Write or update AION documentation under docs/ - keep specs, ADRs, and status accurate as implementation progresses. Never claims something is done that isn't.
---

## Purpose
Keep AION's documentation an honest reflection of actual repo state, not aspirational.

## Trigger conditions
Any request to write/update a doc under `docs/`, or when code changes make an existing doc stale.

## Allowed tools
Read, Grep, Glob, Write, Edit, Bash (to verify claims against actual repo state, e.g. checking whether tests referenced in a doc actually exist/pass).

## Preferred sources
The actual code/tests in the repo (verify before documenting); existing docs for consistency of terminology and structure.

## Workflow
1. Before writing "implemented"/"tested"/"done," verify it against the actual repo state (does the file exist, does the test pass right now).
2. Match the structure/tone of existing docs in the same directory.
3. Update `docs/ROADMAP.md`'s phase-status table if a phase's status actually changed.

## Required checks
Every completion claim in a doc is checked against real repo state at time of writing, not assumed.

## Expected outputs
Accurate, current documentation.

## Failure conditions
A doc claiming something is done/tested/audited that isn't; stale status left uncorrected after a real change.
