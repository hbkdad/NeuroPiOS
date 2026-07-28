# Research Log

Append-only. Each entry: date, question, sources, findings, decision impact.

---

## 2026-07-28 — Competing decentralized AI/compute networks: what should AION borrow/avoid?

**Sources:** see `docs/research/competitors/{bittensor,gensyn,akash,render,allora}.md` for full citation lists.

**Findings:**
- No competing network currently ties rewards to a **requester-anchored, escrow-backed job** the way the orchestration brief requires. Bittensor scores ongoing subjective validator opinion; Akash/Render verify nothing about output quality; Allora is closest (ground-truth accuracy scoring) but its zkML "auditable AI" claim is not corroborated by current zkML proving-cost research and should be treated skeptically.
- Gensyn's Verde/REE optimistic-dispute pattern (commit → challenge window → bisection arbitration) is the most directly reusable verification idea for AI workloads specifically, because it doesn't assume bit-exact reproducibility.
- Reverse-auction compute marketplaces (Akash) are commercially viable today without solving AI verification at all — validates a phased approach where AION's compute-marketplace leg can bootstrap ahead of its verification/PoIG maturity.

**Decision impact:** Informs ADR-0003 (verification tiering — adopts Gensyn-style optimistic dispute as Level 4) and the PoIG "Useful Work Score" design (rejects Bittensor/Akash-style "trust the validator/provider" models in favor of signed, escrow-backed job commitments).

---

## 2026-07-28 — Settlement layer: custom L1 vs. existing L2 vs. app-chain

**Sources:** see `docs/adr/0001-settlement-layer.md`.

**Findings:** ERC-4337 + EIP-7702 account abstraction is production-grade across major L2s (Base, Optimism, Arbitrum, Polygon zkEVM) as of mid-2026, with tens of millions of smart-account deployments and mature paymaster infrastructure. This directly satisfies the orchestration brief's wallet-UX goals (gas sponsorship, session keys, no manual gas management) without AION needing to build any of that itself.

**Decision impact:** Confirms the Stage-0 hypothesis — no technical justification found for a custom L1/appchain at this stage. See ADR-0001.

---

## 2026-07-28 — zkML production maturity for AI-execution verification

**Sources:** EZKL blog benchmarks, Bionetta (arXiv 2510.06784), NANOZK (arXiv 2603.18046), Optimistic TEE-Rollups (arXiv 2512.20176).

**Findings:** EZKL 1.0 handles ONNX models up to ~50M parameters with sub-second proving for MNIST-scale classification; full LLM-inference zkML proving remains economically impractical in 2026. Hybrid optimistic+TEE approaches are an active research direction for scaling verifiable generative-AI inference.

**Decision impact:** ZKML is scoped to Phase 11 as a narrow proof-of-concept (small ONNX model only), not a general verification mechanism — confirmed in `AI-VERIFICATION-MATRIX.md` and ADR-0003.

---

## 2026-07-28 — P2P stack and local inference runtime choices

**Sources:** rust-libp2p docs/GitHub, 2026 LLM-serving-engine comparisons (multiple).

**Findings:** rust-libp2p's GossipSub + Kademlia combination (Kademlia for peer discovery, Identify for capability exchange, GossipSub for pub/sub messaging) remains the standard P2P stack for this workload shape. On the runtime side: Ollama/llama.cpp for single-node consumer inference, vLLM/SGLang for multi-tenant GPU-provider serving (SGLang ~29% faster on shared-context workloads via RadixAttention), HuggingFace TGI is now in maintenance mode (explicitly deprecated in favor of vLLM/SGLang/llama.cpp).

**Decision impact:** Confirms the Stage-0 hypothesis for P2P (rust-libp2p) and refines the AI-runtime ADR to explicitly avoid TGI and split consumer-node vs. provider-node runtime choice.
