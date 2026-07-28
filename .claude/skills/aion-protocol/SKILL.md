---
name: aion-protocol
description: Change or extend the AION job protocol, model manifest, or benchmark object schemas in docs/PROTOCOL.md, or implement them in crates/aion-protocol.
---

## Purpose
Keep the wire-level job/model/benchmark protocol correct, versioned, and replay-safe.

## Trigger conditions
Adding/changing a field on the job object, model manifest, or benchmark object; implementing or modifying `crates/aion-protocol`.

## Allowed tools
Read, Grep, Glob, Write, Edit, Bash.

## Preferred sources
`docs/PROTOCOL.md` (current spec), `simulations/poig_sim/poig_sim/protocol.py` (Phase 2 semantics reference — not the wire-format authority).

## Workflow
1. Determine if the change is additive (minor version bump) or breaking (major version bump, per `docs/PROTOCOL.md`'s versioning rule).
2. Verify replay protection (nonce) and terminal-state invariants (no double-settlement) still hold.
3. Update canonical serialization field order if a field is added — never reorder existing fields.
4. Update `docs/PROTOCOL.md` and, if implemented, the corresponding code + tests.

## Required checks
A protocol change never breaks nonce-based replay protection or the settle-once invariant.

## Expected outputs
Updated `docs/PROTOCOL.md`, version bump noted, corresponding implementation/tests if code exists yet.

## Failure conditions
A breaking schema change without a major version bump; a new field with no replay/serialization consideration.
