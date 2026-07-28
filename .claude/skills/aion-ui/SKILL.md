---
name: aion-ui
description: Implement AION UI components/screens under apps/web, apps/node-desktop, packages/ui, matching the premium/technical dark design language.
---

## Purpose
Build interfaces that look like serious infrastructure, not speculative-crypto branding.

## Trigger conditions
Any component/screen implementation for `apps/web`, `apps/node-desktop`, or `packages/ui`.

## Allowed tools
Read, Grep, Glob, Write, Edit, Bash.

## Preferred sources
shadcn/ui official docs/components (verify current API before generating UI, never guess component props), `docs/ARCHITECTURE.md`.

## Workflow
1. Check shadcn's actual current component composition/API before writing usage code.
2. Use design tokens (background/surface/border/foreground/muted/primary/success/warning/danger/info/chart-series) consistently; support light/dark even if dark is primary.
3. Any data shown that isn't from a real backend must be visibly marked as mock.
4. Never make token price a headline metric.

## Required checks
No guessed component API; no unmarked mock data; keyboard/contrast/reduced-motion support present.

## Expected outputs
Working UI code matching the visual system and accessibility bar.

## Failure conditions
Cheap-crypto aesthetic (neon gradients, meme styling, fake metrics); unmarked mock data presented as real.
