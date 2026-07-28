//! Canonical job object per docs/PROTOCOL.md. This is the wire-format
//! authority (the Python reference in simulations/poig_sim/poig_sim/protocol.py
//! is a semantics reference for the economic simulator only, per that
//! module's own doc comment).

pub const PROTOCOL_VERSION_MAJOR: u32 = 0;
pub const PROTOCOL_VERSION_MINOR: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JobType {
    Inference,
    FineTune,
    Train,
    Compress,
    Quantize,
    BenchmarkCreate,
    BenchmarkRun,
    DatasetContribute,
    Evaluate,
    Host,
    Route,
    AgentExecute,
}

impl JobType {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobType::Inference => "inference",
            JobType::FineTune => "fine_tune",
            JobType::Train => "train",
            JobType::Compress => "compress",
            JobType::Quantize => "quantize",
            JobType::BenchmarkCreate => "benchmark_create",
            JobType::BenchmarkRun => "benchmark_run",
            JobType::DatasetContribute => "dataset_contribute",
            JobType::Evaluate => "evaluate",
            JobType::Host => "host",
            JobType::Route => "route",
            JobType::AgentExecute => "agent_execute",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerificationPolicy {
    L0,
    L1,
    L2,
    L3,
    L4,
    L5,
    L6,
}

impl VerificationPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            VerificationPolicy::L0 => "L0",
            VerificationPolicy::L1 => "L1",
            VerificationPolicy::L2 => "L2",
            VerificationPolicy::L3 => "L3",
            VerificationPolicy::L4 => "L4",
            VerificationPolicy::L5 => "L5",
            VerificationPolicy::L6 => "L6",
        }
    }
}

/// Canonical job object. Field order in `canonical_payload` is fixed and
/// must never be reordered for an existing protocol version -- a reordering
/// is a breaking change requiring a PROTOCOL_VERSION_MAJOR bump, per
/// docs/PROTOCOL.md's versioning rule.
#[derive(Debug, Clone, PartialEq)]
pub struct Job {
    pub job_id: String,
    pub requester_id: String,
    pub job_type: JobType,
    pub base_value: f64,
    pub escrow_amount: f64,
    pub verification_policy: VerificationPolicy,
    pub nonce: u64,
    pub signature: String,
}

impl Job {
    /// The bytes a requester signs (and a verifier re-derives to check the
    /// signature against). Does NOT include `signature` itself, obviously.
    pub fn canonical_payload(&self) -> Vec<u8> {
        format!(
            "{}|{}|{}|{}|{}|{}|{}|{}.{}",
            self.job_id,
            self.requester_id,
            self.job_type.as_str(),
            self.base_value,
            self.escrow_amount,
            self.verification_policy.as_str(),
            self.nonce,
            PROTOCOL_VERSION_MAJOR,
            PROTOCOL_VERSION_MINOR,
        )
        .into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_job() -> Job {
        Job {
            job_id: "job-1".into(),
            requester_id: "alice".into(),
            job_type: JobType::Inference,
            base_value: 10.0,
            escrow_amount: 10.0,
            verification_policy: VerificationPolicy::L4,
            nonce: 0,
            signature: String::new(),
        }
    }

    #[test]
    fn canonical_payload_is_deterministic() {
        let a = sample_job();
        let b = sample_job();
        assert_eq!(a.canonical_payload(), b.canonical_payload());
    }

    #[test]
    fn canonical_payload_changes_when_any_field_changes() {
        let base = sample_job();
        let mut changed = sample_job();
        changed.nonce = 1;
        assert_ne!(base.canonical_payload(), changed.canonical_payload());

        let mut changed2 = sample_job();
        changed2.escrow_amount = 999.0;
        assert_ne!(base.canonical_payload(), changed2.canonical_payload());
    }

    #[test]
    fn canonical_payload_is_stable_across_versions_field_order() {
        // Regression guard: if this fails, someone reordered a field in
        // canonical_payload, which is a breaking wire-format change that
        // needs a PROTOCOL_VERSION_MAJOR bump, not a silent edit.
        let job = sample_job();
        let expected = b"job-1|alice|inference|10|10|L4|0|0.1".to_vec();
        assert_eq!(job.canonical_payload(), expected);
    }
}
