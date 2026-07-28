# services/coordinator — Registry + Job Coordination Service (planned, Phase 3+)

Target stack: Rust, Tokio, Axum, Serde, SQLx, PostgreSQL (per orchestration brief default; Redis only if a concrete need is demonstrated — none identified yet). Owns canonical job/worker/model registry state ahead of full on-chain settlement (Phase 10). A modular monolith, not split into further microservices, until a measured scaling requirement justifies it (per brief's explicit guidance against premature microservices). **Status: not started.**
