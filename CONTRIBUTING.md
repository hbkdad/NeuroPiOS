# Contributing

## NeuroPiOS (landing page)

Edit `index.html` directly; releases are published under `releases/` and surfaced via the GitHub Releases-backed download button. No build step.

## AION

AION is early-stage (see `docs/ROADMAP.md` for current phase status). Before contributing:

1. Read `docs/ARCHITECTURE.md`, `docs/PROTOCOL.md`, and the ADR relevant to the area you're changing (`docs/adr/`).
2. If your change introduces a new architectural decision (new dependency choice, new protocol field, new verification method, new settlement assumption), write an ADR (`docs/adr/NNNN-title.md`: context, options considered, decision, reasoning, consequences) rather than deciding silently.
3. Code changes need tests that actually run. For `simulations/poig_sim`: `cd simulations/poig_sim && pip install -e . && pytest`. For Rust crates: `cargo test --workspace` from the repo root (also run `cargo fmt --check` and `cargo clippy --workspace --all-targets`).
4. Follow `docs/protocol/SLASHING-SPEC.md` and `docs/POIG-SPEC.md` invariants exactly if touching reward/reputation/slashing logic — these encode explicit anti-fraud requirements, not stylistic preferences.
5. No mainnet or production-token changes without explicit maintainer authorization (`docs/adr/0004-token-development-staging.md`).
6. Use feature branches and focused commits/PRs, one logical change per PR where practical.
