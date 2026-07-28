# crates/aion-runtime — AI Execution Backend Abstraction (planned, Phase 4)

Defines a `RuntimeBackend` trait so job execution is not coupled to one inference engine; concrete backends per `docs/adr/0006-ai-runtime-split.md` (Ollama/llama.cpp for consumer nodes, vLLM/SGLang for GPU-provider nodes, PyTorch+PEFT for fine-tuning nodes). **Status: not started** — no GPU/inference runtime exists in the current development container (see `docs/research/CAPABILITY-MATRIX.md`); implementation requires a real environment.
