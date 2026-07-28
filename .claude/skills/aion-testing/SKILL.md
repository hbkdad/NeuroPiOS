---
name: aion-testing
description: Run or write tests for AION code - simulations/poig_sim pytest suite today, crates/services/contracts test suites once they exist. Always runs tests for real and reports actual output.
---

## Purpose
Provide honest, executed test verification — never a claimed-but-unrun result.

## Trigger conditions
Any code change to `simulations/poig_sim` or (later) `crates/`, `services/`, `contracts/`.

## Allowed tools
Read, Grep, Glob, Write, Edit, Bash.

## Preferred sources
Existing test files as the pattern to follow; `docs/POIG-SPEC.md`/`docs/protocol/SLASHING-SPEC.md` for the invariants tests must encode.

## Workflow
1. `cd simulations/poig_sim && pytest -q` (or the equivalent for whichever subtree changed) and capture real output.
2. Report pass/fail counts and any failure detail — never summarize as "tests pass" without having run them this session.
3. When a test can't be run in this environment (e.g. no GPU for `aion-runtime`), state that explicitly instead of skipping the caveat.

## Required checks
Test run actually executed in this session before any pass/fail claim.

## Expected outputs
Real pytest/cargo-test/forge-test output, quoted or summarized accurately.

## Failure conditions
Claiming tests pass without running them; silently omitting a failure.
