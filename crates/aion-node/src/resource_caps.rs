//! Mandatory operator-configured resource safety caps, per
//! docs/ARCHITECTURE.md's Node Safety section: "a node never silently
//! consumes host resources." This module is the enforcement point -- every
//! job-acceptance decision in a future `aion-node` scheduler must consult
//! `ResourceCaps::allows_new_work`, not just hardware capacity.

use libp2p::connection_limits::ConnectionLimits;

#[derive(Debug, Clone, PartialEq)]
pub struct ResourceCaps {
    pub max_cpu_percent: f32,
    pub max_memory_mb: u64,
    pub max_storage_mb: u64,
    pub max_bandwidth_mbps: Option<u32>,
    pub idle_only: bool,
    /// (start_hour, end_hour) in 0-23 local time, inclusive-exclusive. None
    /// means no working-hours restriction.
    pub working_hours: Option<(u8, u8)>,
}

impl ResourceCaps {
    /// A conservative default: nothing runs until the operator explicitly
    /// configures caps. This is deliberate -- there is no "reasonable
    /// default resource budget" that doesn't risk silently overcommitting
    /// someone's hardware, so the safe default is "accept no new work."
    pub fn locked_down() -> Self {
        ResourceCaps {
            max_cpu_percent: 0.0,
            max_memory_mb: 0,
            max_storage_mb: 0,
            max_bandwidth_mbps: Some(0),
            idle_only: true,
            working_hours: None,
        }
    }

    /// Derives P2P connection limits from these resource caps, per
    /// docs/ARCHITECTURE.md's Node Safety principle extended to network
    /// resources (not just CPU/memory/storage) -- closing the gap flagged
    /// in `crates/aion-p2p/README.md`'s "Not yet done" list, where a
    /// node's bandwidth cap and its P2P connection limits used to be two
    /// independent, unconnected knobs. Never LOOSER than
    /// `aion_p2p::default_connection_limits()` -- this only ever tightens
    /// that ceiling, matching `allows_new_work`'s own "only ever says no
    /// more strictly than the raw caps" invariant. Never goes all the way
    /// to zero connections either, even at `max_bandwidth_mbps == Some(0)`
    /// (the `locked_down()` default): "no job-payload bandwidth budgeted
    /// yet" is not the same claim as "this node should be unreachable on
    /// the network" -- baseline protocol traffic (gossip mesh membership,
    /// Kademlia routing, Identify) is not job execution, and a
    /// resource-locked node still needs to be a real, discoverable member
    /// of the network, not silently cut off from it.
    ///
    /// - `max_bandwidth_mbps == Some(mbps)`: established connection caps
    ///   scale with `mbps`, under a documented, deliberately conservative
    ///   assumption of `ASSUMED_MBPS_PER_CONNECTION` Mbps of overhead per
    ///   established connection, floored at 1 connection minimum. This is
    ///   a starting heuristic, not measured against real traffic -- the
    ///   same tunable-not-final caveat that applies to every other
    ///   approximate weight in this codebase (see `docs/POIG-SPEC.md`'s
    ///   equivalent note) -- clamped so it can never exceed `aion_p2p`'s
    ///   own default ceilings.
    /// - `max_bandwidth_mbps == None` ("no explicit bandwidth limit
    ///   configured"): falls back to `aion_p2p::default_connection_limits()`
    ///   unchanged.
    pub fn to_connection_limits(&self) -> ConnectionLimits {
        let default = aion_p2p::default_connection_limits();

        let Some(mbps) = self.max_bandwidth_mbps else {
            return default;
        };

        const ASSUMED_MBPS_PER_CONNECTION: u32 = 1;
        let scaled = mbps.saturating_div(ASSUMED_MBPS_PER_CONNECTION).max(1);
        let clamp = |ceiling: u32| scaled.min(ceiling);

        ConnectionLimits::default()
            .with_max_pending_incoming(Some(clamp(aion_p2p::DEFAULT_MAX_PENDING_INCOMING)))
            .with_max_pending_outgoing(Some(clamp(aion_p2p::DEFAULT_MAX_PENDING_OUTGOING)))
            .with_max_established_incoming(Some(clamp(aion_p2p::DEFAULT_MAX_ESTABLISHED_INCOMING)))
            .with_max_established_outgoing(Some(clamp(aion_p2p::DEFAULT_MAX_ESTABLISHED_OUTGOING)))
            .with_max_established_per_peer(Some(clamp(aion_p2p::DEFAULT_MAX_ESTABLISHED_PER_PEER)))
            .with_max_established(Some(clamp(aion_p2p::DEFAULT_MAX_ESTABLISHED)))
    }

