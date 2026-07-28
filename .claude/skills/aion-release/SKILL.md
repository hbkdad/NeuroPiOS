---
name: aion-release
description: Prepare an AION release - version bumps, changelogs, tagged builds. Testnet-only; no mainnet/production token release without explicit operator authorization.
---

## Purpose
Package and tag AION releases safely within the staged token/deployment policy.

## Trigger conditions
A request to cut a release, bump a version, or prepare release notes.

## Allowed tools
Read, Grep, Glob, Write, Edit, Bash.

## Preferred sources
`docs/adr/0004-token-development-staging.md` (staged deployment policy), `docs/ROADMAP.md` (what phase the release corresponds to).

## Workflow
1. Confirm what's actually shipping (which phase, which components) before writing release notes — don't describe unbuilt features as released.
2. Version bump follows the protocol versioning rule in `docs/PROTOCOL.md` if protocol objects changed.
3. Any release touching settlement/contracts must state explicitly which stage (A-G per ADR-0004) it corresponds to; Stage G (production) requires explicit prior operator authorization and is never initiated by this skill on its own.

## Required checks
Release notes list only what actually exists and passes tests in this repo at release time.

## Expected outputs
Accurate release notes / changelog / version tag.

## Failure conditions
Release notes overstating what shipped; any path from this skill to an unauthorized mainnet/production release.
