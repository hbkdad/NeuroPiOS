# crates/aion-node — Node Runtime

Node identity, hardware auto-detection, Node Safety resource caps, and composable node-role flags, per `docs/ARCHITECTURE.md`.

**Status: real implementation (partial), milestone 3.** `cargo test -p aion-node` → 13/13 passing:

- `hardware.rs` — real hardware detection via the `sysinfo` crate (OS, hostname, CPU core count, total memory/swap), tested against the actual values in whatever container/host runs the tests — not mocked. GPU/VRAM detection is explicitly deferred (no portable dependency-light way to enumerate CUDA/ROCm/Metal GPUs, and no GPU exists in this dev container to test against — see `docs/research/CAPABILITY-MATRIX.md`), not stubbed with fake values.
- `resource_caps.rs` — the Node Safety enforcement point: `ResourceCaps::locked_down()` is the mandatory default (a fresh node accepts *no* work until the operator explicitly widens its caps), plus CPU/memory/working-hours checks, including a working-hours window that correctly wraps past midnight.
- `roles.rs` — composable node-role flags (`Compute`, `Inference`, `Training`, `Storage`, `Validator`, `Benchmark`, `Gateway`); a single node can hold any combination.
- `lib.rs` — `Node::bootstrap()` ties these together with a real `aion_crypto::Identity` (fresh Ed25519 keypair per node).

**Not yet done:** actual P2P wiring to `crates/aion-p2p` (this crate and `aion-p2p` are still independent — `Node` doesn't yet own a running swarm), and the `aion-runtime`/`aion-verifier`/`aion-storage`/`aion-reputation` dependencies listed in the original plan (those crates don't exist yet).
