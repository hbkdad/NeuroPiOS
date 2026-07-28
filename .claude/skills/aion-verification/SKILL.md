---
name: aion-verification
description: Implement or modify AION verification tiers (L0-L6) in crates/aion-verifier / services/verifier, or docs/VERIFICATION.md.
---

## Purpose
Ensure every rewarded job passes through an appropriate, honestly-scoped verification tier.

## Trigger conditions
Adding/modifying a verification method, tolerance band, or quorum-sizing rule.

## Allowed tools
Read, Grep, Glob, Write, Edit, Bash.

## Preferred sources
`docs/VERIFICATION.md`, `docs/adr/0003-verification-tiering.md`, `docs/research/AI-VERIFICATION-MATRIX.md`.

## Workflow
1. Identify which tier(s) the change affects; confirm the verification-rate floor (never zero) is preserved.
2. Numeric-tolerance bands are per-job-type, justified, not arbitrary constants.
3. Update `docs/research/AI-VERIFICATION-MATRIX.md` if a method's real-world cost/maturity has changed since last checked.

## Required checks
No reward path bypasses a declared verification tier.

## Expected outputs
Updated verification logic/docs with rationale for any tolerance or quorum parameter chosen.

## Failure conditions
A verification-rate floor of zero for any worker regardless of reputation; an undocumented tolerance constant.
