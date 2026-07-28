"""Monte Carlo scale sweep for the PoIG economic simulator.

Per the orchestration brief's Economic Simulation requirement: simulate
increasing node counts (100 / 1,000 / 10,000 / 100,000) with a mixed actor
population (honest worker, cheap/low-effort worker, premium worker,
malicious/self-dealing worker, Sybil cluster) and report real measured
results -- not assumed ones.

This is a single-process, pure-Python, in-memory sweep (no real network, no
real chain) -- it measures whether the *economic logic* (poig.py,
reputation.py, verification.py) scales sanely in node/job count and whether
the anti-fraud gate holds at scale, not real network throughput. Run it
directly: `python scale_sweep.py`.
"""
from __future__ import annotations

import random
import time

from poig_sim.market import Market
from poig_sim.protocol import Identity

random.seed(1234)  # deterministic across runs, for reproducibility


def run_sweep(n_actors: int) -> dict:
    market = Market()
    start = time.perf_counter()

    honest_workers = [f"honest-{i}" for i in range(max(1, n_actors // 4))]
    cheap_workers = [f"cheap-{i}" for i in range(max(1, n_actors // 4))]
    premium_workers = [f"premium-{i}" for i in range(max(1, n_actors // 8))]
    malicious_workers = [f"malicious-{i}" for i in range(max(1, n_actors // 10))]
    sybil_workers = [f"sybil-{i}" for i in range(max(1, n_actors // 10))]

    totals = {
        "honest": 0.0,
        "cheap": 0.0,
        "premium": 0.0,
        "malicious_self_dealing": 0.0,
        "sybil": 0.0,
    }
    jobs_run = 0
    nonce = 0

    def run_one(pool: list[str], key: str, self_dealing: bool, base_value: float, verified: bool) -> None:
        nonlocal jobs_run, nonce
        worker_id = random.choice(pool)
        if self_dealing:
            requester = Identity.create(worker_id)  # attacker requests its own job
            actual_worker = worker_id
        else:
            requester = Identity.create(f"requester-for-{worker_id}-{nonce}")
            actual_worker = worker_id
        outcome = market.submit_and_settle_job(
            requester=requester,
            worker_id=actual_worker,
            job_type="inference",
            base_value=base_value,
            escrow_amount=base_value,
            verification_policy="L4",
            nonce=nonce,
            fund_escrow=True,
            verification_passed=verified,
        )
        nonce += 1
        jobs_run += 1
        totals[key] += outcome.reward.reward

    # Each actor pool gets a handful of jobs; kept small per-actor so total
    # job count stays proportional to n_actors instead of exploding.
    jobs_per_actor = 3
    for _ in range(jobs_per_actor):
        for _ in honest_workers:
            run_one(honest_workers, "honest", self_dealing=False, base_value=10.0, verified=True)
        for _ in cheap_workers:
            run_one(cheap_workers, "cheap", self_dealing=False, base_value=2.0, verified=True)
        for _ in premium_workers:
            run_one(premium_workers, "premium", self_dealing=False, base_value=50.0, verified=True)
        for _ in malicious_workers:
            # malicious actors attempt self-dealing every time
            run_one(malicious_workers, "malicious_self_dealing", self_dealing=True, base_value=1000.0, verified=True)
        for _ in sybil_workers:
            run_one(sybil_workers, "sybil", self_dealing=False, base_value=1.0, verified=True)

    elapsed = time.perf_counter() - start
    return {
        "n_actors": n_actors,
        "jobs_run": jobs_run,
        "elapsed_seconds": elapsed,
        "jobs_per_second": jobs_run / elapsed if elapsed > 0 else float("inf"),
        "totals": totals,
    }


if __name__ == "__main__":
    for scale in (100, 1_000, 10_000, 100_000):
        result = run_sweep(scale)
        print(
            f"n_actors={result['n_actors']:>7} "
            f"jobs_run={result['jobs_run']:>7} "
            f"elapsed={result['elapsed_seconds']:.3f}s "
            f"({result['jobs_per_second']:.0f} jobs/s)"
        )
        for k, v in result["totals"].items():
            print(f"    {k:>24}: {v:.2f}")
        # The core anti-fraud invariant, checked at every scale, not just
        # small ones: self-dealing never earns anything, regardless of how
        # many attackers or how large their claimed escrow.
        assert result["totals"]["malicious_self_dealing"] == 0.0, (
            f"UsefulWorkScore gate failed to hold at n_actors={scale}"
        )
    print("\nAll scales: malicious self-dealing total reward == 0.0 (gate held at every scale).")
