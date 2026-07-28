//! Hardware auto-detection, per docs/ARCHITECTURE.md's Node Architecture
//! section: OS, CPU, RAM, disk. GPU/VRAM detection is not included yet --
//! there is no portable, dependency-light way to enumerate GPU vendors
//! across CUDA/ROCm/Metal from a single crate without pulling in a much
//! larger dependency tree, and no GPU exists in this development container
//! to test against (docs/research/CAPABILITY-MATRIX.md). Deferred rather
//! than stubbed with fake values.

use sysinfo::System;

#[derive(Debug, Clone, PartialEq)]
pub struct HardwareProfile {
    pub os: String,
    pub hostname: String,
    pub cpu_cores: usize,
    pub total_memory_mb: u64,
    pub total_swap_mb: u64,
}

impl HardwareProfile {
    /// Queries the actual host via `sysinfo` -- not a fabricated/mocked
    /// value. In an environment with zero detectable CPUs (shouldn't
    /// happen on any real host) `cpu_cores` will legitimately read 0;
    /// callers should treat 0 as "detection failed", not "headless node".
    pub fn detect() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        HardwareProfile {
            os: System::long_os_version().unwrap_or_else(|| "unknown".to_string()),
            hostname: System::host_name().unwrap_or_else(|| "unknown".to_string()),
            cpu_cores: sys.cpus().len(),
            total_memory_mb: sys.total_memory() / (1024 * 1024),
            total_swap_mb: sys.total_swap() / (1024 * 1024),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_returns_plausible_values_on_a_real_host() {
        let profile = HardwareProfile::detect();
        // These assert against the ACTUAL container this test runs in --
        // not fabricated expectations. A container always has >=1 logical
        // CPU and >0 total memory; failing this means sysinfo itself
        // couldn't read /proc, which is a real environment problem worth
        // surfacing loudly rather than silently passing.
        assert!(
            profile.cpu_cores >= 1,
            "expected at least 1 detected CPU core"
        );
        assert!(
            profile.total_memory_mb > 0,
            "expected nonzero detected memory"
        );
        assert_ne!(profile.os, "");
    }
}
