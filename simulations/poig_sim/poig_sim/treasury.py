"""Treasury drain attack model, per docs/security/ECONOMIC-ATTACKS.md's
"Treasury drain" row: "Multisig + timelock + pause capability at the
contract layer (Phase 10, not yet built)". This models that MECHANISM
DESIGN in the abstract, the same way verification.py/quorum.py model
protocol logic before the corresponding on-chain contract exists (Phase
10, contracts/ has no code yet).

Honest scope note: this is a weighted-authorization + timelock/veto state
machine, not a real multisig smart contract -- no on-chain execution, no
real signatures. What it DOES prove, concretely rather than just asserted
in prose: a weighted-authorization scheme alone (no timelock) has the
same majority-collusion limitation already proven for L6 quorum
(quorum.py/aion-verifier's quorum.rs) -- no weighting scheme prevents a
colluding majority by construction. A timelock + independent guardian veto
is a genuinely DIFFERENT, additional defensive layer (not just more
quorum weighting) that can stop an otherwise-fully-authorized drain, but
only if the veto is actually exercised before the timelock expires --
this module proves both the mitigation's real value and its real limit
(a drain that goes unvetoed within the window still executes), not just
the flattering half.
"""
from __future__ import annotations

from dataclasses import dataclass


class TreasuryError(Exception):
    pass


@dataclass
class Signer:
    id: str
    weight: float


class MultisigTreasury:
    def __init__(
        self,
        balance: float,
        signers: list[Signer],
        threshold_fraction: float,
        timelock_rounds: int = 0,
    ):
        self.balance = balance
        self._signers = {s.id: s for s in signers}
        self._total_weight = sum(s.weight for s in signers)
        self.threshold_fraction = threshold_fraction
        self.timelock_rounds = timelock_rounds
        self.pending_drain: dict | None = None

    def _authorized_weight(self, authorizing_signer_ids: list[str]) -> float:
        return sum(
            self._signers[i].weight for i in authorizing_signer_ids if i in self._signers
        )

    def propose_drain(self, amount: float, authorizing_signer_ids: list[str]) -> None:
        """Proposes a drain of `amount`, authorized by the given signer IDs.
        Raises if the authorizing weight doesn't meet the threshold, or if
        the amount exceeds the treasury's current balance -- a proposal
        that could never legitimately execute is rejected outright, not
        silently queued.
        """
        if amount > self.balance:
            raise TreasuryError("cannot propose draining more than the treasury balance")
        weight_fraction = self._authorized_weight(authorizing_signer_ids) / self._total_weight
        if weight_fraction < self.threshold_fraction:
            raise TreasuryError(
                f"insufficient authorization weight ({weight_fraction:.2f}) to propose a drain "
                f"(threshold {self.threshold_fraction:.2f})"
            )
        self.pending_drain = {"amount": amount, "rounds_remaining": self.timelock_rounds}

    def veto(self) -> None:
        """An independent guardian (deliberately NOT one of the weighted
        signers -- a security-council/community-veto role, per the
        Compound-Timelock-style pattern this mirrors) cancels a pending
        drain. Only meaningful while the timelock window is still open;
        once `tick()` has executed the drain, there is nothing left to
        veto.
        """
        if self.pending_drain is None:
            raise TreasuryError("no pending drain to veto")
        self.pending_drain = None

    def tick(self) -> None:
        """Advances one round of the timelock. Executes the pending drain
        once its window has fully elapsed with no veto -- proving the
        timelock is a genuine, bounded delay/veto window, not a silent
        no-op "protection" that never actually lets the drain through.
        """
        if self.pending_drain is None:
            return
        if self.pending_drain["rounds_remaining"] <= 0:
            self.balance -= self.pending_drain["amount"]
            self.pending_drain = None
        else:
            self.pending_drain["rounds_remaining"] -= 1
