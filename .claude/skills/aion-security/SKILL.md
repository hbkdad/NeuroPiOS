---
name: aion-security
description: Perform AION security review work - threat model updates, dependency/supply-chain checks, attack-tree maintenance. Not a substitute for external audit.
---

## Purpose
Keep `docs/security/THREAT-MODEL.md` accurate and catch known-vulnerable dependencies before they're added.

## Trigger conditions
A new entry point, external dependency, or trust-boundary change; a request for a security review.

## Allowed tools
Read, Grep, Glob, Write, Edit, Bash, WebSearch (advisory databases).

## Preferred sources
OWASP (incl. Smart Contract Security and LLM/GenAI guidance), OpenZeppelin, RustSec advisory DB, npm audit / GitHub Advisory Database.

## Workflow
1. New entry point -> new/updated threat-model row, not a silent gap.
2. New dependency -> check its advisory database before adding, not after.
3. Never mark something "secure" or "audited" without an actual external audit having occurred — this skill produces design-time review, not certification.

## Required checks
`docs/security/THREAT-MODEL.md`'s residual-risk section stays current with actual repo state.

## Expected outputs
Updated threat model, flagged dependency issues, never a "certified secure" claim.

## Failure conditions
Claiming audited/secure/production-ready without an actual external audit; adding a dependency with a known unpatched CVE/advisory.
