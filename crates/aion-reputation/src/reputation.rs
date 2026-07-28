//! Multi-dimensional, decaying, non-transferable reputation store, per
//! docs/security/THREAT-MODEL.md and docs/ARCHITECTURE.md. Ported from and
//! pinned against the validated Python reference at
//! `simulations/poig_sim/poig_sim/reputation.py`: reputation is never a
//! single global scalar in the underlying store (tracked per-dimension
//! here), is never purchasable/transferable (no API in this module moves a
//! dimension between identities), and decays over time when an identity is
//! inactive.

use std::collections::HashMap;

/// New identities start here: nonzero (so a first job is possible at all)
/// but well below an established honest worker's steady-state score, per
/// the "new/unproven worker gets max verification rate" design in
/// docs/VERIFICATION.md.
pub const BASELINE_SCORE: f64 = 0.15;
pub const DECAY_PER_TICK_IDLE: f64 = 0.01;
pub const MAX_SCORE: f64 = 1.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReputationRecord {
    pub completion_reliability: f64,
    pub verification_reliability: f64,
    pub jobs_completed: u64,
    pub disputes_lost: u64,
    pub last_active_tick: u64,
}

impl Default for ReputationRecord {
    fn default() -> Self {
        Self {
            completion_reliability: BASELINE_SCORE,
            verification_reliability: BASELINE_SCORE,
            jobs_completed: 0,
            disputes_lost: 0,
            last_active_tick: 0,
        }
    }
}

impl ReputationRecord {
    /// Bounded [0,1] collapse used as PoIG's `ReputationFactor`. The
    /// underlying dimensions remain separately tracked/queryable above.
    pub fn composite(&self) -> f64 {
        let base = 0.5 * self.completion_reliability + 0.5 * self.verification_reliability;
        let penalty = (0.1 * self.disputes_lost as f64).min(0.5);
        (base - penalty).clamp(0.0, MAX_SCORE)
    }
}

#[derive(Default)]
pub struct ReputationStore {
    records: HashMap<String, ReputationRecord>,
}

