# crates/aion-protocol — Wire Protocol Types

Rust implementation of the canonical job object, model manifest, nonce-based replay protection, and the job-lifecycle state machine specified in `docs/PROTOCOL.md`. This crate is the actual wire-format authority — the Python reference in `simulations/poig_sim/poig_sim/protocol.py` (Phase 2) is a semantics reference for the economic simulator only, not the wire-format source of truth.

**Status: real implementation, milestone 2.** `cargo test -p aion-protocol` → 17/17 passing:

- `job.rs` — canonical job object + deterministic canonical-payload encoding (the bytes a requester signs), with a regression test pinning the exact byte layout so a field reorder is caught immediately (would otherwise be a silent breaking wire-format change).
- `nonce.rs` — per-requester monotonic nonce tracking / replay protection, including the "a rejected nonce doesn't mutate state" property.
- `lifecycle.rs` — the job-lifecycle state machine, encoding `docs/security/THREAT-MODEL.md`'s core invariant directly in the type system: `Settled` and `Slashed` are terminal, no transition is permitted out of either, ever (a job cannot settle twice).
- `model_manifest.rs` — content-addressed model manifest struct, plus a cheap local self-referential-parent check (full multi-hop provenance-DAG cycle detection belongs to a future model registry, not this struct).

**Not yet done:** actual signature verification (belongs to `crates/aion-crypto`, composed by a future caller — this crate intentionally has no crypto dependency), benchmark object schema, wire-format serialization beyond the canonical signing payload (e.g. an actual binary/CBOR encoding for P2P transport, per `docs/PROTOCOL.md`'s open items).
