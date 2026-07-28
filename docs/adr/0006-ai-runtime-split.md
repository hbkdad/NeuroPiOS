# ADR 0006: AI Runtime — Split by Node Role, No Premature Ray

## Status
Accepted

## Context
2026 research on LLM-serving engines (see `RESEARCH-LOG.md`) shows meaningfully different tools winning for different deployment shapes: single-user/consumer nodes vs. multi-tenant GPU-provider nodes vs. training/fine-tuning. HuggingFace TGI is now in maintenance mode and explicitly points users to vLLM/SGLang/llama.cpp instead.

## Decision
Split runtime choice by `aion-node` role rather than picking one universal engine:
- **Consumer/desktop compute nodes:** Ollama (built on llama.cpp), GGUF quantization — optimized for ease of setup over throughput, matching this node class's actual workload (occasional single-request inference).
- **GPU-provider inference nodes:** vLLM as the default; SGLang as an alternative where the provider's workload is share-context-heavy (chatbots/RAG/agents) since RadixAttention gives ~29% throughput advantage there.
- **Training/fine-tuning nodes:** PyTorch + Transformers/PEFT for LoRA-style fine-tuning (Phase 4+ of the model-improvement loop).
- **Distributed orchestration (Ray):** explicitly not adopted yet — no current AION workload has been shown to need it; revisit only when a specific multi-node training/serving requirement can't be met by the above.
- **Explicitly excluded:** HuggingFace TGI (maintenance mode, upstream-deprecated as of 2026).

## Reasoning
- Matches the brief's explicit per-node-class guidance and its instruction not to introduce distributed infrastructure prematurely.
- Runtime selection is a node-capability-detection concern (`aion-node` auto-detects OS/GPU/VRAM per the Node Architecture requirements) — the node advertises which runtime(s) it supports rather than the protocol hardcoding one.

## Consequences
- `crates/aion-runtime` needs an abstraction boundary (a `RuntimeBackend` trait) so job execution code isn't coupled to any one engine's API — this is a real design constraint for Phase 4, not just a config toggle.
- Model manifests (`docs/PROTOCOL.md` — ModelManifest) must record which runtime(s)/quantization formats a given model artifact is compatible with, since not every model works on every backend.
