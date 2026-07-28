---
name: aion-research
description: Research AION-relevant technologies, competitors, or verify a claim in docs/research is current. Use when asked to research a technology choice, compare architectures, or check whether a dependency/approach referenced in AION docs is still accurate/current.
---

## Purpose
Ground AION architectural decisions in current, primary-source research rather than training-data memory or marketing content.

## Trigger conditions
User asks to research a technology, compare alternatives for AION, update `docs/research/`, or verify a version/capability claim before it's relied on in an ADR or spec.

## Allowed tools
WebSearch, WebFetch, Read, Grep, Glob, Write.

## Preferred sources (in order)
Official specs > official docs > official GitHub repos > peer-reviewed research > respected security research > maintained OSS implementations > technical community discussion. Never marketing sites, token price, influencer claims, or SEO content as the deciding source.

## Workflow
1. Check `docs/research/CAPABILITY-MATRIX.md` and `docs/research/RESEARCH-LOG.md` for existing findings before re-researching.
2. Search current sources (use the current year in queries).
3. Compare at least two alternatives where a choice is being made; don't accept the first result.
4. Append a dated entry to `docs/research/RESEARCH-LOG.md`.
5. If findings materially change an existing ADR's basis, flag it (new/superseding ADR is the protocol architect's job, not this skill's).

## Required checks
Every cited fact traces to an actual fetched/searched source. No invented URLs or version numbers.

## Expected outputs
Updated `docs/research/RESEARCH-LOG.md` entry, and/or a new/updated `docs/research/competitors/*.md` or comparison doc.

## Failure conditions
Citing an unfetched source; treating single-source marketing content as authoritative; skipping the research log entry.
