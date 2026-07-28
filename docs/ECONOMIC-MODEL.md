# AION Economic Model

Status: Phase 1 specification + Phase 2 (Stage A, internal points ledger) simulation. See `docs/adr/0004-token-development-staging.md` for the staged token policy this document operates under — **Stage A only** in this milestone, no token, no monetary value anywhere in this repo.

## Design principle

The token/economic mechanism exists to **coordinate participants**, not to be the product. Primary KPIs (per the orchestration brief's Final Engineering Principle) are cost-per-useful-inference, verified compute capacity, time-to-first-token, throughput, model availability, verification accuracy, fraud-detection rate, decentralization, worker profitability, requester savings, model improvement rate, energy-per-useful-task, and failure rate — **not** token price. No dashboard, doc, or UI in this repo should ever surface token price as a headline metric.

## Fee flow (target design)

```
request fee
 ├── compute provider        (majority share — does the actual work)
 ├── model creator           (per ModelManifest royalty policy, if any)
 ├── validator(s)            (per verification tier actually used)
 ├── dataset contributor(s)  (per provenance DAG, if applicable)
 ├── protocol treasury       (funds governance-controlled network costs)
 └── burn/other sink         (optional, economically simulated before adoption)
```

Exact percentages are simulation outputs (see below), not fixed here — publishing hardcoded splits before simulation would repeat the exact mistake this brief warns against (assuming the original proposal's numbers are correct without testing them).

## Simulation methodology (Phase 2, this milestone)

`simulations/poig_sim` (Python) implements:
- A node population with configurable actor mix: honest worker, cheap/low-effort worker, premium worker, malicious worker, colluding validator cluster, fake requester, Sybil cluster.
- A minimal job market (signed job creation, escrow, worker assignment, verification-tier application, PoIG reward calculation, reputation update).
- Two concrete attack scenarios with assertions, run via `pytest`:
  1. **Self-dealing / fake-demand reward farming** — an actor creates jobs paid to itself with no real escrow-backed external demand; asserts `UsefulWorkScore` (and therefore `Reward`) stays at 0.
  2. **Sybil cluster reputation farming** — many newly-created identities attempt to bootstrap reputation via mutually-issued low-value jobs; asserts the reputation-gating + adaptive-verification-rate design (new/unproven identities get maximum-rate verification) keeps their effective payout at or below the honest-worker baseline.

This is a **first-milestone-scale** simulation (tens of synthetic nodes, deterministic scenarios), not the 100/1,000/10,000/100,000-node Monte Carlo sweep the brief specifies for full economic validation — that is explicitly out of scope for this milestone and tracked as follow-up work (see final report).

## Economic attack surface

Full attack tree and mitigations: `docs/security/ECONOMIC-ATTACKS.md`.

## Sybil resistance (design, not yet load-tested)

Layered, not single-mechanism: identity has a real (if small) cost (stake-equivalent bonding for validator/high-trust roles), new identities face maximum verification rate and reduced routing-score weight until they accumulate a reputation history, reputation decays over time and cannot be purchased or transferred, and job-history graph analysis (planned, not yet implemented) is available as a later defense against coordinated Sybil clusters. No government-ID requirement for ordinary participation, per the brief's explicit constraint.

## Wallet / agent economy

Deferred to Phase 8/9 implementation; specified in `docs/PROTOCOL.md` (agent economy job constraint) and `docs/POIG-SPEC.md` (spendingAuthorization). No wallet code exists in this repo yet.
