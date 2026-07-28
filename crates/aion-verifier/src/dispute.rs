//! L4 tier: optimistic commit + dispute window + bisection ("challenge
//! protocol"), per docs/VERIFICATION.md and docs/adr/0003-verification-tiering.md
//! (the Gensyn-Verde-style pattern chosen there specifically because it's
//! cheap by default and only pays arbitration cost when a result is
//! actually disputed).
//!
//! Honest scope note: this module implements the STATE MACHINE around a
//! dispute (commit -> challenge window -> disputed/undisputed -> resolved)
//! and its effect on `VerificationConfidence`. It does NOT implement the
//! bisection re-execution/arbitration protocol itself (interactively
//! narrowing down to the exact disputed computation step) -- that needs a
//! real job-execution pipeline to bisect over, which doesn't exist yet
//! (see docs/ROADMAP.md Phase 4). What's real here is the cheap-by-default
//! property and the state transitions a caller (a future
//! `services/verifier`) would actually need to drive.

use crate::tier::{verification_confidence, Tier};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChallengeState {
    /// Result committed, challenge window open, no dispute raised yet.
    AwaitingChallengeWindow,
    /// A dispute was raised before the window closed; awaiting resolution
    /// (bisection arbitration -- not modeled here, see module doc).
    Disputed,
    /// Window closed with no dispute raised -- the common, cheap case.
    UndisputedFinal,
    /// A dispute was resolved and fraud was proven (result was wrong).
    ResolvedFraudProven,
    /// A dispute was resolved and fraud was NOT proven (result stands;
    /// the challenger's bond may be forfeited per
    /// docs/protocol/SLASHING-SPEC.md's slashed-funds policy -- not
    /// modeled here, that's a settlement-layer concern).
    ResolvedFraudNotProven,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ChallengeError {
    #[error("cannot raise a dispute: challenge window is not open (state is {0:?})")]
    WindowNotOpen(ChallengeState),
    #[error("cannot close the challenge window: it is not currently open (state is {0:?})")]
    CannotCloseWindow(ChallengeState),
    #[error("cannot resolve a dispute: no dispute is currently open (state is {0:?})")]
    NoDisputeToResolve(ChallengeState),
}

/// The challenge-window state machine for a single job result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChallengeWindow {
    state: ChallengeState,
}

impl Default for ChallengeWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl ChallengeWindow {
    pub fn new() -> Self {
        Self {
            state: ChallengeState::AwaitingChallengeWindow,
        }
    }

    pub fn state(&self) -> ChallengeState {
        self.state
    }

    pub fn is_final(&self) -> bool {
        matches!(
            self.state,
            ChallengeState::UndisputedFinal
                | ChallengeState::ResolvedFraudProven
                | ChallengeState::ResolvedFraudNotProven
        )
    }

    /// A challenger disputes the result before the window closes.
    pub fn raise_dispute(&mut self) -> Result<(), ChallengeError> {
        if self.state != ChallengeState::AwaitingChallengeWindow {
            return Err(ChallengeError::WindowNotOpen(self.state));
        }
        self.state = ChallengeState::Disputed;
        Ok(())
    }

    /// The challenge window elapses with no dispute raised -- the common,
    /// cheap case this tier is designed around.
    pub fn close_window_undisputed(&mut self) -> Result<(), ChallengeError> {
        if self.state != ChallengeState::AwaitingChallengeWindow {
            return Err(ChallengeError::CannotCloseWindow(self.state));
        }
        self.state = ChallengeState::UndisputedFinal;
        Ok(())
    }

    /// A raised dispute is resolved (by bisection arbitration, not modeled
    /// here) with a verdict on whether fraud was actually proven.
    pub fn resolve_dispute(&mut self, fraud_proven: bool) -> Result<(), ChallengeError> {
        if self.state != ChallengeState::Disputed {
            return Err(ChallengeError::NoDisputeToResolve(self.state));
        }
        self.state = if fraud_proven {
            ChallengeState::ResolvedFraudProven
        } else {
            ChallengeState::ResolvedFraudNotProven
        };
        Ok(())
    }

