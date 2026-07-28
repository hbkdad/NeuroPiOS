---
name: aion-zkml
description: Investigate or implement AION's narrow ZKML proof-of-concept (Phase 11) - EZKL, small ONNX model proving, on/off-chain verifier.
---

## Purpose
Scope and (in Phase 11) implement a narrow ZKML verification proof-of-concept, without overclaiming general applicability.

## Trigger conditions
Any ZKML feasibility question, or Phase 11 implementation work.

## Allowed tools
Read, Grep, Glob, Write, Edit, Bash, WebSearch (proving-cost benchmarks move fast; re-verify before relying on old numbers).

## Preferred sources
EZKL official docs/benchmarks, current arXiv research on zkML proving cost (re-search — do not rely on the 2026-07-28 figures in `docs/research/AI-VERIFICATION-MATRIX.md` past a few months without re-checking).

## Workflow
1. Re-verify current proving-cost/model-size limits before starting (this space moves fast).
2. Scope to a small ONNX model or narrow sub-component (e.g. a quantization step) — never claim full LLM-inference ZKML coverage.
3. Install EZKL (`pip install ezkl`) only when this phase actually begins; not before.
4. Measure and report actual proving/verification time and cost — don't estimate.

## Required checks
No claim of ZKML coverage beyond what was actually measured in this proof-of-concept.

## Expected outputs
A working small-model proof + verifier, with measured proving/verification cost reported honestly.

## Failure conditions
Claiming general zkML inference verification is solved; skipping the re-verification of current proving-cost limits.
