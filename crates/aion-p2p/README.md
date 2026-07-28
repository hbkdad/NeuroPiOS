# crates/aion-p2p — P2P Networking

rust-libp2p wrapper per `docs/adr/0002-p2p-stack.md`.

**Status: real implementation (first pass), milestone 3.** `cargo test -p aion-p2p` → 4/4 passing (3 unit + 1 real two-peer integration test):

- GossipSub pub/sub over **real TCP+Noise+Yamux**, message signing required (`ValidationMode::Strict`, `MessageAuthenticity::Signed`) — anonymous/unsigned gossip is not permitted, since every AION gossip message must be attributable to a peer for the anti-spam/reputation layers in `docs/security/THREAT-MODEL.md`.
- Identify protocol wired in alongside GossipSub via a combined `AionBehaviour` (`#[derive(NetworkBehaviour)]`).
- `tests/gossip_roundtrip.rs` — **a genuine two-peer integration test**: two independent swarms, real TCP loopback connection, manual dial (no discovery layer yet), both subscribe to the job-announcements topic, one peer's published message is actually received by the other, byte-for-byte. Runs in ~0.5s.

**Not yet done (explicitly deferred, not hidden):**
- **Kademlia DHT peer discovery** — peers must currently be dialed with a known address; no discovery layer yet. Combining Kademlia into `AionBehaviour` alongside GossipSub+Identify is real additional work (a third behaviour in the derive macro, plus bootstrap/routing-table logic) scoped as a follow-up rather than rushed into this pass.
- **AutoNAT / circuit relay** (NAT traversal) — not implemented; this crate currently only works when peers have directly dialable addresses (e.g. same LAN, or public IPs).
- **Peer scoring / connection limits** (anti-spam) — `docs/adr/0002-p2p-stack.md`'s requirement that "every new gossip topic gets a peer-scoring/rate-limit policy defined alongside it" is not yet satisfied; the single `JOB_ANNOUNCEMENTS_TOPIC` currently has no scoring params configured beyond gossipsub's own defaults. Must be addressed before this crate is used on an open, adversarial network.
- **Wiring into `crates/aion-node`** — `Node` doesn't yet own a running `Swarm`.
- The P2P identity keypair here is a fresh `libp2p::identity::Keypair` (Ed25519), not yet unified with `aion_crypto::Identity`'s key material — see `src/lib.rs`'s module doc for why that bridging was deliberately deferred rather than forced.
