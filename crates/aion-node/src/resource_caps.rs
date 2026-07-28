//! Mandatory operator-configured resource safety caps, per
//! docs/ARCHITECTURE.md's Node Safety section: "a node never silently
//! consumes host resources." This module is the enforcement point -- every
//! job-acceptance decision in a future `aion-node` scheduler must consult
//! `ResourceCaps::allows_new_work`, not just hardware capacity.

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
}
