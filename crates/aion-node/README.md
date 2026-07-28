# crates/aion-node — Node Runtime (planned, Phase 3)

Rust binary combining node identity, hardware auto-detection (OS/CPU/RAM/GPU/VRAM/disk/bandwidth), Node Safety resource caps, and the composable node-role flags (`ComputeNode`, `InferenceNode`, `TrainingNode`, `StorageNode`, `ValidatorNode`, `BenchmarkNode`, `GatewayNode`) described in `docs/ARCHITECTURE.md`. Depends on `aion-p2p`, `aion-protocol`, `aion-crypto`, `aion-runtime`, `aion-verifier`, `aion-storage`, `aion-reputation`. **Status: not started.**
