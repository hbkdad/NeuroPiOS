---
name: aion-smart-contracts
description: Implement or modify AION settlement contracts under contracts/ - escrow, staking, rewards, slashing, registries, governance. Phase 10+ only.
---

## Purpose
Build AION's on-chain settlement layer safely, with mandatory testing before any deployment.

## Trigger conditions
Any change under `contracts/`.

## Allowed tools
Read, Grep, Glob, Write, Edit, Bash.

## Preferred sources
OpenZeppelin Contracts docs, Foundry docs, `docs/adr/0001-settlement-layer.md`, `docs/security/THREAT-MODEL.md` core invariants.

## Workflow
1. Confirm Foundry is actually installed before writing Solidity that assumes a toolchain exists.
2. Use OpenZeppelin primitives for access control, pausability, reentrancy protection — no reimplementation.
3. Write unit + fuzz + invariant tests (`contracts/test/`, `contracts/invariant/`) covering the core invariants in `docs/security/THREAT-MODEL.md` before considering any contract done.
4. Run Slither/Echidna before proposing any testnet deployment.
5. No deployment anywhere without operator-provided RPC + funded key for that specific action; no mainnet ever without explicit authorization.

## Required checks
Every core invariant (no double-settlement, escrow can't create value, only verified jobs settle, no unauthorized minting) has an actual passing invariant test.

## Expected outputs
Tested Solidity contracts with static-analysis output shown.

## Failure conditions
Claiming a contract is tested/audited without shown output; any deployment action without explicit operator-provided credentials for that action.
