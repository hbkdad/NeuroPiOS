---
name: aion-ai-runtime
description: Implement or modify AION's AI execution backend (crates/aion-runtime) - Ollama/vLLM/SGLang/PyTorch integration behind the RuntimeBackend trait.
---

## Purpose
Execute AI jobs across multiple inference/training engines without coupling job logic to any one engine.

## Trigger conditions
Adding/modifying a runtime backend integration, or any job-execution code that touches inference.

## Allowed tools
Read, Grep, Glob, Write, Edit, Bash, WebSearch (runtime APIs move fast — verify current API before coding).

## Preferred sources
`docs/adr/0006-ai-runtime-split.md`, official docs for the specific engine (Ollama/vLLM/SGLang/PyTorch+PEFT).

## Workflow
1. New engines integrate behind the `RuntimeBackend` trait — no engine-specific leakage into caller code.
2. Confirm `ModelManifest.metadata.runtimeRequirements` accurately reflects what the model actually needs before claiming compatibility.
3. Do not claim a model "runs" without actually executing it in an environment that has the runtime + hardware present.

## Required checks
No engine-specific assumptions outside the backend implementation itself.

## Expected outputs
Working, tested Rust code in `crates/aion-runtime` (or an explicit statement that testing requires an environment this container doesn't have, per `docs/research/CAPABILITY-MATRIX.md`).

## Failure conditions
Claiming inference succeeded without running it; breaking the RuntimeBackend abstraction boundary.
