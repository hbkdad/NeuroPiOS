//! L6 tier: commit/reveal multi-validator weighted quorum, per
//! docs/VERIFICATION.md and docs/adr/0003-verification-tiering.md. Ported
//! from and pinned against the validated Python reference at
//! simulations/poig_sim/poig_sim/quorum.py, but using REAL cryptographic
//! commit/reveal (`aion_crypto::commit_reveal`, SHA-256 via the audited
//! `sha2` crate) rather than that module's simplified string-concat hash
//! -- an improvement made possible by `aion-crypto` already existing as a
//! real dependency here, which it didn't when the Python reference was
//! written.
//!
//! Honest scope note (same as the Python reference): this module
//! demonstrates that quorum weighting resists a MINORITY colluding
//! validator cluster (docs/security/ECONOMIC-ATTACKS.md, "Collusion
//! (validator cartel)" row). It does NOT claim to prevent a colluding
//! MAJORITY from flipping an outcome -- no quorum-weighting scheme can,
//! by construction, since majority agreement is the mechanism's whole
//! basis for trust. That residual risk is documented, not hidden -- see
//! this module's own tests and docs/security/THREAT-MODEL.md.

use aion_crypto::{commit, verify_reveal, Commitment};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum QuorumError {
    #[error("cannot reveal until every validator in this quorum has committed")]
    RevealBeforeAllCommitted,
    #[error("revealed vote does not match earlier commit for validator {0} -- cannot change a vote after committing")]
    CommitMismatch(String),
    #[error("no validator revealed a vote in this quorum")]
    NoRevealedVotes,
    #[error("unknown validator id: {0}")]
    UnknownValidator(String),
}

struct Ballot {
    weight: f64,
    commitment: Option<Commitment>,
    revealed_vote: Option<bool>,
}

#[derive(Default)]
pub struct Quorum {
    ballots: HashMap<String, Ballot>,
}

fn vote_bytes(vote: bool) -> [u8; 1] {
    [vote as u8]
}

impl Quorum {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_validator(&mut self, validator_id: &str, weight: f64) {
        self.ballots.insert(
            validator_id.to_string(),
            Ballot {
                weight,
                commitment: None,
                revealed_vote: None,
            },
        );
    }

    fn all_committed(&self) -> bool {
        !self.ballots.is_empty() && self.ballots.values().all(|b| b.commitment.is_some())
    }

    /// Validators commit a hash of (vote, salt) -- the vote itself is
    /// hidden until reveal, so a validator cannot simply copy another
    /// validator's already-visible vote.
    pub fn commit(
        &mut self,
        validator_id: &str,
        vote: bool,
        salt: &[u8],
    ) -> Result<(), QuorumError> {
        let ballot = self
            .ballots
            .get_mut(validator_id)
            .ok_or_else(|| QuorumError::UnknownValidator(validator_id.to_string()))?;
        ballot.commitment = Some(commit(&vote_bytes(vote), salt));
        Ok(())
    }

    pub fn reveal(
        &mut self,
        validator_id: &str,
        vote: bool,
        salt: &[u8],
    ) -> Result<(), QuorumError> {
        if !self.all_committed() {
            return Err(QuorumError::RevealBeforeAllCommitted);
        }
        let ballot = self
            .ballots
            .get_mut(validator_id)
            .ok_or_else(|| QuorumError::UnknownValidator(validator_id.to_string()))?;
        let commitment = ballot
            .commitment
            .as_ref()
            .expect("all_committed checked above");
        if !verify_reveal(commitment, &vote_bytes(vote), salt) {
            return Err(QuorumError::CommitMismatch(validator_id.to_string()));
        }
        ballot.revealed_vote = Some(vote);
        Ok(())
    }

