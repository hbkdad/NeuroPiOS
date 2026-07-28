"""Hidden/rotating benchmark model, per docs/PROTOCOL.md's Benchmark object
and docs/POIG-SPEC.md's Proof of Improvement anti-overfitting requirements.

This module exists specifically to give docs/security/ECONOMIC-ATTACKS.md's
"Benchmark leakage" row real simulation coverage instead of leaving it as
design-time-only analysis.
"""
from __future__ import annotations

import random
from dataclasses import dataclass, field


@dataclass
class HiddenBenchmark:
    """A benchmark whose answer key is hidden -- only an AGGREGATE score is
    ever returned to a submitter, per docs/PROTOCOL.md's anti-leakage
    policy field. Rotates its answer key after `rotation_period`
    submissions, so accumulated partial knowledge from probing goes stale.
    """

    answer_key: list[bool]
    rotation_period: int
    _submissions_since_rotation: int = field(default=0, repr=False)
    _rotation_count: int = field(default=0, repr=False)

    def evaluate(self, answers: list[bool], rng: random.Random) -> float:
        if len(answers) != len(self.answer_key):
            raise ValueError("answer vector length must match benchmark size")
        correct = sum(1 for a, b in zip(answers, self.answer_key) if a == b)
        score = correct / len(self.answer_key)

        self._submissions_since_rotation += 1
        if self._submissions_since_rotation >= self.rotation_period:
            self._rotate(rng)
        return score

    def _rotate(self, rng: random.Random) -> None:
        self.answer_key = [rng.random() < 0.5 for _ in self.answer_key]
        self._submissions_since_rotation = 0
        self._rotation_count += 1

    @property
    def rotation_count(self) -> int:
        return self._rotation_count


@dataclass
class SubmissionRateLimiter:
    """Per-identity submission rate limiting, per
    docs/security/ECONOMIC-ATTACKS.md's stated mitigation
    ("rate-limited submission attempts per identity")."""

    max_submissions_per_window: int
    _counts: dict[str, int] = field(default_factory=dict)

    def try_consume(self, identity_id: str) -> bool:
        used = self._counts.get(identity_id, 0)
        if used >= self.max_submissions_per_window:
            return False
        self._counts[identity_id] = used + 1
        return True

    def reset(self) -> None:
        self._counts.clear()


def bit_flip_probing_attack(
    benchmark: HiddenBenchmark,
    rng: random.Random,
    *,
    rate_limiter: SubmissionRateLimiter | None = None,
    identity_id: str = "attacker",
) -> list[bool]:
    """A realistic aggregate-score-only extraction attack: submit a
    baseline guess, then flip one bit at a time and use the resulting
    score delta to infer each hidden bit -- classic oracle probing via a
    side channel (the aggregate score), not a brute-force guess of the
    whole vector at once. Needs `len(answer_key) + 1` submissions to fully
    recover the key when nothing stops it.

    Returns the attacker's best current guess at the hidden answer key
    (which may be incomplete/wrong if it got rate-limited or the benchmark
    rotated mid-attack).
    """
    n = len(benchmark.answer_key)
    guess = [True] * n

    def submit(answers: list[bool]) -> float | None:
        if rate_limiter is not None and not rate_limiter.try_consume(identity_id):
            return None
        return benchmark.evaluate(answers, rng)

    baseline_score = submit(guess)
    if baseline_score is None:
        return guess  # rate-limited before even a baseline read

    for i in range(n):
        flipped = list(guess)
        flipped[i] = not flipped[i]
        score = submit(flipped)
        if score is None:
            break  # rate-limited mid-attack; guess stays partially inferred
        if score > baseline_score:
            # flipping bit i improved the score -> our original guess for
            # bit i was wrong; adopt the flip.
            guess[i] = not guess[i]
            baseline_score = score
        # if score <= baseline, keep current guess for bit i (it was already right,
        # or flipping made it worse either way that's consistent with "keep it").

    return guess


def guess_accuracy(guess: list[bool], answer_key: list[bool]) -> float:
    correct = sum(1 for a, b in zip(guess, answer_key) if a == b)
    return correct / len(answer_key)
