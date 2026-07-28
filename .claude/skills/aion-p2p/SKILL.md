---
name: aion-p2p
description: Implement or modify AION's P2P networking layer (crates/aion-p2p) - GossipSub, Kademlia, NAT traversal, peer scoring.
---

## Purpose
Build/maintain the rust-libp2p-based P2P layer per `docs/adr/0002-p2p-stack.md`.

## Trigger conditions
Work on peer discovery, gossip topics, NAT traversal, or anti-spam/peer-scoring in `crates/aion-p2p` or `crates/aion-node`.

## Allowed tools
Read, Grep, Glob, Write, Edit, Bash, WebSearch (for current rust-libp2p API checks).

## Preferred sources
rust-libp2p official docs/GitHub (version-check before relying on API shape, since this crate evolves).

## Workflow
1. Confirm the rust-libp2p version pinned in `Cargo.toml` before writing code against its API.
2. Any new gossip topic gets a peer-scoring/rate-limit policy defined alongside it, not added later.
3. Test NAT-traversal-failure and partition scenarios explicitly, not just the happy path.

## Required checks
Every P2P feature respects `docs/ARCHITECTURE.md`'s Node Safety resource caps even under load.

## Expected outputs
Working, tested Rust code in `crates/aion-p2p`; updated ADR if the stack choice itself changes.

## Failure conditions
A new gossip topic with no anti-spam policy; untested partition/NAT-failure behavior.
