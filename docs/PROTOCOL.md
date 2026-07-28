# AION Job & Model Protocol Specification

Status: Phase 1 specification (schema-level design). Reference implementation lives in `simulations/poig_sim/poig_sim/protocol.py` (Phase 2, in-process only — not yet a wire protocol over real P2P transport).

## Versioning

Every serialized protocol object carries a `protocolVersion` field (semver-like, `major.minor`). Consumers reject objects with an unknown `major` version rather than attempting best-effort parsing. This document specifies **protocolVersion 0.1** (pre-network, simulation-only).

## Canonical job object

```json
{
  "protocolVersion": "0.1",
  "jobId": "uuid-v4",
  "requester": "wallet-address-or-agent-id",
  "jobType": "inference | fine_tune | train | compress | quantize | benchmark_create | benchmark_run | dataset_contribute | evaluate | host | route | agent_execute",
  "model": {
    "cid": "content-id-of-model-manifest",
    "version": "semver"
  },
  "inputCommitment": "hash-of-input-or-encrypted-payload-reference",
  "executionProfile": {
    "runtime": "ollama | vllm | sglang | pytorch",
    "hardware": ["gpu-class", "min-vram-gb", "region-hint"]
  },
  "verificationPolicy": "L0 | L1 | L2 | L3 | L4 | L5 | L6",
  "maxPrice": "decimal-string, token-denominated",
  "deadline": "iso8601",
  "nonce": "monotonic-per-requester, replay protection",
  "signature": "signature-over-canonical-serialization-of-all-above-fields"
}
```

### Replay protection
`nonce` is monotonically increasing per `requester` and tracked by the coordinator/registry; a job with a nonce ≤ the requester's last-seen nonce is rejected before it ever reaches a worker. `jobId` alone is not sufficient replay protection since it does not prevent a requester from re-signing an old commitment with a new ID — the nonce is what actually prevents replay of a stale signed intent.

### Canonical serialization
Fields are serialized in the fixed key order shown above (not alphabetical, not insertion order) before signing, to avoid cross-implementation signature mismatches. A conformance test in a future milestone must assert that two independent serializers of the same object produce byte-identical output.

## Model manifest (content-addressed, per Model Distribution requirements)

```
ModelManifest
 ├── metadata (id, creator, architecture, version, parentModel, license, size, quantization, runtimeRequirements)
 ├── weightsCid
 ├── tokenizerCid
 ├── configCid
 ├── licenseCid
 └── benchmarkManifestCid
```

Weights are never stored on-chain; the chain (or coordinator, pre-Phase-10) only ever stores the manifest's content identifier. See `docs/adr/0005-monorepo-structure.md` context and the brief's Storage Research section (IPFS/Kubo/Bitswap as the reference content-addressing implementation, Phase 6).

## Model provenance DAG

Each `ModelManifest.metadata.parentModel` link forms a DAG edge; a model can have multiple parents (e.g., base model + adapter + dataset + training recipe, per the brief's Contribution DAG example). Attribution/royalty computation over this DAG is performed off-chain (see `POIG-SPEC.md` — Model Royalties) to avoid unbounded on-chain gas costs from recursive attribution; only the final aggregated distribution is settled on-chain.

## Benchmark object

```
Benchmark
 ├── benchmarkId, version, creator, domain, metric
 ├── datasetCommitment
 ├── evaluationCodeCid
 ├── expectedRuntime
 ├── antiLeakagePolicy   // e.g. "rotating", "hidden-until-execution"
 └── verificationRules    // which verification tier(s) apply to submissions against this benchmark
```

## Job lifecycle (state machine)

```
CREATED --(signed, escrow funded)--> OPEN --(worker accepts)--> ASSIGNED
  --> EXECUTING --> RESULT_SUBMITTED --> (verification per verificationPolicy)
  --> VERIFIED --> SETTLED
                 \-> DISPUTED --> (challenge/arbitration, see SLASHING-SPEC.md) --> RESOLVED --> SETTLED | SLASHED
```

A job cannot settle twice: `SETTLED` and `SLASHED` are terminal states; the escrow contract must reject any settlement instruction referencing a `jobId` already in a terminal state (this is a core invariant, see `docs/security/THREAT-MODEL.md`).

## Open items for the next specification pass
- Exact binary/CBOR wire encoding for P2P transport (Phase 3) — this document currently specifies JSON-shaped schemas for readability; wire format may differ for efficiency.
- Agent-economy job objects (agent-to-agent paid requests) — extends this schema with a `spendingAuthorization` reference; deferred until the Agent Economy section of `ECONOMIC-MODEL.md` is implemented.
