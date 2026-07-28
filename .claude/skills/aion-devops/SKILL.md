---
name: aion-devops
description: Set up or modify AION CI/CD, containerization, or the aion devnet local dev-network tooling. No automatic production/mainnet deployment ever.
---

## Purpose
Reproducible dev/CI environments and a safe, explicitly-scoped deployment pipeline.

## Trigger conditions
CI pipeline changes, Dockerfiles, `aion devnet` tooling, release scripting.

## Allowed tools
Read, Grep, Glob, Write, Edit, Bash.

## Preferred sources
GitHub Actions docs, Foundry/Cargo/npm official CI guidance; zero-cost-first tooling per the orchestration brief.

## Workflow
1. CI stages: format, lint, typecheck, unit tests, Rust tests, contract tests, integration tests, security scan, build, artifact generation.
2. Production/mainnet deployment is never reachable from an automated pipeline — this is a hard architectural constraint on any CI config written under this skill, not a preference.
3. Prefer free/local tooling; if a paid service is genuinely needed, document the cost and the free alternative considered first.

## Required checks
No code path in any CI/CD config can trigger a mainnet deployment automatically.

## Expected outputs
Working CI config with a real triggered run shown, or `aion devnet` scripts that actually start/stop a local multi-node setup.

## Failure conditions
Any automated path to production/mainnet deployment; claiming a pipeline "works" without having triggered it.
