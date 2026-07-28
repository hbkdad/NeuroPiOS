---
name: aion-ux
description: Design AION information architecture, navigation, onboarding, and wallet/node UX flows before implementation.
---

## Purpose
Get the flow right (navigation, onboarding, wallet/node setup) before UI implementation begins.

## Trigger conditions
A request to design a new user flow, navigation structure, or onboarding sequence.

## Allowed tools
Read, Grep, Glob, Write, Edit.

## Preferred sources
`docs/ARCHITECTURE.md` (Main Web Navigation), `docs/adr/0001-settlement-layer.md` (smart-account wallet UX), orchestration brief's UI/UX sections.

## Workflow
1. Map the flow against the top-level navigation (Overview/Network/Nodes/Compute/Jobs/Models/Agents/Benchmarks/Intelligence/Explorer/Economics/Governance/Developer/Settings).
2. Wallet UX should avoid manual gas management (smart-account session keys/paymaster per ADR-0001) — don't design a flow that forces a user to hold/manage gas manually.
3. Node-setup flow must surface the Node Safety resource-cap controls explicitly, not bury them.

## Required checks
Every flow is accessible via keyboard alone; error states are described, not just visually indicated.

## Expected outputs
A flow spec (can be a doc, wireframe description, or direct implementation plan) ready to hand to aion-ui.

## Failure conditions
A flow that requires manual gas handling for an ordinary AI-job request; a resource-cap control buried or missing from node setup.