    /// `VerificationConfidence` for docs/POIG-SPEC.md's reward formula.
    /// Zero while a dispute is still pending resolution -- a job is never
    /// reward-eligible mid-dispute.
    pub fn verification_confidence(&self) -> f64 {
        match self.state {
            ChallengeState::UndisputedFinal | ChallengeState::ResolvedFraudNotProven => {
                verification_confidence(Tier::L4, true)
            }
            ChallengeState::ResolvedFraudProven
            | ChallengeState::AwaitingChallengeWindow
            | ChallengeState::Disputed => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undisputed_window_yields_full_l4_confidence() {
        let mut w = ChallengeWindow::new();
        assert_eq!(w.verification_confidence(), 0.0); // pending: not yet reward-eligible
        w.close_window_undisputed().unwrap();
        assert!(w.is_final());
        assert_eq!(w.verification_confidence(), Tier::L4.base_confidence());
    }

    #[test]
    fn disputed_and_fraud_proven_yields_zero_confidence() {
        let mut w = ChallengeWindow::new();
        w.raise_dispute().unwrap();
        assert_eq!(w.verification_confidence(), 0.0); // still pending resolution
        w.resolve_dispute(true).unwrap();
        assert!(w.is_final());
        assert_eq!(w.state(), ChallengeState::ResolvedFraudProven);
        assert_eq!(w.verification_confidence(), 0.0);
    }

    #[test]
    fn disputed_but_fraud_not_proven_still_yields_full_l4_confidence() {
        // A frivolous or unsuccessful challenge doesn't punish the honest
        // worker -- the result stands at full L4 confidence once cleared.
        let mut w = ChallengeWindow::new();
        w.raise_dispute().unwrap();
        w.resolve_dispute(false).unwrap();
        assert_eq!(w.state(), ChallengeState::ResolvedFraudNotProven);
        assert_eq!(w.verification_confidence(), Tier::L4.base_confidence());
    }

    #[test]
    fn cannot_raise_a_second_dispute_on_an_already_disputed_result() {
        let mut w = ChallengeWindow::new();
        w.raise_dispute().unwrap();
        let err = w.raise_dispute().unwrap_err();
        assert_eq!(err, ChallengeError::WindowNotOpen(ChallengeState::Disputed));
    }

    #[test]
    fn cannot_raise_a_dispute_after_the_window_already_closed() {
        let mut w = ChallengeWindow::new();
        w.close_window_undisputed().unwrap();
        let err = w.raise_dispute().unwrap_err();
        assert_eq!(
            err,
            ChallengeError::WindowNotOpen(ChallengeState::UndisputedFinal)
        );
    }

    #[test]
    fn cannot_close_the_window_twice() {
        let mut w = ChallengeWindow::new();
        w.close_window_undisputed().unwrap();
        let err = w.close_window_undisputed().unwrap_err();
        assert_eq!(
            err,
            ChallengeError::CannotCloseWindow(ChallengeState::UndisputedFinal)
        );
    }

    #[test]
    fn cannot_resolve_a_dispute_that_was_never_raised() {
        let mut w = ChallengeWindow::new();
        let err = w.resolve_dispute(true).unwrap_err();
        assert_eq!(
            err,
            ChallengeError::NoDisputeToResolve(ChallengeState::AwaitingChallengeWindow)
        );
    }

    #[test]
    fn cannot_resolve_the_same_dispute_twice() {
        let mut w = ChallengeWindow::new();
        w.raise_dispute().unwrap();
        w.resolve_dispute(true).unwrap();
        let err = w.resolve_dispute(false).unwrap_err();
        assert_eq!(
            err,
            ChallengeError::NoDisputeToResolve(ChallengeState::ResolvedFraudProven)
        );
    }
}