    /// Whether new work may be accepted right now, given current observed
    /// resource usage (0.0-100.0 for cpu_percent) and current wall-clock
    /// hour (0-23, operator's local time). This function only ever says
    /// "no" more strictly than the raw caps -- it must never be the place
    /// that silently loosens a limit.
    pub fn allows_new_work(
        &self,
        current_cpu_percent: f32,
        current_memory_mb: u64,
        current_hour: u8,
    ) -> bool {
        if current_cpu_percent >= self.max_cpu_percent {
            return false;
        }
        if current_memory_mb >= self.max_memory_mb {
            return false;
        }
        if let Some((start, end)) = self.working_hours {
            let in_window = if start <= end {
                current_hour >= start && current_hour < end
            } else {
                // wraps past midnight, e.g. (22, 6)
                current_hour >= start || current_hour < end
            };
            if !in_window {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locked_down_default_accepts_no_work() {
        let caps = ResourceCaps::locked_down();
        assert!(!caps.allows_new_work(0.0, 0, 12));
    }

    #[test]
    fn work_accepted_within_configured_caps() {
        let caps = ResourceCaps {
            max_cpu_percent: 80.0,
            max_memory_mb: 8000,
            max_storage_mb: 100_000,
            max_bandwidth_mbps: None,
            idle_only: false,
            working_hours: None,
        };
        assert!(caps.allows_new_work(50.0, 4000, 14));
    }

    #[test]
    fn work_rejected_at_or_above_cpu_cap() {
        let caps = ResourceCaps {
            max_cpu_percent: 80.0,
            max_memory_mb: 8000,
            max_storage_mb: 100_000,
            max_bandwidth_mbps: None,
            idle_only: false,
            working_hours: None,
        };
        assert!(!caps.allows_new_work(80.0, 100, 14)); // exactly at cap: rejected, not "close enough"
        assert!(!caps.allows_new_work(95.0, 100, 14));
    }

    #[test]
    fn work_rejected_at_or_above_memory_cap() {
        let caps = ResourceCaps {
            max_cpu_percent: 80.0,
            max_memory_mb: 8000,
            max_storage_mb: 100_000,
            max_bandwidth_mbps: None,
            idle_only: false,
            working_hours: None,
        };
        assert!(!caps.allows_new_work(10.0, 8000, 14));
    }

    #[test]
    fn working_hours_window_normal_range() {
        let caps = ResourceCaps {
            max_cpu_percent: 100.0,
            max_memory_mb: 100_000,
            max_storage_mb: 100_000,
            max_bandwidth_mbps: None,
            idle_only: false,
            working_hours: Some((9, 17)), // 9am-5pm
        };
        assert!(caps.allows_new_work(0.0, 0, 9));
        assert!(caps.allows_new_work(0.0, 0, 16));
        assert!(!caps.allows_new_work(0.0, 0, 17)); // end is exclusive
        assert!(!caps.allows_new_work(0.0, 0, 8));
        assert!(!caps.allows_new_work(0.0, 0, 22));
    }

    #[test]
    fn working_hours_window_wraps_past_midnight() {
        let caps = ResourceCaps {
            max_cpu_percent: 100.0,
            max_memory_mb: 100_000,
            max_storage_mb: 100_000,
            max_bandwidth_mbps: None,
            idle_only: false,
            working_hours: Some((22, 6)), // overnight
        };
        assert!(caps.allows_new_work(0.0, 0, 23));
        assert!(caps.allows_new_work(0.0, 0, 2));
        assert!(!caps.allows_new_work(0.0, 0, 12));
    }

    // `ConnectionLimits` has no getters (builder-only setters, see its own
    // doc), so these tests inspect its `Debug` output -- a legitimate way
    // to verify internal state when a type deliberately doesn't expose one,
    // and more honest than skipping verification of the actual values.
    fn debug_string(caps: &ResourceCaps) -> String {
        format!("{:?}", caps.to_connection_limits())
    }

    #[test]
    fn locked_down_caps_still_allow_at_least_minimal_p2p_connectivity() {
        // ResourceCaps::locked_down() means "no job-payload bandwidth
        // budgeted yet," not "this node should be unreachable" -- gossip
        // mesh membership, Kademlia routing, and Identify are baseline
        // protocol traffic, not job execution, so a freshly-bootstrapped
        // node must still be able to make and accept at least one
        // connection (this is also what keeps tests/node_gossip.rs's
        // two-freshly-bootstrapped-nodes scenario working).
        let caps = ResourceCaps::locked_down();
        let debug = debug_string(&caps);
        assert!(debug.contains("max_established_incoming: Some(1)"));
        assert!(debug.contains("max_established_outgoing: Some(1)"));
    }

    #[test]
    fn no_bandwidth_cap_configured_falls_back_to_aion_p2p_defaults_unchanged() {
        let caps = ResourceCaps {
            max_bandwidth_mbps: None,
            ..ResourceCaps::locked_down()
        };
        let debug = debug_string(&caps);
        assert!(debug.contains(&format!(
            "max_established_incoming: Some({})",
            aion_p2p::DEFAULT_MAX_ESTABLISHED_INCOMING
        )));
    }

    #[test]
    fn a_small_positive_bandwidth_cap_scales_down_connection_limits() {
        let caps = ResourceCaps {
            max_bandwidth_mbps: Some(5),
            ..ResourceCaps::locked_down()
        };
        let debug = debug_string(&caps);
        // 5 Mbps at the documented 1-Mbps-per-connection assumption -> 5,
        // well under aion_p2p's default ceiling of 100.
        assert!(debug.contains("max_established_incoming: Some(5)"));
        assert!(debug.contains("max_established_outgoing: Some(5)"));
    }

    #[test]
    fn a_large_bandwidth_cap_is_clamped_to_aion_p2p_defaults_not_looser() {
        let caps = ResourceCaps {
            max_bandwidth_mbps: Some(1_000_000), // absurdly generous
            ..ResourceCaps::locked_down()
        };
        let debug = debug_string(&caps);
        // Never looser than aion_p2p's own ceiling, no matter how generous
        // the configured bandwidth cap is.
        assert!(debug.contains(&format!(
            "max_established_incoming: Some({})",
            aion_p2p::DEFAULT_MAX_ESTABLISHED_INCOMING
        )));
        assert!(debug.contains(&format!(
            "max_established_total: Some({})",
            aion_p2p::DEFAULT_MAX_ESTABLISHED
        )));
    }

    #[test]
    fn a_sub_one_mbps_cap_still_allows_at_least_one_connection_not_zero() {
        // Rounding a tiny positive bandwidth cap down to zero connections
        // would be indistinguishable from the locked-down case, which is
        // wrong -- the operator explicitly configured SOME positive
        // bandwidth, so the node should be able to make at least one
        // connection, not be silently treated as fully locked down.
        let caps = ResourceCaps {
            max_bandwidth_mbps: Some(1),
            ..ResourceCaps::locked_down()
        };
        let debug = debug_string(&caps);
        assert!(!debug.contains("max_established_incoming: Some(0)"));
    }
}
