//! L2 tier: random replicated execution, per docs/VERIFICATION.md and the
//! numeric-tolerance-band concept in docs/protocol/SLASHING-SPEC.md.
//! Compares N workers' results for the same job within a relative
//! tolerance, distinguishing "numerical nondeterminism" (within
//! tolerance, not slashable) from "diverging/wrong result" (an outlier,
//! a candidate for the SLASHING-SPEC's dispute process -- outlier status
//! ALONE is explicitly not sufficient grounds for a slash per that spec;
//! this module only identifies divergence, it does not decide fault).

#[derive(Debug, Clone, PartialEq)]
pub struct ReplicatedResult {
    pub worker_id: String,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplicationOutcome {
    /// Mean of the largest cluster of mutually-agreeing results.
    pub consensus_value: f64,
    pub agreeing_worker_ids: Vec<String>,
    pub outlier_worker_ids: Vec<String>,
}

impl ReplicationOutcome {
    /// Fraction of results that agreed with the consensus cluster -- used
    /// as an input to `docs/POIG-SPEC.md`'s `VerificationConfidence`
    /// alongside the L2 tier's base confidence (this is NOT the tier
    /// confidence itself, just the agreement fraction within it).
    pub fn agreement_fraction(&self) -> f64 {
        let total = self.agreeing_worker_ids.len() + self.outlier_worker_ids.len();
        if total == 0 {
            return 0.0;
        }
        self.agreeing_worker_ids.len() as f64 / total as f64
    }
}

fn within_tolerance(a: f64, b: f64, relative_tolerance: f64) -> bool {
    if a == b {
        return true;
    }
    let denom = a.abs().max(b.abs()).max(f64::EPSILON);
    (a - b).abs() / denom <= relative_tolerance
}

/// Clusters replicated results by mutual tolerance and returns the
/// largest cluster as consensus. Returns `None` for an empty result set
/// (nothing to compare -- a caller error, not a "job failed" signal).
pub fn evaluate_replicated_results(
    results: &[ReplicatedResult],
    relative_tolerance: f64,
) -> Option<ReplicationOutcome> {
    if results.is_empty() {
        return None;
    }

    let mut best_cluster: Vec<usize> = Vec::new();
    for candidate in results {
        let cluster: Vec<usize> = results
            .iter()
            .enumerate()
            .filter(|(_, other)| within_tolerance(candidate.value, other.value, relative_tolerance))
            .map(|(j, _)| j)
            .collect();
        if cluster.len() > best_cluster.len() {
            best_cluster = cluster;
        }
    }

    let consensus_value =
        best_cluster.iter().map(|&i| results[i].value).sum::<f64>() / best_cluster.len() as f64;
    let agreeing_worker_ids = best_cluster
        .iter()
        .map(|&i| results[i].worker_id.clone())
        .collect();
    let outlier_worker_ids = results
        .iter()
        .enumerate()
        .filter(|(i, _)| !best_cluster.contains(i))
        .map(|(_, r)| r.worker_id.clone())
        .collect();

    Some(ReplicationOutcome {
        consensus_value,
        agreeing_worker_ids,
        outlier_worker_ids,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result(worker_id: &str, value: f64) -> ReplicatedResult {
        ReplicatedResult {
            worker_id: worker_id.to_string(),
            value,
        }
    }

    #[test]
    fn unanimous_agreement_has_no_outliers() {
        let results = vec![result("a", 1.0), result("b", 1.0), result("c", 1.0)];
        let outcome = evaluate_replicated_results(&results, 0.01).unwrap();
        assert_eq!(outcome.consensus_value, 1.0);
        assert!(outcome.outlier_worker_ids.is_empty());
        assert_eq!(outcome.agreement_fraction(), 1.0);
    }

    #[test]
    fn a_single_divergent_result_is_flagged_as_an_outlier() {
        let results = vec![result("a", 1.0), result("b", 1.0), result("c", 50.0)];
        let outcome = evaluate_replicated_results(&results, 0.01).unwrap();
        assert_eq!(outcome.outlier_worker_ids, vec!["c".to_string()]);
        assert_eq!(outcome.agreeing_worker_ids.len(), 2);
    }

    #[test]
    fn small_numerical_nondeterminism_within_tolerance_is_not_an_outlier() {
        // 1.0 vs 1.0005 is a 0.05% relative difference -- well within a 1%
        // tolerance band, representing legitimate hardware/kernel
        // floating-point variance per docs/protocol/SLASHING-SPEC.md, not
        // a wrong result.
        let results = vec![result("a", 1.0), result("b", 1.0005), result("c", 0.9998)];
        let outcome = evaluate_replicated_results(&results, 0.01).unwrap();
        assert!(outcome.outlier_worker_ids.is_empty());
    }

    #[test]
    fn a_result_just_outside_tolerance_is_flagged() {
        let results = vec![result("a", 1.0), result("b", 1.0), result("c", 1.5)]; // 50% off
        let outcome = evaluate_replicated_results(&results, 0.01).unwrap();
        assert_eq!(outcome.outlier_worker_ids, vec!["c".to_string()]);
    }

    #[test]
    fn empty_results_return_none_not_a_false_consensus() {
        assert_eq!(evaluate_replicated_results(&[], 0.01), None);
    }

    #[test]
    fn a_minority_split_still_picks_the_largest_cluster_as_consensus() {
        let results = vec![
            result("a", 1.0),
            result("b", 1.0),
            result("c", 1.0),
            result("d", 2.0),
            result("e", 2.0),
        ];
        let outcome = evaluate_replicated_results(&results, 0.01).unwrap();
        assert_eq!(outcome.consensus_value, 1.0);
        assert_eq!(outcome.outlier_worker_ids.len(), 2);
    }
}
