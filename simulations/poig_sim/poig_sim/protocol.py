"""Simplified job-object protocol per docs/PROTOCOL.md.

NOT production cryptography. Identity.sign()/verify() use HMAC-SHA256 as a
deterministic stand-in for real Ed25519/secp256k1 signing so this package can
prove out the *economic* logic (escrow gating, replay protection via nonce)
without pulling in a crypto dependency. The real wire protocol and signature
scheme belong in crates/aion-protocol and crates/aion-crypto (Rust, using
reviewed libraries only, per AGENTS.md's "never invent cryptography" rule).
"""
from __future__ import annotations

import hashlib
import hmac
import secrets
import uuid
from dataclasses import dataclass, field


class ReplayError(Exception):
    pass


class SignatureError(Exception):
    pass


@dataclass
class Identity:
    """A simulated actor identity. `id` is public; `secret` never leaves this object."""

    identity_id: str
    secret: bytes = field(default_factory=lambda: secrets.token_bytes(32), repr=False)

    @staticmethod
    def create(identity_id: str | None = None) -> "Identity":
        return Identity(identity_id=identity_id or str(uuid.uuid4()))

    def sign(self, payload: bytes) -> str:
        return hmac.new(self.secret, payload, hashlib.sha256).hexdigest()


class NonceTracker:
    """Per-requester monotonic nonce tracking, per docs/PROTOCOL.md replay-protection rule."""

    def __init__(self) -> None:
        self._last_nonce: dict[str, int] = {}

    def check_and_advance(self, requester_id: str, nonce: int) -> None:
        last = self._last_nonce.get(requester_id, -1)
        if nonce <= last:
            raise ReplayError(
                f"nonce {nonce} <= last seen nonce {last} for requester {requester_id}"
            )
        self._last_nonce[requester_id] = nonce


@dataclass
class Job:
    """Canonical job object, simplified per docs/PROTOCOL.md."""

    job_id: str
    requester_id: str
    job_type: str
    base_value: float
    escrow_amount: float
    verification_policy: str
    nonce: int
    signature: str = ""

    CANONICAL_FIELDS = (
        "job_id",
        "requester_id",
        "job_type",
        "base_value",
        "escrow_amount",
        "verification_policy",
        "nonce",
    )

    def canonical_payload(self) -> bytes:
        parts = [str(getattr(self, f)) for f in self.CANONICAL_FIELDS]
        return "|".join(parts).encode("utf-8")


def create_job(
    requester: Identity,
    job_type: str,
    base_value: float,
    escrow_amount: float,
    verification_policy: str,
    nonce: int,
) -> Job:
    """Requester signs a new job. Escrow must be funded by the caller separately
    (see market.EscrowLedger) — signing alone does not move funds."""
    job = Job(
        job_id=str(uuid.uuid4()),
        requester_id=requester.identity_id,
        job_type=job_type,
        base_value=base_value,
        escrow_amount=escrow_amount,
        verification_policy=verification_policy,
        nonce=nonce,
    )
    job.signature = requester.sign(job.canonical_payload())
    return job


def verify_job(job: Job, requester: Identity, nonces: NonceTracker) -> None:
    """Raises SignatureError or ReplayError if the job is invalid.
    Nonce is only advanced if the signature is valid, so a bad signature can't
    burn a legitimate future nonce."""
    expected_sig = requester.sign(job.canonical_payload())
    if not hmac.compare_digest(expected_sig, job.signature):
        raise SignatureError(f"invalid signature on job {job.job_id}")
    nonces.check_and_advance(job.requester_id, job.nonce)
