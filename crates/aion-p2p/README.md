# crates/aion-p2p — P2P Networking

rust-libp2p wrapper per `docs/adr/0002-p2p-stack.md`.

**Status: real implementation, milestone 5.** `cargo test -p aion-p2p` → 10/10 passing (8 unit + 2 real network integration tests):

- **GossipSub** pub/sub over real TCP+Noise+Yamux, message signing required (`ValidationMode::Strict`, `MessageAuthenticity::Signed`) — anonymous/unsigned gossip is not permitted, since every AION gossip message must be attributable to a peer for the anti-spam/reputation layers in `docs/security/THREAT-MODEL.md`. **Peer scoring is active** (`with_peer_score`, milestone 5), with an explicit, deliberately-tuned `TopicScoreParams` for the job-announcements topic (`job_announcements_topic_score_params`) — not left as unconfigured library defaults, per `docs/adr/0002-p2p-stack.md`'s requirement that every gossip topic get its own anti-spam policy. Rewards mesh tenure and first-message delivery; penalizes under-delivery and (heavily, quadratically) invalid messages. Values are a documented starting point, not yet tuned against real network data — same caveat as every other weight in this codebase (see `docs/POIG-SPEC.md`).
- **Identify**, wired in alongside GossipSub and Kademlia via a combined `AionBehaviour` (`#[derive(NetworkBehaviour)]`).
- **Kademlia**, on a custom protocol name (`/aion/kad/0.1.0`, deliberately distinct from the default `/ipfs/kad/1.0.0` so an AION node never accidentally joins or pollutes the public IPFS DHT), explicitly set to **server mode** (`kad::Mode::Server`) at construction — the library default is client mode, which silently refuses to answer other peers' FIND_NODE requests until an external address is confirmed via AutoNAT (not implemented yet), which would make every AION node a DHT free-rider and break discovery entirely. `feed_identify_into_kademlia` bridges addresses learned via Identify into Kademlia's routing table (the standard libp2p integration pattern — Kademlia does not do this automatically).
- **Identity bridging**: `keypair_from_identity` constructs a `libp2p::identity::Keypair` from the exact same Ed25519 key material as an `aion_crypto::Identity` (verified by a test that checks the raw public-key bytes match, not just that the derivation is deterministic) — a node's P2P PeerID is genuinely the same key as its `aion_crypto::Identity`, not two unrelated keys.
- `tests/gossip_roundtrip.rs` — two independent swarms, real TCP loopback connection, manual dial, one peer's published message is actually received by the other, byte-for-byte (~0.5s), with peer scoring active throughout.
- `tests/kademlia_discovery.rs` — a real Kademlia FIND_NODE request/response round-trips successfully end-to-end over `/aion/kad/0.1.0`. Honestly scoped: in this 2-node network the response legitimately comes back empty (a FIND_NODE responder returns peers from its own routing table closer than itself, and a fresh 2-node network has none yet) — what's proven is the protocol round-trip works, not that discovery finds anyone in a network this small.

**Not yet done (explicitly deferred, not hidden):**
- **AutoNAT / circuit relay** (NAT traversal) — not implemented; peers currently need directly dialable addresses.
- **Connection limits** — peer-scoring is active but the `libp2p_connection_limits` behaviour is not yet added to `AionBehaviour`; a peer could still open unbounded connections before scoring has a chance to catch up.
- **`app_specific_weight`** in peer scoring is unused — reserved for `crates/aion-reputation`'s multi-dimensional reputation score to plug in once that crate exists as real Rust (currently only the Phase 2 Python simulator).
- 3+ node relayed Kademlia discovery — see the Known Issue below.

## Known issue: 3-node relayed Kademlia discovery (attempted, not working, milestone 4)

A genuine attempt was made at a 3-node test (`peer C discovers peer A through peer B, without ever dialing A directly`) — the actual proof needed for the Phase 3 MVP criterion "three machines discover each other." It does not currently work, and the failure was root-caused as far as time allowed rather than left as a mystery:

- B (queried by C) correctly computes the closer-peer list server-side — confirmed directly via `kad::Event::InboundRequest { request: FindNode { num_closer_peers: 1 } }`, i.e. B's routing table lookup for A succeeds and B decides to send A's record back to C.
- C's query nonetheless completes with an empty peer list, and C's own routing table never gains an entry for A afterward — even under retry, and even with B's own swarm kept polled throughout (ruling out a "stale unpolled connection" theory).
- This points to the response (or C's decoding of it) being lost or rejected somewhere between B sending it and C's query algorithm recording it — not a timeout (failures happen within ~100ms, far under any configured timeout) and not a routing-table population issue (confirmed populated on B's side before the query).

This was not chased further into `libp2p-kad`'s internals (would need protocol-level tracing/logging to pin down definitively) in order to keep making forward progress elsewhere. **Next session working on this crate should start here** — the working 2-node request/response test (`tests/kademlia_discovery.rs`) plus this diagnostic trail is the starting point, not a fresh investigation.
