# poig_sim — AION Phase 2 Economic Simulator

The first executable proof-of-concept for AION's PoIG (Proof of Intelligence Gain) reward mechanism, per `docs/ROADMAP.md` Phase 2 ("local simulator, no blockchain"). Implements the logic specified in `docs/POIG-SPEC.md` and `docs/ECONOMIC-MODEL.md`, and exercises six attack scenarios from `docs/security/ECONOMIC-ATTACKS.md`.

## What this is
An in-memory (no network, no chain, no real inference) model of:
- Signed, nonce-protected job objects with escrow (`poig_sim/protocol.py`) — a simplified stand-in for the real cryptographic signing that will live in `crates/aion-crypto`; **not production cryptography**, deliberately, since this package only needs to prove the economic logic is sound.
- Multi-dimensional, decaying, non-transferable reputation (`poig_sim/reputation.py`).
- Adaptive per-tier verification confidence and rate (`poig_sim/verification.py`), implementing the "never fully eliminate verification" floor from `docs/VERIFICATION.md`.
- Commit/reveal multi-validator quorum aggregation (`poig_sim/quorum.py`), per the L6 tier and anti-copy-voting requirement in `docs/VERIFICATION.md`.
- The deterministic Routing Score (`poig_sim/routing.py`), per `docs/ARCHITECTURE.md`.
- The PoIG reward formula and its `UsefulWorkScore` anti-farming gate (`poig_sim/poig.py`).
- A hidden/rotating benchmark model with rate limiting (`poig_sim/benchmark.py`), per `docs/PROTOCOL.md`'s Benchmark object anti-leakage policy.
- A minimal job market / actor simulation (`poig_sim/market.py`).
- A Monte Carlo scale sweep (`scale_sweep.py`) measuring how the market simulation actually performs at increasing node counts.

## What this is NOT
Not a P2P network, not a blockchain, not connected to any real inference runtime, and not modeling multi-agent off-protocol collusion economics (side payments, cartel formation dynamics) — those remain design-time analysis in `docs/security/ECONOMIC-ATTACKS.md`, explicitly not claimed as simulated.

## Running it

```
cd simulations/poig_sim
pip install -e .
pytest -q                 # 19 tests
python scale_sweep.py     # Monte Carlo scale sweep, prints measured timing
```

## What the tests actually prove

- `tests/test_poig_basic.py` — the reward formula behaves as specified for a straightforward honest, escrow-backed, verified job (sanity baseline).
- `tests/test_self_dealing.py` — an actor who creates a job paid to itself, with no real independent escrow-backed requester, gets `Reward == 0` (the `UsefulWorkScore` gate holds).
- `tests/test_sybil_cluster.py` — a cluster of freshly-created identities issuing each other low-value jobs to bootstrap reputation cannot out-earn an established honest worker of equal real output, because new identities face the verification-rate floor at its maximum and contribute no real external demand signal.
- `tests/test_validator_collusion.py` — commit/reveal prevents vote-copying; a **minority** colluding validator cluster cannot flip a quorum outcome; a **majority** colluding cluster *can* (honestly documented as a real residual risk, not hidden).
- `tests/test_price_manipulation.py` — a worker that manipulates only its price signal cannot outscore an established honest worker on routing, and single-signal leverage is mathematically capped at that signal's own configured weight share.
- `tests/test_benchmark_leakage.py` — a real aggregate-score-only bit-flip probing attack fully recovers a hidden 30-item benchmark answer key when unmitigated (100% accuracy, proving the attack is real), but rate limiting + rotation together keep the same attacker's final accuracy well below full recovery (chance-level, typically 25-70% across tested seeds) by rotating the answer key out from under them before they can complete the probe.

## Scale sweep results

Actual measured output from this environment (2026-07-28, `python scale_sweep.py`; re-run to reproduce — timing is environment-dependent, treat it as a scaling signal for the economic logic itself, not a claim about real network throughput):

```
n_actors=    100 jobs_run=    246 elapsed=0.006s (43779 jobs/s)
n_actors=   1000 jobs_run=   2475 elapsed=0.056s (43885 jobs/s)
n_actors=  10000 jobs_run=  24750 elapsed=0.586s (42269 jobs/s)
n_actors= 100000 jobs_run= 247500 elapsed=6.877s (35991 jobs/s)

All scales: malicious self-dealing total reward == 0.0 (gate held at every scale).
```

All four scales (100 / 1,000 / 10,000 / 100,000 actors) specified in the orchestration brief ran to completion in this container in ~7.6s total wall-clock. The `UsefulWorkScore` anti-self-dealing gate held exactly (`0.0` reward to the malicious/self-dealing pool) at every scale, not just in the small hand-written unit tests. This validates the *economic logic's* correctness and its performance characteristics as pure Python at these job-count scales — it says nothing about real P2P/consensus/settlement latency, which doesn't exist yet (Phases 3+).