    /// Weighted-majority aggregation. Returns `(outcome, agreement_fraction)`.
    /// Ballots that never revealed are excluded (non-response), matching
    /// docs/security/THREAT-MODEL.md's tolerance for nonresponsive
    /// validators without treating silence as a vote either way.
    pub fn aggregate(&self, threshold: f64) -> Result<(bool, f64), QuorumError> {
        let revealed: Vec<&Ballot> = self
            .ballots
            .values()
            .filter(|b| b.revealed_vote.is_some())
            .collect();
        if revealed.is_empty() {
            return Err(QuorumError::NoRevealedVotes);
        }
        let total_weight: f64 = revealed.iter().map(|b| b.weight).sum();
        let yes_weight: f64 = revealed
            .iter()
            .filter(|b| b.revealed_vote == Some(true))
            .map(|b| b.weight)
            .sum();
        let fraction = if total_weight > 0.0 {
            yes_weight / total_weight
        } else {
            0.0
        };
        Ok((fraction >= threshold, fraction))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_quorum(
        honest_ids: &[&str],
        colluding_ids: &[&str],
        weight_honest: f64,
        weight_colluding: f64,
    ) -> Quorum {
        let mut q = Quorum::new();
        for id in honest_ids {
            q.add_validator(id, weight_honest);
        }
        for id in colluding_ids {
            q.add_validator(id, weight_colluding);
        }
        q
    }

    fn commit_and_reveal_all(q: &mut Quorum, honest_ids: &[&str], colluding_ids: &[&str]) {
        for id in honest_ids {
            q.commit(id, false, format!("salt-{id}").as_bytes())
                .unwrap(); // honest: votes fail (correct)
        }
        for id in colluding_ids {
            q.commit(id, true, format!("salt-{id}").as_bytes()).unwrap(); // colluders: vote pass anyway
        }
        for id in honest_ids {
            q.reveal(id, false, format!("salt-{id}").as_bytes())
                .unwrap();
        }
        for id in colluding_ids {
            q.reveal(id, true, format!("salt-{id}").as_bytes()).unwrap();
        }
    }

    #[test]
    fn minority_collusion_does_not_flip_the_outcome() {
        let honest: Vec<String> = (0..7).map(|i| format!("honest-{i}")).collect();
        let colluding: Vec<String> = (0..3).map(|i| format!("colluder-{i}")).collect();
        let honest_refs: Vec<&str> = honest.iter().map(String::as_str).collect();
        let colluding_refs: Vec<&str> = colluding.iter().map(String::as_str).collect();

        let mut q = build_quorum(&honest_refs, &colluding_refs, 1.0, 1.0); // minority: 3 of 10
        commit_and_reveal_all(&mut q, &honest_refs, &colluding_refs);

        let (outcome_passed, agreement) = q.aggregate(0.5).unwrap();
        assert!(!outcome_passed); // correct outcome held: minority collusion blocked
        assert!((agreement - 0.3).abs() < 1e-9);
    }

    #[test]
    fn majority_collusion_does_flip_the_outcome_documented_residual_risk() {
        let honest: Vec<String> = (0..4).map(|i| format!("honest-{i}")).collect();
        let colluding: Vec<String> = (0..6).map(|i| format!("colluder-{i}")).collect();
        let honest_refs: Vec<&str> = honest.iter().map(String::as_str).collect();
        let colluding_refs: Vec<&str> = colluding.iter().map(String::as_str).collect();

        let mut q = build_quorum(&honest_refs, &colluding_refs, 1.0, 1.0); // majority: 6 of 10
        commit_and_reveal_all(&mut q, &honest_refs, &colluding_refs);

        let (outcome_passed, agreement) = q.aggregate(0.5).unwrap();
        // Documenting the known limitation, not hiding it: a colluding
        // majority DOES flip the aggregate outcome in this design.
        assert!(outcome_passed);
        assert!((agreement - 0.6).abs() < 1e-9);
    }

    #[test]
    fn commit_reveal_prevents_copying_another_validators_vote() {
        let mut q = Quorum::new();
        q.add_validator("v1", 1.0);
        q.add_validator("v2", 1.0);

        q.commit("v1", true, b"s1").unwrap();
        q.commit("v2", true, b"s2").unwrap();

        // v2 tries to reveal a DIFFERENT vote than it committed to.
        let err = q.reveal("v2", false, b"s2").unwrap_err();
        assert_eq!(err, QuorumError::CommitMismatch("v2".to_string()));
    }

    #[test]
    fn reveal_before_all_committed_is_rejected() {
        let mut q = Quorum::new();
        q.add_validator("v1", 1.0);
        q.add_validator("v2", 1.0);

        q.commit("v1", true, b"s1").unwrap();
        // v2 has not committed yet.
        let err = q.reveal("v1", true, b"s1").unwrap_err();
        assert_eq!(err, QuorumError::RevealBeforeAllCommitted);
    }

    #[test]
    fn aggregate_with_no_revealed_votes_errors_rather_than_defaulting() {
        let mut q = Quorum::new();
        q.add_validator("v1", 1.0);
        assert_eq!(q.aggregate(0.5).unwrap_err(), QuorumError::NoRevealedVotes);
    }

    #[test]
    fn a_validator_who_commits_but_never_reveals_is_excluded_from_aggregate() {
        let mut q = Quorum::new();
        q.add_validator("v1", 1.0);
        q.add_validator("v2", 1.0);
        q.add_validator("v3", 1.0);

        q.commit("v1", true, b"s1").unwrap();
        q.commit("v2", true, b"s2").unwrap();
        q.commit("v3", false, b"s3").unwrap();

        q.reveal("v1", true, b"s1").unwrap();
        q.reveal("v2", true, b"s2").unwrap();
        // v3 committed but never reveals (e.g. a nonresponsive validator).

        let (outcome_passed, agreement) = q.aggregate(0.5).unwrap();
        // v3's weight is excluded entirely, not counted as either a yes
        // or a no vote -- outcome is decided only among the 2 revealed
        // ballots (both true), not diluted by the nonresponsive third.
        assert!(outcome_passed);
        assert!((agreement - 1.0).abs() < 1e-9);
    }

    #[test]
    fn a_validator_who_never_commits_at_all_blocks_the_reveal_phase_for_everyone() {
        // Different failure mode from the above: all_committed() requires
        // EVERY registered validator to have committed before ANY reveal
        // is accepted -- a validator who never even commits (as opposed
        // to committing then going silent) blocks reveals entirely. This
        // is a real design property worth a dedicated test rather than
        // being conflated with the "committed but didn't reveal" case.
        let mut q = Quorum::new();
        q.add_validator("v1", 1.0);
        q.add_validator("v2", 1.0);
        q.add_validator("v3", 1.0); // never commits

        q.commit("v1", true, b"s1").unwrap();
        q.commit("v2", true, b"s2").unwrap();

        let err = q.reveal("v1", true, b"s1").unwrap_err();
        assert_eq!(err, QuorumError::RevealBeforeAllCommitted);
    }
}
