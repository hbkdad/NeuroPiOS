"""Attack scenario from docs/security/ECONOMIC-ATTACKS.md: 'Treasury
drain' -- governance or admin-key compromise draining the protocol
treasury.

Assertions under test, matching the mitigation column's claimed design
("multisig + timelock + pause capability"):
1. A weighted-authorization scheme alone has the same majority-collusion
   limitation already proven for L6 quorum (test_validator_collusion.py):
   a minority of compromised signing weight cannot even PROPOSE a drain,
   but a compromised majority can -- no weighting scheme prevents that by
   construction.
2. A timelock + independent guardian veto is a genuinely different,
   additional layer: it can stop an otherwise-fully-authorized drain, but
   only if actually exercised before the window expires -- both the real
   value and the real limit of this mitigation are tested, not just the
   flattering half.
"""
import pytest

from poig_sim.treasury import MultisigTreasury, Signer, TreasuryError


def build_signers(honest_weight: float, compromised_weight: float) -> list[Signer]:
    return [
        Signer("honest-1", honest_weight),
        Signer("attacker-1", compromised_weight),
    ]


def test_minority_compromised_weight_cannot_even_propose_a_drain():
    signers = build_signers(honest_weight=8.0, compromised_weight=2.0)
    treasury = MultisigTreasury(balance=1000.0, signers=signers, threshold_fraction=0.5)

    with pytest.raises(TreasuryError):
        treasury.propose_drain(500.0, authorizing_signer_ids=["attacker-1"])

    # The treasury balance is untouched -- the proposal was rejected
    # outright, not queued and later blocked.
    assert treasury.balance == 1000.0
    assert treasury.pending_drain is None


def test_majority_compromised_weight_can_propose_and_without_a_timelock_drains_instantly():
    # Documenting the known limitation, not hiding it: a colluding
    # MAJORITY of signing weight authorizes and, with zero timelock,
    # drains the treasury on the very next tick -- the same "no
    # weighting scheme prevents a majority" finding already proven for
    # L6 quorum, now shown for treasury authorization specifically.
    signers = build_signers(honest_weight=2.0, compromised_weight=8.0)
    treasury = MultisigTreasury(
        balance=1000.0, signers=signers, threshold_fraction=0.5, timelock_rounds=0
    )

    treasury.propose_drain(1000.0, authorizing_signer_ids=["attacker-1"])
    treasury.tick()

    assert treasury.balance == 0.0
    assert treasury.pending_drain is None


def test_timelock_gives_a_veto_window_that_can_stop_an_authorized_drain():
    # The genuinely different, additional defensive layer: even with a
    # compromised majority authorizing the drain, an independent guardian
    # (not one of the weighted signers -- a security-council/community
    # role) can veto it during the timelock window before it executes.
    signers = build_signers(honest_weight=2.0, compromised_weight=8.0)
    treasury = MultisigTreasury(
        balance=1000.0, signers=signers, threshold_fraction=0.5, timelock_rounds=3
    )

    treasury.propose_drain(1000.0, authorizing_signer_ids=["attacker-1"])
    treasury.tick()  # round 1 of 3
    treasury.tick()  # round 2 of 3
    treasury.veto()  # caught before round 3 elapses

    assert treasury.pending_drain is None
    assert treasury.balance == 1000.0  # fully preserved


def test_a_drain_executes_if_the_timelock_expires_without_a_veto():
    # Control case: the timelock is a genuine, bounded window, not a
    # silent no-op "protection" -- if nobody vetoes in time, the
    # authorized drain still executes exactly as it would without a
    # timelock at all.
    signers = build_signers(honest_weight=2.0, compromised_weight=8.0)
    treasury = MultisigTreasury(
        balance=1000.0, signers=signers, threshold_fraction=0.5, timelock_rounds=2
    )

    treasury.propose_drain(1000.0, authorizing_signer_ids=["attacker-1"])
    treasury.tick()  # round 1 of 2
    treasury.tick()  # round 2 of 2 -- executes now
    treasury.tick()  # extra tick: idempotent no-op, nothing pending

    assert treasury.balance == 0.0
    assert treasury.pending_drain is None


def test_cannot_propose_draining_more_than_the_treasury_balance():
    signers = build_signers(honest_weight=2.0, compromised_weight=8.0)
    treasury = MultisigTreasury(balance=500.0, signers=signers, threshold_fraction=0.5)

    with pytest.raises(TreasuryError):
        treasury.propose_drain(500.01, authorizing_signer_ids=["attacker-1"])


def test_cannot_veto_when_nothing_is_pending():
    signers = build_signers(honest_weight=8.0, compromised_weight=2.0)
    treasury = MultisigTreasury(balance=1000.0, signers=signers, threshold_fraction=0.5)

    with pytest.raises(TreasuryError):
        treasury.veto()
