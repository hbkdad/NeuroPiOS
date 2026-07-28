---
name: aion-benchmarking
description: Create or run AION benchmark definitions (docs/PROTOCOL.md Benchmark object) or measure real performance (latency, throughput, proving cost). Never optimizes without measurements.
---

## Purpose
Produce hidden/rotating benchmark suites for Proof of Improvement, and measure real system performance rather than assuming it.

## Trigger conditions
A request to define a benchmark, or to measure/optimize performance of any AION component.

## Allowed tools
Read, Grep, Glob, Write, Edit, Bash.

## Preferred sources
`docs/PROTOCOL.md` (Benchmark object schema), `docs/POIG-SPEC.md` (Proof of Improvement anti-overfitting requirements).

## Workflow
1. For a new benchmark: define `benchmarkId`, domain, metric, dataset commitment, evaluation code CID, anti-leakage policy (hidden/rotating), verification rules — per the schema.
2. For performance work: measure first (job scheduling latency, P2P discovery, inference latency, verification overhead, proof time, DB latency, etc. per the orchestration brief's Performance Testing section), then optimize only what the measurement shows is actually slow.
3. Never claim a benchmark result without having actually run the measurement.

## Required checks
Anti-leakage policy is specified for every new benchmark (hidden test sets, rotation schedule).

## Expected outputs
A benchmark definition file, or a measured performance report with real numbers.

## Failure conditions
Optimizing without a measurement; a benchmark with no anti-leakage policy (allows overfitting).
