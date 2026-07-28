"""Griefing / dispute-cost-allocation model for the L4 optimistic dispute
mechanism, per docs/protocol/SLASHING-SPEC.md's "slashed funds follow a
defined policy: a portion compensates the disputing/harmed party, a portion
goes to protocol treasury" and docs/security/ECONOMIC-ATTACKS.md's
"Griefing" row, whose mitigation claims (until now unsimulated) that
"L4's bisection cost falls disproportionately on the losing party in a
dispute, disincentivizing frivolous challenges."

Honest scope note: this models the direct CAPITAL cost of raising a
dispute (challenger bond, forfeiture, compensation split) abstractly, the
same way verification.py models tier confidence abstractly -- there is no
real bisection arbitration to run (Phase 4, no execution pipeline exists).
It does NOT model latency/opportunity-cost griefing damage (a worker being
tied up in a dispute even when they ultimately win costs them real time
regardless of who receives the forfeited bond) -- that would need a
different, harder-to-quantify model and is explicitly NOT covered here,
the same way test_wash_trading.py explicitly does not cover the "inflate
apparent activity for optics" half of that attack.
"""
from __future__ import annotations

from dataclasses import dataclass


@dataclass
class DisputeEconomicsConfig:
    challenger_bond: float
    # Fraction of a FORFEITED (fraud-not-proven) bond that goes to the
    # disputed worker as compensation for being dragged into a dispute;
    # the remainder goes to protocol treasury. A simulation-tunable
    # parameter, per SLASHING-SPEC -- not fixed by this module.
    worker_compensation_fraction: float
    fraud_proven_worker_slash: float
    fraud_proven_challenger_reward: float


@dataclass
class GriefingCampaignResult:
    rounds: int
    challenger_net_pnl: float
    worker_net_pnl: float
    treasury_net_intake: float


def simulate_dispute_campaign(
    config: DisputeEconomicsConfig,
    rounds: int,
    worker_actually_cheats_fraction: float = 0.0,
) -> GriefingCampaignResult:
    """Runs `rounds` disputes raised by a single challenger against a single
    worker. `worker_actually_cheats_fraction` controls what fraction of
    those disputes are actually legitimate (fraud really occurred) -- 0.0
    models PURE GRIEFING (a challenger disputing every job with no
    informational edge, purely to impose cost), while 1.0 models a
    challenger who is always right, as a control case proving the
    mechanism correctly rewards genuine challenges rather than just
    punishing all disputes indiscriminately.
    """
    if not 0.0 <= worker_actually_cheats_fraction <= 1.0:
        raise ValueError("worker_actually_cheats_fraction must be in [0, 1]")

    challenger_pnl = 0.0
    worker_pnl = 0.0
    treasury = 0.0
    fraud_rounds = int(round(rounds * worker_actually_cheats_fraction))

    for i in range(rounds):
        fraud_actually_occurred = i < fraud_rounds
        challenger_pnl -= config.challenger_bond
        if fraud_actually_occurred:
            challenger_pnl += config.challenger_bond + config.fraud_proven_challenger_reward
            worker_pnl -= config.fraud_proven_worker_slash
        else:
            worker_compensation = config.challenger_bond * config.worker_compensation_fraction
            worker_pnl += worker_compensation
            treasury += config.challenger_bond - worker_compensation

    return GriefingCampaignResult(
        rounds=rounds,
        challenger_net_pnl=challenger_pnl,
        worker_net_pnl=worker_pnl,
        treasury_net_intake=treasury,
    )
