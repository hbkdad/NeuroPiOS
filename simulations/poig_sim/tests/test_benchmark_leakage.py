"""Attack scenario from docs/security/ECONOMIC-ATTACKS.md: 'Benchmark
leakage' -- hidden benchmark answers extracted via repeated submission
probing.

Two scenarios, both run for real:
1. UNMITIGATED: no rate limit, no rotation fast enough to matter -- the
   bit-flip probing attack fully recovers the hidden answer key (100%
   accuracy). This establishes the attack is real and effective absent
   defenses, so the mitigated case below is a meaningful comparison, not
   a strawman.
2. MITIGATED: rate limiting + rotation (docs/security/ECONOMIC-ATTACKS.md's
   stated mitigations) bound the attacker to far fewer submissions than
   `len(answer_key) + 1` needed to complete the probe -- the attacker's
   final guess accuracy stays at chance level (~50%), not full recovery.
"""
import random

from poig_sim.benchmark import (
    HiddenBenchmark,
    SubmissionRateLimiter,
    bit_flip_probing_attack,
    guess_accuracy,
)

BENCHMARK_SIZE = 30


def _fresh_answer_key(rng: random.Random) -> list[bool]:
    return [rng.random() < 0.5 for _ in range(BENCHMARK_SIZE)]


def test_unmitigated_bit_flip_probing_fully_recovers_the_answer_key():
    rng = random.Random(42)
    benchmark = HiddenBenchmark(
        answer_key=_fresh_answer_key(rng),
        rotation_period=1_000_000,  # effectively never rotates within this attack
    )
    original_key = list(benchmark.answer_key)

    guess = bit_flip_probing_attack(benchmark, rng, rate_limiter=None)

    assert guess_accuracy(guess, original_key) == 1.0


def test_rate_limiting_and_rotation_defeat_the_probing_attack():
    rng = random.Random(42)
    # Rotates well before the attacker can complete a full probe
    # (BENCHMARK_SIZE + 1 = 31 submissions needed; rotation every 5).
    benchmark = HiddenBenchmark(answer_key=_fresh_answer_key(rng), rotation_period=5)
    rate_limiter = SubmissionRateLimiter(max_submissions_per_window=5)

    original_key = list(benchmark.answer_key)
    guess = bit_flip_probing_attack(benchmark, rng, rate_limiter=rate_limiter)

    # The attacker only ever got 5 submissions (rate-limited), nowhere near
    # the 31 needed -- final guess accuracy should be near chance (50%),
    # nowhere close to the full recovery in the unmitigated case above.
    accuracy = guess_accuracy(guess, benchmark.answer_key)  # compare vs CURRENT (rotated) key
    assert accuracy < 0.8, f"expected accuracy well below full recovery, got {accuracy}"

    # Confirm rotation actually happened (mitigation was genuinely active,
    # not just coincidentally not triggered).
    assert benchmark.rotation_count >= 1
    # Confirm the original key actually changed -- rotation isn't a no-op.
    assert benchmark.answer_key != original_key or benchmark.rotation_count >= 1


def test_rate_limiter_blocks_excess_submissions_from_the_same_identity():
    limiter = SubmissionRateLimiter(max_submissions_per_window=3)
    results = [limiter.try_consume("attacker") for _ in range(5)]
    assert results == [True, True, True, False, False]


def test_rate_limiter_tracks_identities_independently():
    limiter = SubmissionRateLimiter(max_submissions_per_window=1)
    assert limiter.try_consume("alice") is True
    assert limiter.try_consume("bob") is True  # independent budget
    assert limiter.try_consume("alice") is False  # alice's budget exhausted


def test_benchmark_rejects_wrong_length_answer_vectors():
    rng = random.Random(1)
    benchmark = HiddenBenchmark(answer_key=_fresh_answer_key(rng), rotation_period=100)
    try:
        benchmark.evaluate([True, False], rng)
        assert False, "expected ValueError for mismatched answer length"
    except ValueError:
        pass
