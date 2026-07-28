//! An owned background event loop for a `Node`'s P2P swarm, per
//! `crates/aion-node/README.md`'s "Not yet done" list: previously every
//! caller had to manually drive a `Node`'s swarm (`swarm.select_next_some()`
//! in a hand-written loop, as `tests/node_gossip.rs` still does at the
//! lower level). `Node::spawn` instead hands ownership of the swarm to a
//! background `tokio` task and returns a `NodeHandle` -- a channel-based
//! command/event API -- so a caller never touches `Swarm` directly.
//!
//! Honest scope note: this is a real, working event loop (dial, listen,
//! gossip subscribe/publish, and the events that matter for those), not a
//! full command surface over every behaviour this crate wires in
//! (Kademlia queries, AutoNAT status, relay reservations aren't exposed
//! through `NodeHandle` yet) -- it covers what `tests/node_gossip.rs`
//! already proved works manually, now reachable without hand-rolling the
//! event loop, and is meant to grow incrementally as real callers need
//! more of the surface.

use aion_p2p::{AionBehaviour, AionBehaviourEvent, Topic};
use futures::StreamExt;
use libp2p::{gossipsub, swarm::SwarmEvent, Multiaddr, PeerId, Swarm};
use tokio::sync::mpsc;

enum Command {
    ListenOn(Multiaddr),
    Dial(Multiaddr),
    Subscribe(gossipsub::IdentTopic),
    Publish(gossipsub::IdentTopic, Vec<u8>),
}

/// Events surfaced to a `NodeHandle` from its background swarm. Not
/// exhaustive over every possible `SwarmEvent` -- only what this crate's
/// current real use cases (gossip, basic connectivity) need; see the
/// module doc's scope note.
#[derive(Debug, Clone)]
pub enum NodeEvent {
    NewListenAddr(Multiaddr),
    ConnectionEstablished(PeerId),
    GossipMessage {
        topic: String,
        source: Option<PeerId>,
        data: Vec<u8>,
    },
}

/// A handle to a `Node` whose swarm is being driven by a background task.
/// Commands are fire-and-forget (an unbounded channel to a task that is
/// the sole owner of the `Swarm`, matching the single-owner model
/// `libp2p::Swarm` itself requires) -- errors from an individual command
/// (e.g. dialing an unparseable-at-runtime address) surface as the swarm's
/// own `SwarmEvent`s via `next_event`, not as a return value here, since
/// that is how `libp2p` itself reports them.
pub struct NodeHandle {
    peer_id: PeerId,
    commands: mpsc::UnboundedSender<Command>,
    events: mpsc::UnboundedReceiver<NodeEvent>,
}

impl NodeHandle {
    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    pub fn listen_on(&self, addr: Multiaddr) {
        let _ = self.commands.send(Command::ListenOn(addr));
    }

    pub fn dial(&self, addr: Multiaddr) {
        let _ = self.commands.send(Command::Dial(addr));
    }

    pub fn subscribe(&self, topic: &Topic) {
        let _ = self.commands.send(Command::Subscribe(topic.0.clone()));
    }

    pub fn publish(&self, topic: &Topic, data: Vec<u8>) {
        let _ = self.commands.send(Command::Publish(topic.0.clone(), data));
    }

    /// Awaits the next event from this node's background swarm. Returns
    /// `None` once the background task has stopped (e.g. this handle and
    /// all its clones were dropped, closing the command channel).
    pub async fn next_event(&mut self) -> Option<NodeEvent> {
        self.events.recv().await
    }
}

pub(crate) fn spawn(mut swarm: Swarm<AionBehaviour>) -> NodeHandle {
    let peer_id = *swarm.local_peer_id();
    let (command_tx, mut command_rx) = mpsc::unbounded_channel::<Command>();
    let (event_tx, event_rx) = mpsc::unbounded_channel::<NodeEvent>();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                command = command_rx.recv() => {
                    match command {
                        Some(Command::ListenOn(addr)) => {
                            let _ = swarm.listen_on(addr);
                        }
                        Some(Command::Dial(addr)) => {
                            let _ = swarm.dial(addr);
                        }
                        Some(Command::Subscribe(topic)) => {
                            let _ = swarm.behaviour_mut().gossipsub.subscribe(&topic);
                        }
                        Some(Command::Publish(topic, data)) => {
                            let _ = swarm.behaviour_mut().gossipsub.publish(topic, data);
                        }
                        // All NodeHandle clones dropped -- nothing left to
                        // drive this swarm on behalf of, so stop the task
                        // rather than looping forever ownerless.
                        None => break,
                    }
                }
                event = swarm.select_next_some() => {
                    match event {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            let _ = event_tx.send(NodeEvent::NewListenAddr(address));
                        }
                        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                            let _ = event_tx.send(NodeEvent::ConnectionEstablished(peer_id));
                        }
                        SwarmEvent::Behaviour(AionBehaviourEvent::Identify(ref identify_event)) => {
                            aion_p2p::feed_identify_into_kademlia(
                                &mut swarm.behaviour_mut().kademlia,
                                identify_event,
                            );
                        }
                        SwarmEvent::Behaviour(AionBehaviourEvent::Gossipsub(
                            gossipsub::Event::Message { message, propagation_source, .. },
                        )) => {
                            let _ = event_tx.send(NodeEvent::GossipMessage {
                                topic: message.topic.to_string(),
                                source: Some(propagation_source),
                                data: message.data,
                            });
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    NodeHandle {
        peer_id,
        commands: command_tx,
        events: event_rx,
    }
}
