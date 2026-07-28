# AION Roadmap

Phases as defined by the orchestration brief. Status reflects reality as of this milestone (2026-07-28) — see the milestone report in the PR description for the authoritative "what actually happened" record.

| Phase | Name | Status |
|---|---|---|
| 0 | Research | **Done this milestone** — capability matrix, 5 competitor analyses, verification matrix, research log |
| 1 | Specification | **Done this milestone** — ARCHITECTURE, PROTOCOL, POIG-SPEC, ECONOMIC-MODEL, VERIFICATION, SLASHING-SPEC, THREAT-MODEL, ECONOMIC-ATTACKS, 6 ADRs |
| 2 | Local simulator (no blockchain) | **Started this milestone** — `simulations/poig_sim`: nodes, jobs, PoIG scoring, reputation, 2 of the brief's attack scenarios simulated with real passing tests. NOT complete: no 100/1k/10k/100k-node sweep yet, no quality/improvement-factor validation against real models |
| 3 | Real P2P node | Not started |
| 4 | AI execution | Not started (no GPU/runtime in this dev container; needs a real environment) |
| 5 | Verification | Not started (design complete in docs, no implementation) |
| 6 | Storage | Not started |
| 7 | Marketplace | Not started |
| 8 | Web dashboard | Not started |
| 9 | Desktop node | Not started |
| 10 | Testnet settlement | Not started — blocked on operator-provided testnet RPC/funded deployer key (credentials stop-condition) |
| 11 | ZKML experiment | Not started |
| 12 | Economic testnet | Not started |
| 13 | Security review | Not started — requires external audit, cannot be self-certified by this agent |

## Next highest-priority task (per COMPLETION RULE)

Resolve the open architectural risk flagged in `docs/security/THREAT-MODEL.md` ("off-chain PoIG number trusted blindly on-chain") before extending Phase 2's simulator toward Phase 3 (real P2P node) — this is a design gap, not an implementation gap, and should be closed with a dedicated ADR before more code is built on top of the current PoIG-to-settlement assumption.

Second-priority: scale the Phase 2 simulator's economic-attack coverage (validator collusion, price manipulation, benchmark leakage) beyond the two attacks actually simulated this milestone, and run the multi-scale (100 → 100,000 node) Monte Carlo sweep the brief specifies for full economic validation.

## Definition of MVP (unchanged from brief, restated for tracking)

Three machines discover each other; a requester creates a signed job; a worker executes it; another node verifies it; the model artifact is retrievable by content address; all messages are authenticated; a simulated/testnet reward settles; the dashboard shows real telemetry; bad results can be challenged; a job cannot be paid twice. None of this is built yet — Phase 2 validates the economic logic in isolation, which is a prerequisite, not the MVP itself.
