//! Per-requester monotonic nonce tracking (replay protection), per
//! docs/PROTOCOL.md.

use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
#[error("nonce {nonce} <= last seen nonce {last:?} for requester {requester_id}")]
pub struct ReplayError {
    pub requester_id: String,
    pub nonce: u64,
    pub last: Option<u64>,
}

#[derive(Default)]
pub struct NonceTracker {
    last_nonce: HashMap<String, u64>,
}

impl NonceTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns Ok(()) and records the nonce if it's strictly greater than
    /// the last one seen for this requester (or if none has been seen yet).
    /// Returns Err without mutating state otherwise -- a rejected nonce
    /// never gets "half-recorded".
    pub fn check_and_advance(&mut self, requester_id: &str, nonce: u64) -> Result<(), ReplayError> {
        let last = self.last_nonce.get(requester_id).copied();
        if let Some(last_nonce) = last {
            if nonce <= last_nonce {
                return Err(ReplayError {
                    requester_id: requester_id.to_string(),
                    nonce,
                    last,
                });
            }
        }
        self.last_nonce.insert(requester_id.to_string(), nonce);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_nonce_for_a_requester_always_succeeds() {
        let mut t = NonceTracker::new();
        assert!(t.check_and_advance("alice", 0).is_ok());
    }

    #[test]
    fn increasing_nonces_succeed() {
        let mut t = NonceTracker::new();
        assert!(t.check_and_advance("alice", 0).is_ok());
        assert!(t.check_and_advance("alice", 1).is_ok());
        assert!(t.check_and_advance("alice", 5).is_ok());
    }

    #[test]
    fn replayed_nonce_is_rejected() {
        let mut t = NonceTracker::new();
        assert!(t.check_and_advance("alice", 5).is_ok());
        let err = t.check_and_advance("alice", 5).unwrap_err();
        assert_eq!(err.requester_id, "alice");
        assert_eq!(err.nonce, 5);
        assert_eq!(err.last, Some(5));
    }

    #[test]
    fn lower_nonce_after_higher_is_rejected() {
        let mut t = NonceTracker::new();
        assert!(t.check_and_advance("alice", 10).is_ok());
        assert!(t.check_and_advance("alice", 3).is_err());
    }

    #[test]
    fn rejected_nonce_does_not_mutate_state() {
        let mut t = NonceTracker::new();
        assert!(t.check_and_advance("alice", 5).is_ok());
        let _ = t.check_and_advance("alice", 5); // rejected
                                                 // last-seen nonce is still 5, not somehow bumped by the rejected attempt
        assert!(t.check_and_advance("alice", 6).is_ok());
    }

    #[test]
    fn nonces_are_independent_per_requester() {
        let mut t = NonceTracker::new();
        assert!(t.check_and_advance("alice", 0).is_ok());
        assert!(t.check_and_advance("bob", 0).is_ok()); // bob's nonce space is separate
    }
}
