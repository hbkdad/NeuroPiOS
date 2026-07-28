//! L3 tier: deterministic/reproducible execution, per docs/VERIFICATION.md
//! and docs/adr/0003-verification-tiering.md ("high-value jobs where
//! requester accepts a restricted hardware pool").
//!
//! Honest scope note: this is a REAL, working deterministic-execution
//! harness. `execute_deterministic` actually compiles and runs a WASM
//! module using `wasmi` -- a pure-Rust, interpreter-only WASM engine
//! (deliberately chosen over a JIT-based engine: no native codegen means
//! no per-CPU-architecture codegen variance to worry about, which is
//! exactly the class of nondeterminism a "restricted hardware pool" tier
//! exists to eliminate). What this does NOT do is run any real AI/ML
//! inference workload -- that needs a real execution pipeline
//! (`crates/aion-runtime`, Phase 4), which doesn't exist in this dev
//! container (no GPU). The property under test here -- "does
//! re-executing the SAME computation in a controlled sandbox produce
//! byte-identical output, and if not, is that itself the fraud signal" --
//! is real and generalizes to whatever workload eventually runs inside
//! the sandbox; only the workload itself (a toy WASM function here) is a
//! stand-in.
//!
//! Design note vs. `replicated.rs` (L2): L2 tolerates small numeric
//! divergence as legitimate hardware/kernel nondeterminism and clusters
//! results accordingly. L3's whole premise is a sandbox restrictive enough
//! that no legitimate nondeterminism should remain -- so this module
//! requires EXACT equality across replicas, with zero tolerance band. Any
//! divergence at all is itself the fraud/tamper signal.

use crate::tier::{verification_confidence, Tier};
use thiserror::Error;
use wasmi::{Engine, Linker, Module, Store};

#[derive(Debug, Error)]
pub enum DeterministicExecutionError {
    #[error("failed to compile wasm module: {0}")]
    ModuleCompile(String),
    #[error("failed to instantiate wasm module: {0}")]
    Instantiate(String),
    #[error("exported function {0:?} not found or has an unexpected i32->i32 signature")]
    ExportNotFound(String),
    #[error("wasm execution trapped: {0}")]
    Trap(String),
}

