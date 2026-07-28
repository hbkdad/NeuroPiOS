# AION System Architecture

Status: Phase 1 specification. Nothing described here beyond the Phase 2 simulator (`simulations/poig_sim`) is implemented yet — this document specifies the target, it does not claim the target is built.

## What AION is

Decentralized infrastructure for useful AI work: compute, model distribution, agent execution, and verification, coordinated by an economic layer (PoIG) that pays for *measured, verified* contribution rather than raw computation. See `docs/adr/` for the decisions behind each architectural choice below, and `docs/research/` for the competitor/technology research that grounds them.

## Layer diagram (target architecture, hypothesis per the orchestration brief, validated so far by ADRs 0001–0006)

```
                    AION Users
                        |
                        v
              Web (Next.js) / Desktop (Tauri) UI
                        |
                        v
                    AION API (Axum)
                        |
        +---------------+---------------+
        |               |               |
        v               v               v
    Scheduler       Registry        Indexer
        |          (model/job/     (chain events,
        |           worker)         job history)
        v
    P2P Network (rust-libp2p: GossipSub + Kademlia + Noise/QUIC)
        |
   +----+----+----+
   |    |         |
   v    v         v
 Worker Validator Storage
 Nodes   Nodes     Nodes
   |       |
   +---+---+
       v
  AI Execution (Ollama / vLLM / SGLang / PyTorch+PEFT — see ADR-0006)
       v
  Verification (tiered L0-L6 — see ADR-0003, VERIFICATION.md)
       v
  PoIG calculation (off-chain scoring — see POIG-SPEC.md)
       v
  Settlement (L2 testnet — see ADR-0001)
```

## Component responsibilities

- **Scheduler** (`services/scheduler`): matches jobs to workers using the Routing Score (see below); does not itself execute or verify jobs.
- **Registry** (`services/coordinator` + on-chain `ModelRegistry`/`WorkerRegistry`): canonical source of model manifests, worker capability advertisements, and job status.
- **Indexer** (`services/indexer`): reads chain events + P2P telemetry into a queryable store for the dashboard/explorer; read-only, never a source of truth for settlement.
- **P2P network** (`crates/aion-p2p`): peer discovery, job/result gossip, model-manifest propagation. Does not itself judge correctness.
- **Worker/Validator/Storage/Compute node roles** (`crates/aion-node`): a single physical device may run multiple roles simultaneously (see Node Architecture below); roles are capability flags, not separate binaries.
- **Verifier** (`crates/aion-verifier` + `services/verifier`): implements the L0–L6 tiers from ADR-0003. Produces a verification confidence signal consumed by PoIG, never itself pays out rewards.
- **PoIG scoring** (`crates/aion-reputation` + off-chain aggregator): computes `Reward` per `POIG-SPEC.md`; is a scoring function, not a consensus mechanism — see "PoIG is not consensus" below.
- **Settlement** (`contracts/`): escrow, staking, reward distribution, disputes, on the L2 chosen in ADR-0001.

## PoIG is not blockchain consensus

This is a load-bearing distinction from the orchestration brief, worth restating here: PoIG is an **economic contribution-scoring protocol** that runs off-chain (or in a rollup/appchain-internal computation), producing reward amounts that are then **settled** through ordinary L2 smart-contract transactions. Chain consensus (block production, finality) is entirely delegated to the chosen L2 (ADR-0001) — AION does not implement its own blockchain consensus algorithm.

## Node architecture

`aion-node` auto-detects OS, CPU, RAM, GPU/VRAM, GPU backend, disk, and bandwidth at startup, and exposes configurable resource caps (max CPU/GPU/RAM/storage/bandwidth %, idle-only mode, working hours, thermal limits) that the user must explicitly set before the node accepts jobs — a node never silently consumes host resources (see `docs/NODE.md`, Node Safety section, once written in a later milestone).

Node classes (capability flags, combinable on one device): `ComputeNode`, `InferenceNode`, `TrainingNode`, `StorageNode`, `ValidatorNode`, `BenchmarkNode`, `GatewayNode`.

## Identity separation

Three logically separate keypairs per operator, per the brief's Node Identity requirement:
1. **P2P identity** (libp2p PeerID, Ed25519) — used for transport-layer authentication only.
2. **Wallet identity** (secp256k1, Ethereum-style, used with the ERC-4337/EIP-7702 smart account from ADR-0001) — used only when settlement is actually required (job payment, staking), not broadcast on every gossip message.
3. **Reputation identity** — a commitment tied to the wallet identity but tracked separately in `aion-reputation` so reputation history can be queried without exposing the wallet address to every P2P peer.

## Routing score (initial, deterministic — see brief's Routing Engine section)

```
RoutingScore =
  w1 * reliability
+ w2 * performance
+ w3 * priceEfficiency
+ w4 * reputation
+ w5 * locality
+ w6 * verificationStrength
```

Weights are configuration, not constants, and must be tuned against the Phase 2 simulator's data before being treated as defaults for real scheduling. No learned/neural routing model is introduced until real production routing data exists (explicit brief constraint).

## Non-goals for this milestone

Full implementations of the scheduler, P2P node, verifier, and contracts are **not** part of this milestone. This document specifies their target shape; `simulations/poig_sim` (Phase 2) is the first executable proof of the PoIG/reputation/attack-resistance concepts described here, deliberately without any P2P, chain, or real-inference dependency.
