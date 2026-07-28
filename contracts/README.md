# contracts — AION Settlement Contracts (planned, Phase 10)

Target: Solidity via Foundry, OpenZeppelin primitives, mandatory Slither + Echidna + fuzz/invariant testing before any testnet deployment (see `docs/adr/0001-settlement-layer.md`, `docs/SECURITY.md`). Planned modules: `AionRegistry`, `ModelRegistry`, `JobEscrow`, `WorkerRegistry`, `StakeManager`, `RewardDistributor`, `DisputeManager`, `ReputationCommitment`, `Treasury`, `Governance`.

**Status: not started.** No Solidity toolchain is installed in the current development container (see `docs/research/CAPABILITY-MATRIX.md`) — this is a real gap, not a placeholder; Foundry must be installed (`foundryup`) when this phase begins. No blockchain deployment happens without an explicit testnet RPC + funded deployer key provided by the operator, and no mainnet deployment happens without explicit authorization (`docs/adr/0004-token-development-staging.md`).

`test/` and `invariant/` subdirectories are reserved per the brief's mandatory testing requirements and are currently empty.
