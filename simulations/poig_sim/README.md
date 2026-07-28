# poig_sim — AION Phase 2 Economic Simulator

The first executable proof-of-concept for AION's PoIG (Proof of Intelligence Gain) reward mechanism, per `docs/ROADMAP.md` Phase 2 ("local simulator, no blockchain"). Implements the logic specified in `docs/POIG-SPEC.md` and `docs/ECONOMIC-MODEL.md`, and exercises ten attack scenarios from `docs/security/ECONOMIC-ATTACKS.md`.

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
- L4 dispute-cost-allocation economics (`poig_sim/dispute_economics.py`), modeling challenger bond forfeiture and its split between the disputed worker and protocol treasury, per `docs/protocol/SLASHING-SPEC.md`.
- A weighted-authorization treasury with a timelock/veto mechanism (`poig_sim/treasury.py`), modeling the multisig + timelock + guardian-veto design described (not yet built) for the Phase 10 contract layer.

## What this is NOT
Not a P2P network, not a blockchain, not connected to any real inference runtime, and not modeling multi-agent off-protocol collusion economics (side payments, cartel formation dynamics) — those remain design-time analysis in `docs/security/ECONOMIC-ATTACKS.md`, explicitly not claimed as simulated.

## Running it

```
cd simulations/poig_sim
pip install -e .
pytest -q                 # 37 tests
python scale_sweep.py     # Monte Carlo scale sweep, prints measured timing
```

## What the tests actually prove

- `tests/test_poig_basic.py` — the reward formula behaves as specified for a straightforward honest, escrow-backed, verified job (sanity baseline).
- `tests/test_self_dealing.py` — an actor who creates a job paid to itself, with no real independent escrow-backed requester, gets `Reward == 0` (the `UsefulWorkScore` gate holds).
- `tests/test_sybil_cluster.py` — a cluster of freshly-created identities issuing each other low-value jobs to bootstrap reputation cannot out-earn an established honest worker of equal real output, because new identities face the verification-rate floor at its maximum and contribute no real external demand signal.
- `tests/test_validator_collusion.py` — commit/reveal prevents vote-copying; a **minority** colluding validator cluster cannot flip a quorum outcome; a **majority** colluding cluster *can* (honestly documented as a real residual risk, not hidden).
- `tests/test_price_manipulation.py` — a worker that manipulates only its price signal cannot outscore an established honest worker on routing, and single-signal leverage is mathematically capped at that signal's own configured weight share.
- `tests/test_benchmark_leakage.py` — a real aggregate-score-only bit-flip probing attack fully recovers a hidden 30-item benchmark answer key when unmitigated (100% accuracy, proving the attack is real), but rate limiting + rotation together keep the same attacker's final accuracy well below full recovery (chance-level, typically 25-70% across tested seeds) by rotating the answer key out from under them before they can complete the probe.
- `tests/test_stake_concentration.py` — a 2-of-10 headcount-minority validator cluster with concentrated stake (weight 40 vs. the honest majority's 8) dominates a quorum outcome, while the identical headcount split under equal weights does not; a weight sweep confirms the flip point tracks the whales' aggregate weight share crossing 50%, not their headcount — weight, not headcount, is the real security parameter, and this remains a genuinely open design gap (not resolved by quorum weighting alone).
- `tests/test_wash_trading.py` — two distinct sock-puppet identities (not the same self-dealing gate — a genuinely different attack) cycling capital through many rounds of jobs cannot net-extract more cumulative reward than they commit, even at the attacker's most favorable settings (maxed `DemandFactor`, matured reputation, L4 passing every round): the reward/escrow ratio stays well under 1.0 (asymptotically bounded by `VerificationConfidence`'s L4 ceiling). The reward formula's multiplicative structure, not graph analysis, is what prevents direct profit here — honestly scoped to not claim this also prevents the "fake apparent activity/volume for optics" half of wash trading, which is a separate, still-unmitigated concern.
- `tests/test_griefing.py` — a challenger who disputes every job from a target worker with no informational edge (pure griefing) loses their entire bond every round regardless of how the forfeited bond is split between the worker and protocol treasury, while a challenger who is genuinely always right (a control case, not griefing) is profitable — confirming the mechanism penalizes frivolous disputing specifically, not disputing in general. Honestly scoped: covers the direct capital cost only, not latency/opportunity-cost griefing damage, and documents that a worker-compensation split of 0 leaves the griefed worker with no direct payout at all.
- `tests/test_treasury_drain.py` — a weighted-authorization scheme alone has the same majority-collusion limitation already proven for L6 quorum: a minority of compromised signing weight cannot even propose a treasury drain, but a compromised majority can, and without a timelock it executes instantly (documented residual risk, not hidden). A timelock + independent guardian veto is a genuinely different, additional mitigation: it can stop an otherwise-fully-authorized drain if exercised within the window, but a control test confirms the timelock is a real bounded window — an unvetoed drain still executes once it expires, not a silent no-op "protection."

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
