# Security Policy

## NeuroPiOS

Report issues with the landing page or release pipeline via a GitHub issue on this repository.

## AION

AION is pre-production (see `docs/ROADMAP.md`) — there is no deployed network, no real token, and no smart contract on any chain yet. `docs/security/THREAT-MODEL.md` and `docs/security/ECONOMIC-ATTACKS.md` document known/anticipated risks at the design level; treat findings against those documents as design feedback (open an issue), not a live vulnerability disclosure, until Phase 10+ (testnet settlement) actually deploys something.

`docs/security/AUDIT-2026-07-28.md` records a real internal (self-run, not independent) audit: `cargo audit`/`pip-audit` results, an `unsafe`-code and unguarded-panic sweep, a secrets sweep, and the CI/Dependabot gap it found and closed. It is not a substitute for Phase 13's external audit, which remains not started.

Once real infrastructure exists:
- Smart contracts will follow OpenZeppelin primitives, mandatory Slither/Echidna static analysis and fuzz/invariant testing before any testnet deployment (see `docs/adr/0001-settlement-layer.md`, `docs/adr/0004-token-development-staging.md`).
- No mainnet or production token deployment happens without explicit, documented authorization — see `docs/adr/0004-token-development-staging.md`.
- Dependency pinning and lockfiles are in place (`Cargo.lock` committed; CI and Dependabot added per `docs/security/AUDIT-2026-07-28.md`). SBOM generation is required before any external release and is not yet implemented.

Until then, report design-level security concerns (protocol gaps, economic attack vectors not covered in `docs/security/ECONOMIC-ATTACKS.md`) as GitHub issues.
