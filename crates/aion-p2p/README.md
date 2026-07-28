# crates/aion-p2p — P2P Networking

rust-libp2p wrapper per `docs/adr/0002-p2p-stack.md`.

**Status: real implementation, milestone 4.** `cargo test -p aion-p2p` → 7/7 passing (5 unit + 2 real network integration tests):

- **GossipSub** pub/sub over real TCP+Noise+Yamux, message signing required (`ValidationMode::Strict`, `MessageAuthenticity::Signed`) — anonymous/unsigned gossip is not permitted, since every AION gossip message must be attributable to a peer for the anti-spam/reputation layers in `docs/security/THREAT-MODEL.md`.
- **Identify**, wired in alongside GossipSub and Kademlia via a combined `AionBehaviour` (`#[derive(NetworkBehaviour)]`).
- **Kademlia**, on a custom protocol name (`/aion/kad/0.1.0`, deliberately distinct from the default `/ipfs/kad/1.0.0` so an AION node never accidentally joins or pollutes the public IPFS DHT). `feed_identify_into_kademlia` bridges addresses learned via Identify into Kademlia's routing table (the standard libp2p integration pattern — Kademlia does not do this automatically).
- **Identity bridging**: `keypair_from_identity` constructs a `libp2p::identity::Keypair` from the exact same Ed25519 key material as an `aion_crypto::Identity` (verified by a test that checks the raw public-key bytes match, not just that the derivation is deterministic) — a node's P2P PeerID is genuinely the same key as its `aion_crypto::Identity`, not two unrelated keys.
- `tests/gossip_roundtrip.rs` — two independent swarms, real TCP loopback connection, manual dial, one peer's published message is actually received by the other, byte-for-byte (~0.5s).
- `tests/kademlia_discovery.rs` — a real Kademlia FIND_NODE request/response round-trips successfully end-to-end over `/aion/kad/0.1.0`. Honestly scoped: in this 2-node network the response legitimately comes back empty (a FIND_NODE responder returns peers from its own routing table closer than itself, and a fresh 2-node network has none yet) — what's proven is the protocol round-trip works, not that discovery finds anyone in a network this small. A 3+ node "peer C discovers peer A via peer B, without dialing A directly" test is the natural follow-up.

**Not yet done (explicitly deferred, not hidden):**
- **AutoNAT / circuit relay** (NAT traversal) — not implemented; peers currently need directly dialable addresses.
- **Peer scoring / connection limits** (anti-spam) — `docs/adr/0002-p2p-stack.md`'s requirement that "every new gossip topic gets a peer-scoring/rate-limit policy defined alongside it" is not yet satisfied beyond gossipsub's own defaults. Must be addressed before this crate is used on an open, adversarial network.
- **3+ node indirect Kademlia discovery** (see `tests/kademlia_discovery.rs`'s scope note above).
