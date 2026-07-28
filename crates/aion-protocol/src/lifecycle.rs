//! Job lifecycle state machine per docs/PROTOCOL.md. Encodes the core
//! invariant from docs/security/THREAT-MODEL.md: "a job cannot settle
//! twice" -- Settled and Slashed are terminal; no transition is permitted
//! out of a terminal state, ever.

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JobState {
    Created,
    Open,
    Assigned,
    Executing,
    ResultSubmitted,
    Verified,
    Disputed,
    Resolved,
    Settled,
    Slashed,
}

impl JobState {
    pub fn is_terminal(self) -> bool {
        matches!(self, JobState::Settled | JobState::Slashed)
    }

    /// States reachable directly from this one, per the job lifecycle
    /// diagram in docs/PROTOCOL.md.
    pub fn allowed_next(self) -> &'static [JobState] {
        use JobState::*;
        match self {
            Created => &[Open],
            Open => &[Assigned],
            Assigned => &[Executing],
            Executing => &[ResultSubmitted],
            ResultSubmitted => &[Verified, Disputed],
            Verified => &[Settled],
            Disputed => &[Resolved],
            Resolved => &[Settled, Slashed],
            Settled => &[],
            Slashed => &[],
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TransitionError {
    #[error("cannot transition out of terminal state {0:?} -- a job cannot settle twice")]
    AlreadyTerminal(JobState),
    #[error("invalid transition from {from:?} to {to:?}")]
    InvalidTransition { from: JobState, to: JobState },
}

#[derive(Debug)]
pub struct JobLifecycle {
    state: JobState,
}

impl Default for JobLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

impl JobLifecycle {
    pub fn new() -> Self {
        Self {
            state: JobState::Created,
        }
    }

    pub fn state(&self) -> JobState {
        self.state
    }

    pub fn transition(&mut self, to: JobState) -> Result<(), TransitionError> {
        if self.state.is_terminal() {
            return Err(TransitionError::AlreadyTerminal(self.state));
        }
        if !self.state.allowed_next().contains(&to) {
            return Err(TransitionError::InvalidTransition {
                from: self.state,
                to,
            });
        }
        self.state = to;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use JobState::*;

    #[test]
    fn happy_path_settles_once() {
        let mut lc = JobLifecycle::new();
        for step in [
            Open,
            Assigned,
            Executing,
            ResultSubmitted,
            Verified,
            Settled,
        ] {
            lc.transition(step).unwrap();
        }
        assert_eq!(lc.state(), Settled);
    }

    #[test]
    fn dispute_path_can_end_in_slashed() {
        let mut lc = JobLifecycle::new();
        for step in [
            Open,
            Assigned,
            Executing,
            ResultSubmitted,
            Disputed,
            Resolved,
            Slashed,
        ] {
            lc.transition(step).unwrap();
        }
        assert_eq!(lc.state(), Slashed);
    }

    #[test]
    fn cannot_settle_twice() {
        let mut lc = JobLifecycle::new();
        for step in [
            Open,
            Assigned,
            Executing,
            ResultSubmitted,
            Verified,
            Settled,
        ] {
            lc.transition(step).unwrap();
        }
        let err = lc.transition(Settled).unwrap_err();
        assert_eq!(err, TransitionError::AlreadyTerminal(Settled));
    }

    #[test]
    fn cannot_transition_out_of_slashed_either() {
        let mut lc = JobLifecycle::new();
        for step in [
            Open,
            Assigned,
            Executing,
            ResultSubmitted,
            Disputed,
            Resolved,
            Slashed,
        ] {
            lc.transition(step).unwrap();
        }
        let err = lc.transition(Settled).unwrap_err();
        assert_eq!(err, TransitionError::AlreadyTerminal(Slashed));
    }

    #[test]
    fn cannot_skip_states() {
        let mut lc = JobLifecycle::new();
        // Created -> Settled directly is not a legal transition.
        let err = lc.transition(Settled).unwrap_err();
        assert_eq!(
            err,
            TransitionError::InvalidTransition {
                from: Created,
                to: Settled
            }
        );
    }

    #[test]
    fn cannot_reopen_a_resolved_dispute_as_verified() {
        let mut lc = JobLifecycle::new();
        for step in [Open, Assigned, Executing, ResultSubmitted, Disputed] {
            lc.transition(step).unwrap();
        }
        // Disputed can only go to Resolved, not directly to Verified.
        let err = lc.transition(Verified).unwrap_err();
        assert_eq!(
            err,
            TransitionError::InvalidTransition {
                from: Disputed,
                to: Verified
            }
        );
    }
}
