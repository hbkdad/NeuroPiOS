---
name: aion-redteam
description: Actively attempt to break AION's economic or technical mechanisms and report honest results (success or failure), updating docs/security/ECONOMIC-ATTACKS.md's simulated-status column.
---

## Purpose
Provide adversarial validation of AION's stated mitigations, not defensive review.

## Trigger conditions
A request to red-team a specific mechanism, or before treating an economic-attack mitigation as validated.

## Allowed tools
Read, Grep, Glob, Write, Edit, Bash.

## Preferred sources
`docs/security/ECONOMIC-ATTACKS.md`, `docs/security/THREAT-MODEL.md` attack tree.

## Workflow
1. Pick one specific attack from the attack tree/table.
2. Construct a concrete scenario, ideally an actual test run against real code (`simulations/poig_sim` today; contracts/services later).
3. Report success or failure honestly — a clean pass is a valid, useful result, not a reason to manufacture a finding.
4. If an attack succeeds, state exactly which invariant broke and where.

## Required checks
Every reported outcome (success/blocked) is backed by an actual run, not analysis alone.

## Expected outputs
Updated `docs/security/ECONOMIC-ATTACKS.md` "Simulated in Phase 2?" column and any newly-discovered gap documented in `docs/security/THREAT-MODEL.md`.

## Failure conditions
Reporting a mitigation as effective without having tried to break it; fabricating an attack result.
