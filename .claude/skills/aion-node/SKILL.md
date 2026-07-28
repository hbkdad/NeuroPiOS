---
name: aion-node
description: Implement or modify the AION node runtime (crates/aion-node) - hardware detection, node roles, resource safety caps, identity.
---

## Purpose
Build/maintain the node binary that hosts compute/inference/training/storage/validator/benchmark/gateway roles.

## Trigger conditions
Work on hardware auto-detection, node-role composition, resource-cap enforcement, or node identity (P2P/wallet/reputation separation).

## Allowed tools
Read, Grep, Glob, Write, Edit, Bash.

## Preferred sources
`docs/ARCHITECTURE.md` (Node Architecture, Node Safety, Identity Separation sections).

## Workflow
1. Never let a node consume resources beyond operator-configured caps, even implicitly (e.g. via a dependency's default thread pool sizing).
2. Keep P2P identity, wallet identity, and reputation identity as separate keypairs/commitments, never conflated.
3. Node role flags are composable (a device can run multiple roles) — don't hardcode single-role assumptions.

## Required checks
Resource caps are enforced, not advisory; identity separation is preserved in any new code path.

## Expected outputs
Working, tested Rust code in `crates/aion-node`.

## Failure conditions
A code path that silently exceeds a configured resource cap; wallet address leaked onto the P2P layer where only PeerID was needed.
