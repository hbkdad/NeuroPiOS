# poig_sim — AION Phase 2 Economic Simulator

The first executable proof-of-concept for AION's PoIG (Proof of Intelligence Gain) reward mechanism, per `docs/ROADMAP.md` Phase 2 ("local simulator, no blockchain"). Implements the logic specified in `docs/POIG-SPEC.md` and `docs/ECONOMIC-MODEL.md`, and exercises two of the attack scenarios from `docs/security/ECONOMIC-ATTACKS.md`.

## What this is
An in-memory (no network, no chain, no real inference) model of:
- Signed, nonce-protected job objects with escrow (`poig_sim/protocol.py`) — a simplified stand-in for the real cryptographic signing that will live in `crates/aion-crypto`; **not production cryptography**, deliberately, since this package only needs to prove the economic logic is sound.
- Multi-dimensional, decaying, non-transferable reputation (`poig_sim/reputation.py`).
- Adaptive per-tier verification confidence and rate (`poig_sim/verification.py`), implementing the "never fully eliminate verification" floor from `docs/VERIFICATION.md`.
- The PoIG reward formula and its `UsefulWorkScore` anti-farming gate (`poig_sim/poig.py`).
- A minimal job market / actor simulation (`poig_sim/market.py`).

## What this is NOT
Not a P2P network, not a blockchain, not connected to any real inference runtime, and not the full 100/1,000/10,000/100,000-node Monte Carlo sweep the orchestration brief specifies for complete economic validation — that is explicitly future work (see `docs/ROADMAP.md`).

## Running it

```
cd simulations/poig_sim
pip install -e .
pytest -q
```

## What the tests actually prove

- `tests/test_poig_basic.py` — the reward formula behaves as specified for a straightforward honest, escrow-backed, verified job (sanity baseline).
- `tests/test_self_dealing.py` — an actor who creates a job paid to itself, with no real independent escrow-backed requester, gets `Reward == 0` (the `UsefulWorkScore` gate holds).
- `tests/test_sybil_cluster.py` — a cluster of freshly-created identities issuing each other low-value jobs to bootstrap reputation cannot out-earn an established honest worker of equal real output, because new identities face the verification-rate floor at its maximum and contribute no real external demand signal.
