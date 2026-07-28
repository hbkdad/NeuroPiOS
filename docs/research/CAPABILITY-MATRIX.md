# AION Capability Matrix

Snapshot of the tooling actually available to the coding agent in this environment as of 2026-07-28. This governs what Stage 0 could realistically use. Re-run this inventory at the start of each major phase — availability changes across sessions/environments.

Legend: **Available** = usable now without setup. **Requires auth** = present but needs a credential/connection the operator must grant. **Not available** = does not exist in this environment; do not fabricate its use.

## Coding / repo tooling

| Name | Type | Purpose | Available? | Auth required? | AION phase | Risk | Planned usage |
|---|---|---|---|---|---|---|---|
| Bash | shell | Run commands, package managers, test runners | Available | No | All | Medium (irreversible ops) | Build, test, lint, git |
| Read/Write/Edit/Glob/Grep | file tools | Inspect and modify the repo | Available | No | All | Low | Primary editing surface |
| GitHub MCP (`mcp__github__*`) | VCS integration | Branches, PRs, issues, reviews, CI status | Available | Pre-authorized for `hbkdad/neuropios` | All | Medium (public visibility) | Branch mgmt, draft PRs, issue tracking for milestones |
| Agent (subagents: general-purpose, Explore, Plan, claude, claude-code-guide) | task delegation | Parallelize research/search | Available | No | Research, QA | Low | Used selectively; avoided here to keep synthesis in one place |
| WebSearch / WebFetch | research | Current external info beyond training cutoff | Available | No | Research (all specialist roles) | Low | Grounding competitor/tech research (used this session) |
| ScheduleWakeup / send_later / create_trigger (Routines) | scheduling | Recurring or delayed re-entry into this session | Available | No | DevOps/PR babysitting | Low | Not yet used; candidate for PR CI monitoring |
| Artifact | publishing | Render HTML/MD as a shareable page | Available | No | UI/UX prototyping, dashboards | Low (private by default) | Candidate for future dashboard mockups |

## Blockchain / smart-contract tooling

| Name | Type | Purpose | Available? | Auth required? | AION phase | Risk | Planned usage |
|---|---|---|---|---|---|---|---|
| Foundry / Hardhat / Slither / Echidna | contract dev+security | Compile, test, fuzz, statically analyze Solidity | **Not installed** in this container | N/A | Phase 10 (testnet settlement) | N/A until installed | Install via `foundryup` / npm when contracts phase begins; do not hand-roll a Solidity toolchain |
| Any RPC/wallet MCP (Coinbase, Alchemy, etc.) | chain access | Deploy/interact with testnets | **Not present** | N/A | Phase 10+ | N/A | No blockchain deployment tooling is connected. Contracts phase is blocked until a testnet RPC + funded deployer key is explicitly provided by the operator — this is a "credentials" stop condition per the orchestration rules, not something to work around. |

## AI/ML tooling

| Name | Type | Purpose | Available? | Auth required? | AION phase | Risk | Planned usage |
|---|---|---|---|---|---|---|---|
| Hugging Face MCP (`mcp__Hugging_Face__*`) | model/dataset hub | Search/inspect models, datasets, spaces | Available | Authenticated as `HBKcustoms` | Phase 4 (AI execution), model registry | Low (read-heavy) | Candidate for model manifest metadata lookups later; not used this milestone |
| Local Ollama/vLLM/llama.cpp | inference runtime | Actually run models | **Not installed** in this container | N/A | Phase 4 | N/A | Simulator (Phase 2) stubs execution instead of running real weights — no GPU in this container |

## Irrelevant-to-AION connectors present in this session

Amplitude, Apollo.io, Asana, Figma, Gamma, Gmail, Google Calendar/Drive, HubSpot, Slack, Spotify, Supabase, Vercel, Zapier, Composio are all present as MCP servers in this account but have **no relevant AION phase**. Per the "do not invoke irrelevant tools merely to claim they were used" rule, none of these are used in this project.

## Skills (this repo, project-scoped)

Created this session under `.claude/skills/aion-*` — see `docs/research/RESEARCH-LOG.md` entry 2026-07-28 for the list and `.claude/skills/` for definitions. These load only when this repo is the working directory.

## Subagents (this repo, project-scoped)

Created this session under `.claude/agents/aion-*.md` — 13 specialist roles per the orchestration brief (Research, Protocol Architect, Distributed Systems, AI Systems, Verification, Crypto, Smart Contracts, Economics, Security, Red Team, UI/UX, QA, DevOps). Each is a real Claude Code subagent definition (frontmatter: name, description, tools, model) that can be invoked via the `Agent` tool with that `subagent_type` in future sessions on this repo.

## Gaps identified and how they're handled

| Missing capability | Official equivalent | Status |
|---|---|---|
| Solidity toolchain (Foundry) | `foundryup` (getfoundry.sh) | Not installed; deferred to Phase 10, will be installed then, not faked now |
| Chain RPC / funded testnet wallet | Base Sepolia / OP Sepolia public RPC + operator-provided key | Blocked on operator-supplied credentials; explicit stop condition |
| GPU / real inference runtime | Ollama (local) | Not present in this container; Phase 2 simulator models job execution abstractly instead of running real inference |
| ZKML tooling (EZKL) | `pip install ezkl` | Not installed; deferred to Phase 11 proof-of-concept, install then |

No tool, skill, plugin, MCP server, or slash command is referenced in project docs unless it is listed above as Available.