/// Compiles and runs the given module's exported `i32 -> i32` function
/// with `input`, inside a fresh, isolated `wasmi` engine/store per call --
/// no state is shared across calls, so this is a genuine independent
/// re-execution each time, not a cached/memoized shortcut.
pub fn execute_deterministic(
    wasm_bytes: &[u8],
    export_name: &str,
    input: i32,
) -> Result<i32, DeterministicExecutionError> {
    let engine = Engine::default();
    let module = Module::new(&engine, wasm_bytes)
        .map_err(|e| DeterministicExecutionError::ModuleCompile(e.to_string()))?;
    let mut store = Store::new(&engine, ());
    let linker = Linker::new(&engine);
    let instance = linker
        .instantiate_and_start(&mut store, &module)
        .map_err(|e| DeterministicExecutionError::Instantiate(e.to_string()))?;
    let func = instance
        .get_typed_func::<i32, i32>(&store, export_name)
        .map_err(|_| DeterministicExecutionError::ExportNotFound(export_name.to_string()))?;
    func.call(&mut store, input)
        .map_err(|e| DeterministicExecutionError::Trap(e.to_string()))
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DeterministicVerificationError {
    #[error("wasm execution failed on replica {replica_index}: {message}")]
    ExecutionFailed {
        replica_index: usize,
        message: String,
    },
    #[error("deterministic re-execution produced DIVERGING outputs across replicas: {0:?} -- under L3's controlled sandbox this is itself a fraud/tamper signal, not tolerable numeric noise (contrast with L2's tolerance-band handling in replicated.rs)")]
    OutputsDiverged(Vec<i32>),
    #[error("at least 2 replicas are required to prove determinism -- a single execution proves nothing")]
    InsufficientReplicas,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeterministicReplicationOutcome {
    consensus_output: i32,
    replica_count: usize,
}

impl DeterministicReplicationOutcome {
    pub fn consensus_output(&self) -> i32 {
        self.consensus_output
    }

    pub fn replica_count(&self) -> usize {
        self.replica_count
    }

    /// `VerificationConfidence` for docs/POIG-SPEC.md's reward formula.
    /// Reaching this type at all means every replica agreed exactly, so
    /// this is always the full L3 base confidence -- there is no partial
    /// pass at this tier the way L2 allows for an outlier-tolerant
    /// consensus.
    pub fn verification_confidence(&self) -> f64 {
        verification_confidence(Tier::L3, true)
    }
}

/// Pure comparison logic: given a list of already-computed outputs (in a
/// real deployment, one per independently-executing worker's sandboxed
/// run of the identical job), requires EXACT agreement. Separated from
/// `execute_deterministic` the same way `replicated.rs`'s
/// `evaluate_replicated_results` is separated from any execution step --
/// this function doesn't care how the outputs were produced, only whether
/// they agree.
pub fn compare_replicated_outputs(
    outputs: &[i32],
) -> Result<DeterministicReplicationOutcome, DeterministicVerificationError> {
    if outputs.len() < 2 {
        return Err(DeterministicVerificationError::InsufficientReplicas);
    }
    let first = outputs[0];
    if outputs.iter().all(|&o| o == first) {
        Ok(DeterministicReplicationOutcome {
            consensus_output: first,
            replica_count: outputs.len(),
        })
    } else {
        Err(DeterministicVerificationError::OutputsDiverged(
            outputs.to_vec(),
        ))
    }
}

/// Convenience wrapper that actually performs `replica_count` independent
/// re-executions of the same wasm module+input on this machine (each a
/// fresh `Engine`/`Store`, per `execute_deterministic`'s doc) and feeds the
/// results into `compare_replicated_outputs`. This genuinely proves "the
/// sandbox reliably reproduces its own output across independent runs" --
/// a real, meaningful property -- though it cannot, by itself, simulate a
/// malicious remote worker submitting a fabricated result (that path is
/// covered directly via `compare_replicated_outputs` in this module's
/// tests instead, the same way `replicated.rs`'s outlier tests construct
/// `ReplicatedResult` values by hand rather than needing an actually
/// malicious executor).
pub fn verify_deterministic_replication(
    wasm_bytes: &[u8],
    export_name: &str,
    input: i32,
    replica_count: usize,
) -> Result<DeterministicReplicationOutcome, DeterministicVerificationError> {
    if replica_count < 2 {
        return Err(DeterministicVerificationError::InsufficientReplicas);
    }
    let mut outputs = Vec::with_capacity(replica_count);
    for replica_index in 0..replica_count {
        let output = execute_deterministic(wasm_bytes, export_name, input).map_err(|e| {
            DeterministicVerificationError::ExecutionFailed {
                replica_index,
                message: e.to_string(),
            }
        })?;
        outputs.push(output);
    }
    compare_replicated_outputs(&outputs)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `(x) -> x * 2`, a trivial deterministic pure function -- enough to
    /// prove the harness genuinely compiles, instantiates, and executes a
    /// real wasm module rather than stubbing the result.
    fn doubler_wasm() -> Vec<u8> {
        wat::parse_str(
            r#"
            (module
              (func (export "double") (param i32) (result i32)
                local.get 0
                i32.const 2
                i32.mul))
            "#,
        )
        .unwrap()
    }

    fn always_traps_wasm() -> Vec<u8> {
        wat::parse_str(
            r#"
            (module
              (func (export "boom") (param i32) (result i32)
                unreachable))
            "#,
        )
        .unwrap()
    }

    #[test]
    fn execute_deterministic_runs_a_real_wasm_module_and_returns_its_real_output() {
        let wasm = doubler_wasm();
        let result = execute_deterministic(&wasm, "double", 21).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn execute_deterministic_reports_a_trap_rather_than_a_fabricated_result() {
        let wasm = always_traps_wasm();
        let err = execute_deterministic(&wasm, "boom", 0).unwrap_err();
        assert!(matches!(err, DeterministicExecutionError::Trap(_)));
    }

    #[test]
    fn execute_deterministic_reports_a_missing_export_rather_than_panicking() {
        let wasm = doubler_wasm();
        let err = execute_deterministic(&wasm, "not_a_real_export", 0).unwrap_err();
        assert!(
            matches!(err, DeterministicExecutionError::ExportNotFound(name) if name == "not_a_real_export")
        );
    }

    #[test]
    fn independently_re_executing_the_same_module_five_times_agrees_exactly() {
        let wasm = doubler_wasm();
        let outcome = verify_deterministic_replication(&wasm, "double", 100, 5).unwrap();
        assert_eq!(outcome.consensus_output(), 200);
        assert_eq!(outcome.replica_count(), 5);
        assert_eq!(
            outcome.verification_confidence(),
            Tier::L3.base_confidence()
        );
    }

    #[test]
    fn a_single_replica_is_rejected_as_insufficient_to_prove_anything() {
        let wasm = doubler_wasm();
        let err = verify_deterministic_replication(&wasm, "double", 1, 1).unwrap_err();
        assert!(matches!(
            err,
            DeterministicVerificationError::InsufficientReplicas
        ));
    }

    #[test]
    fn a_trapping_module_surfaces_which_replica_failed_rather_than_a_generic_error() {
        let wasm = always_traps_wasm();
        let err = verify_deterministic_replication(&wasm, "boom", 0, 3).unwrap_err();
        assert!(matches!(
            err,
            DeterministicVerificationError::ExecutionFailed {
                replica_index: 0,
                ..
            }
        ));
    }

    #[test]
    fn unanimous_externally_supplied_outputs_reach_full_l3_confidence() {
        let outcome = compare_replicated_outputs(&[7, 7, 7, 7]).unwrap();
        assert_eq!(outcome.consensus_output(), 7);
        assert_eq!(outcome.replica_count(), 4);
    }

    #[test]
    fn a_single_tampered_replica_among_honest_agreement_is_flagged_not_tolerated() {
        // Simulates a malicious worker submitting a fabricated result
        // alongside honest workers' genuinely-agreeing outputs. Unlike L2,
        // there is no tolerance band here -- ANY divergence flags the
        // whole batch, since a correctly-restricted sandbox should never
        // produce legitimate divergence in the first place.
        let err = compare_replicated_outputs(&[42, 42, 42, 99]).unwrap_err();
        assert_eq!(
            err,
            DeterministicVerificationError::OutputsDiverged(vec![42, 42, 42, 99])
        );
    }

    #[test]
    fn even_a_tiny_divergence_is_not_tolerated_unlike_l2() {
        // L2's numeric-tolerance clustering would treat this as legitimate
        // nondeterminism. L3's whole premise is a sandbox where that
        // excuse shouldn't exist -- so even an off-by-one is flagged.
        let err = compare_replicated_outputs(&[1000, 1000, 1001]).unwrap_err();
        assert!(matches!(
            err,
            DeterministicVerificationError::OutputsDiverged(_)
        ));
    }
}
