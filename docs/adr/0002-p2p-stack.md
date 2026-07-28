# ADR 0002: P2P Stack — rust-libp2p (Tokio)

## Status
Accepted, and implemented through milestone 5: `crates/aion-p2p` has a working GossipSub+Identify+Kademlia swarm over TCP+Noise+Yamux, with real peer-scoring active on the job-announcements topic (explicit, deliberately-tuned `TopicScoreParams`, not library defaults), two-peer publish/receive, and Kademlia FIND_NODE round-trip integration tests, and `crates/aion-node`'s `Node::build_swarm()` drives a swarm built from the node's own bridged identity (`tests/node_gossip.rs` in `aion-node`). AutoNAT and circuit relay (NAT traversal), connection limits, and 3+ node relayed Kademlia discovery (attempted, root-caused as far as time allowed, documented as a known issue) remain unresolved — see `crates/aion-p2p/README.md` for exactly what's done vs. deferred.

## Context
AION's node network needs peer discovery, NAT traversal, gossip-based job/announcement propagation, and a DHT for content/model lookup. The orchestration brief's default hypothesis is Rust + Tokio + rust-libp2p unless research shows a stronger alternative.

## Options considered
1. **rust-libp2p** — actively maintained (2026 docs current), has Kademlia DHT, GossipSub, Identify, AutoNAT, circuit-relay/hole-punching, Noise transport security, QUIC and TCP transports, and a peer-scoring API for GossipSub — covers essentially every requirement listed in the brief's P2P research section natively.
2. **go-libp2p** — equally mature (it's the reference/most battle-tested implementation, used by IPFS/Filecoin/Ethereum consensus clients), but would fragment the stack from the rest of AION's Rust node/runtime crates and forces a cross-language FFI boundary or a separate service process.
3. **js-libp2p** — best fit only if the node itself needed to run in-browser; AION's node roles (compute/storage/validator) are not browser-hosted, so this is not the right fit for the node layer (though relevant later for any browser-based lightweight peer in the web app).

## Decision
Use **rust-libp2p** on **Tokio**, matching the orchestration brief's default. Use GossipSub for job/announcement pub-sub, Kademlia for peer/content discovery (paired with Identify, since GossipSub does not do discovery on its own), Noise for transport encryption, and QUIC as primary transport with TCP fallback. AutoNAT + circuit relay for NAT traversal/hole punching.

## Reasoning
- Single-language node stack: `aion-node`, `aion-p2p`, `aion-runtime`, `aion-verifier` all in Rust means no FFI boundary and one build/test/security-audit surface.
- All required primitives (GossipSub, Kademlia, Noise, QUIC, AutoNAT, relay, peer scoring, connection limits/resource manager) are first-party rust-libp2p crates as of 2026, not third-party bolt-ons.
- go-libp2p's maturity edge doesn't outweigh the stack-fragmentation cost given rust-libp2p already covers every listed requirement.

## Consequences
- AION inherits rust-libp2p's release cadence and any of its unresolved issues; pin versions explicitly and track the GitHub advisory feed for this dependency (per supply-chain requirements in `SECURITY.md`).
- Peer-scoring and resource-manager configuration (connection limits, per-peer rate limiting) must be tuned deliberately in `crates/aion-p2p` to satisfy the anti-spam/gossip-flood requirements in the threat model — not left at defaults.
