# crates/aion-reputation — Multi-Dimensional Reputation + PoIG Scoring (planned, Phase 3+)

Rust implementation of the reward function in `docs/POIG-SPEC.md` and the multi-dimensional, non-transferable, decaying reputation store described in `docs/ARCHITECTURE.md` and `docs/security/THREAT-MODEL.md`. `simulations/poig_sim/poig_sim/poig.py` (Python, Phase 2) is the validated reference logic this crate should port and extend, not diverge from without an ADR. **Status: not started** (Python reference exists and is tested — see `simulations/poig_sim`).