impl ReputationStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the identity's current record, creating a fresh
    /// (baseline-scored) one on first access -- matching the Python
    /// reference's `dict.setdefault` behavior.
    pub fn get(&mut self, identity_id: &str) -> ReputationRecord {
        *self.records.entry(identity_id.to_string()).or_default()
    }

    pub fn record_successful_job(&mut self, identity_id: &str, tick: u64, verified: bool) {
        let rec = self.records.entry(identity_id.to_string()).or_default();
        rec.jobs_completed += 1;
        rec.last_active_tick = tick;
        // Reliability grows toward 1.0 with diminishing returns
        // (asymptotic), never in one jump, so a single job can never
        // establish full trust.
        rec.completion_reliability += (MAX_SCORE - rec.completion_reliability) * 0.08;
        if verified {
            rec.verification_reliability += (MAX_SCORE - rec.verification_reliability) * 0.08;
        }
    }

    pub fn record_dispute_loss(&mut self, identity_id: &str, tick: u64) {
        let rec = self.records.entry(identity_id.to_string()).or_default();
        rec.disputes_lost += 1;
        rec.last_active_tick = tick;
    }

    pub fn decay(&mut self, identity_id: &str, current_tick: u64) {
        let rec = self.records.entry(identity_id.to_string()).or_default();
        let idle_ticks = current_tick.saturating_sub(rec.last_active_tick);
        if idle_ticks == 0 {
            return;
        }
        let raw = rec.completion_reliability - BASELINE_SCORE;
        let capped = DECAY_PER_TICK_IDLE * idle_ticks as f64;
        let decay = raw.min(capped).max(0.0);
        rec.completion_reliability = (rec.completion_reliability - decay).max(BASELINE_SCORE);
        rec.verification_reliability = (rec.verification_reliability - decay).max(BASELINE_SCORE);
    }

    pub fn reputation_factor(&mut self, identity_id: &str, current_tick: u64) -> f64 {
        self.decay(identity_id, current_tick);
        self.get(identity_id).composite()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_identity_starts_at_the_documented_baseline() {
        let mut store = ReputationStore::new();
        let rec = store.get("worker-a");
        assert_eq!(rec.completion_reliability, BASELINE_SCORE);
        assert_eq!(rec.verification_reliability, BASELINE_SCORE);
        assert_eq!(rec.jobs_completed, 0);
        assert_eq!(rec.disputes_lost, 0);
        // composite() at baseline: 0.5*0.15 + 0.5*0.15 - 0 = 0.15
        assert!((rec.composite() - BASELINE_SCORE).abs() < 1e-12);
    }

    #[test]
    fn successful_jobs_grow_reliability_asymptotically_never_in_one_jump() {
        let mut store = ReputationStore::new();
        store.record_successful_job("worker-a", 1, true);
        let after_one = store.get("worker-a").completion_reliability;
        // 0.15 + (1.0 - 0.15) * 0.08 = 0.218
        assert!((after_one - 0.218).abs() < 1e-9);
        assert!(after_one < MAX_SCORE);

        // Reputation compounds across further jobs but a single job can
        // never reach full trust, matching the Sybil-resistance argument
        // in test_sybil_cluster.py's Python reference.
        for _ in 0..50 {
            store.record_successful_job("worker-a", 1, true);
        }
        let after_many = store.get("worker-a").completion_reliability;
        assert!(after_many > after_one);
        assert!(after_many < MAX_SCORE);
    }

    #[test]
    fn later_jobs_by_a_persistent_identity_earn_a_strictly_higher_reputation_factor_than_the_first()
    {
        let mut store = ReputationStore::new();
        let first = store.reputation_factor("worker-a", 0);
        store.record_successful_job("worker-a", 0, true);
        for _ in 0..10 {
            store.record_successful_job("worker-a", 1, true);
        }
        let later = store.reputation_factor("worker-a", 1);
        assert!(later > first);
    }

    #[test]
    fn a_sybil_that_never_reuses_an_identity_is_pinned_at_baseline_every_time() {
        // The Sybil-resistance argument this store enables: a cluster
        // that mints a fresh identity per job never lets reputation grow,
        // regardless of how many jobs the cluster runs in total.
        let mut store = ReputationStore::new();
        for i in 0..20 {
            let id = format!("sybil-{i}");
            let factor = store.reputation_factor(&id, 0);
            assert!((factor - BASELINE_SCORE).abs() < 1e-12);
        }
    }

    #[test]
    fn disputes_lost_penalize_the_composite_score_capped_at_half() {
        let mut store = ReputationStore::new();
        for _ in 0..3 {
            store.record_successful_job("worker-a", 0, true);
        }
        let before = store.get("worker-a").composite();

        store.record_dispute_loss("worker-a", 0);
        let after_one_loss = store.get("worker-a").composite();
        assert!(after_one_loss < before);
        assert!((before - after_one_loss - 0.1).abs() < 1e-9);

        // Many dispute losses: penalty saturates at 0.5, never driving
        // the composite unboundedly negative.
        for _ in 0..20 {
            store.record_dispute_loss("worker-a", 0);
        }
        let after_many_losses = store.get("worker-a").composite();
        assert!(after_many_losses >= 0.0);
    }

    #[test]
    fn an_idle_identity_decays_toward_but_never_below_baseline() {
        let mut store = ReputationStore::new();
        for _ in 0..50 {
            store.record_successful_job("worker-a", 0, true);
        }
        let grown = store.get("worker-a").completion_reliability;
        assert!(grown > BASELINE_SCORE);

        // 1000 idle ticks is far more than enough to fully decay back to
        // baseline given DECAY_PER_TICK_IDLE = 0.01. Compared with an
        // epsilon, not exact equality: repeated floating-point
        // subtraction and `max()` clamping can leave a record a few ULPs
        // above BASELINE_SCORE rather than bit-identical to the constant.
        let decayed = store.reputation_factor("worker-a", 1000);
        let rec = store.get("worker-a");
        assert!((rec.completion_reliability - BASELINE_SCORE).abs() < 1e-9);
        assert!((rec.verification_reliability - BASELINE_SCORE).abs() < 1e-9);
        assert!((decayed - BASELINE_SCORE).abs() < 1e-9);
    }

    #[test]
    fn decay_is_a_no_op_for_an_identity_that_was_just_active() {
        let mut store = ReputationStore::new();
        store.record_successful_job("worker-a", 5, true);
        let before = store.get("worker-a").completion_reliability;
        store.decay("worker-a", 5); // current_tick == last_active_tick
        let after = store.get("worker-a").completion_reliability;
        assert_eq!(before, after);
    }

    #[test]
    fn reputation_is_never_transferable_between_identities() {
        // There is deliberately no API on ReputationStore that moves a
        // dimension from one identity to another -- the closest thing to
        // a regression test for that property is confirming two distinct
        // identities' records stay fully independent.
        let mut store = ReputationStore::new();
        for _ in 0..10 {
            store.record_successful_job("worker-a", 0, true);
        }
        let b = store.get("worker-b");
        assert_eq!(b.completion_reliability, BASELINE_SCORE);
        assert_eq!(b.jobs_completed, 0);
    }
}
