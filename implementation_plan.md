# Comprehensive Master Implementation Plan: `rust-agent-core`

This comprehensive master implementation plan details the architecture and roadmap for expanding `rust-agent-core` into an autonomous, self-contained AI Agent Engine and cross-platform CLI tool with dual-layer memory, hybrid RRF search, observability, and zero-dependency packaging.

---

## Architectural Decision Summary

> [!IMPORTANT]
> **Custom Agent Loop (No `rig-core`)**: The system uses a custom-built, lightweight async agent loop in `src/agent.rs`. Third-party frameworks like `rig-core` are excluded to maintain 100% fine-grained control and zero framework overhead.

> [!IMPORTANT]
> **No MCP Servers Needed**: The agent does NOT use Model Context Protocol (MCP) servers or IPC subprocesses. All tools (web search, document search, system commands, file I/O) are implemented as native Rust functions (`ToolDef` schemas in `src/llm.rs`) or called via direct REST APIs.

> [!NOTE]
> **Single Binary Execution**: The application strictly remains a 100% self-contained single Rust binary. External database daemons (Qdrant, PostgreSQL, Milvus, Docker) are excluded.

> [!TIP]
> **Fast LPU Engine (Qwen 3.6 27B on Groq)**: Default model set to `qwen/qwen3.6-27b` on Groq with OpenRouter and Gemini fallbacks for native token-level function calling, zero raw XML syntax corruption, sub-second response times, and exponential backoff retry handling.

---

## Completed Phases

### Phase 1: Tool-Calling Stabilization & Groq Qwen 3.6 27B Upgrade [COMPLETED]
- Set `temperature: 0.0` for deterministic parameter generation.
- Added `maxLength: 120` to `search_documents` parameters schema.
- Added `list_documents_tool()`, `web_search_tool()`, `Message::assistant()`, and exponential backoff retry loop in `src/llm.rs`.
- Configured Groq (`qwen/qwen3.6-27b`) as primary provider with OpenRouter and Gemini fallbacks in `src/main.rs`.

### Phase 2: Multi-Format Ingestion, Sliding-Window Chunking & Disk Vector Cache [COMPLETED]
- Multi-format file parsing added for `.txt`, `.md` (Markdown), `.pdf` (PDF pages), `.csv` (Row-to-text key-values), and `.json` files.
- Sliding-window text chunker (300 words with 50-word sliding overlap) implemented in `src/store.rs`.
- Binary disk vector cache (`.vector_cache.bin`) using SHA-256 file checksums implemented in `src/store.rs` for 0-second instant startup.

### Phase 3: System & Web Tool Expansion [COMPLETED]
- Implemented `list_dir`, `read_file`, `write_file`, sandboxed `run_command`, and keyless/API `web_search` tools in `src/agent.rs`.

### Phase 4: Interactive Safety Guardrails [COMPLETED]
- Implemented `ExecutionMode` enum (`Safe`, `ReadOnly`, `Yolo`).
- `--safe-mode` (default): Prompts `[y/N]` before executing shell commands or writing files.
- `--read-only`: Auditing mode blocking destructive operations.
- `--yolo`: Fully autonomous execution mode.

### Phase 5: REPL UX & Command Interceptors [COMPLETED]
- Implemented zero-latency command interceptors in `src/main.rs`: `exit`, `quit`, `:q`, `clear`, and `help`.
- Intercepts built-in commands instantly without making LLM API calls over the network.

---

## Upcoming Technical Phases

### Phase 6: Dual-Layer Agent Memory & Token Budgeting [CURRENT / HIGH PRIORITY]
#### [NEW] [src/memory.rs](file:///home/anmol/Projects/rust-agent-core/src/memory.rs)
- **Short-Term Sliding Window Buffer**: Maintain active window of last 4–6 turns (~1,200 token budget) to prevent hitting Groq 8,000 TPM Free Tier rate limits.
- **Fact Extraction & Summarization**: When turns slide out of short-term window, extract key facts (e.g. *"User's name is Anmol"*).
- **Long-Term Memory Store (`.memory_cache.bin`)**: Embed extracted facts into a dedicated memory vector index saved to disk.
- **Semantic Memory Recall**: Perform local vector search over past facts before LLM invocation and inject top-relevant facts (~100 tokens) into system prompt.

---

### Phase 7: Hybrid Retrieval (BM25 + Vector RERANK via RRF) [HIGH PRIORITY]
#### [MODIFY] [src/store.rs](file:///home/anmol/Projects/rust-agent-core/src/store.rs)
- **BM25 Tokenizer & Keyword Index**: Build in-memory BM25 index alongside vector embeddings to catch exact code symbols (e.g. `check_permission`, `cosine_similarity`).
- **Reciprocal Rank Fusion (RRF)**: Implement RRF algorithm combining BM25 keyword ranks and vector similarity ranks:
  $$\text{RRF\_Score}(d) = \frac{1}{60 + \text{Rank}_{\text{BM25}}(d)} + \frac{1}{60 + \text{Rank}_{\text{Vector}}(d)}$$

---

### Phase 8: Telemetry, Observability & Secret Masking
#### [MODIFY] [src/main.rs](file:///home/anmol/Projects/rust-agent-core/src/main.rs) & [src/agent.rs](file:///home/anmol/Projects/rust-agent-core/src/agent.rs)
- **`--verbose` Mode**: Display turn latency (ms), active LLM provider, tokens used / TPM percentage, and memory recall metrics after every turn.
- **Secret & Key Masking**: Automatically redact API keys (`GROQ_API_KEY`, AWS tokens, SSH keys) from logs and output text.
- **Path Boundary Enforcer**: Restrict file read/write operations to safe directory bounds.

---

### Phase 9: Session Persistence & Checkpoints (`--resume`)
#### [NEW] [src/session.rs](file:///home/anmol/Projects/rust-agent-core/src/session.rs)
- **Session Checkpoints**: Save conversation `Vec<Message>` state to `~/.config/rust-agent-core/sessions/<session_id>.json`.
- **CLI Flags**: `--resume <session_id>` and `--list-sessions`.

---

### Phase 10: Async Sub-Agent Worker Swarms (`spawn_subagent`)
#### [MODIFY] [src/agent.rs](file:///home/anmol/Projects/rust-agent-core/src/agent.rs)
- **Sub-Agent Delegation**: Allow main agent loop to spawn background worker tasks via `tokio::spawn` and `tokio::sync::mpsc` channels.
- **Parallel Execution**: Concurrently execute web searches, document retrievals, and shell diagnostics.

---

### Phase 11: Cross-Platform Packaging & Distribution
#### [MODIFY] [Cargo.toml](file:///home/anmol/Projects/rust-agent-core/Cargo.toml) & GitHub Actions
- **`cargo-dist` Integration**: Automate pre-compiled binary releases for Linux (x86_64, aarch64), macOS (Apple Silicon / Intel), and Windows.
- **Distribution Channels**: Crates.io (`cargo install rust-agent-core`), Arch AUR (`PKGBUILD`), and GitHub Release tarballs.

---

## Verification Plan

### Automated Tests
- `cargo check`: Verify schema typing and crate linkage.
- `cargo test`: Unit tests for sliding-window memory buffer, BM25 tokenizer, and RRF rank fusion.
- `cargo build --release`: Verify single-binary compilation.

### Manual Verification
- Test `--safe-mode` interactive prompt `[y/N]` when agent calls `run_command` or `write_file`.
- Test `--read-only` flag blocking mutating tools with permission error.
- Test `--yolo` flag executing tools autonomously without prompts.
- Test document retrieval across mixed `.pdf`, `.csv`, `.md`, `.json`, and `.txt` files.
- Test 0-second instant startup from `.vector_cache.bin`.
- Test live web search queries.
- Test `exit`, `quit`, and `:q` command interceptors in REPL.
- Test 10+ turn conversation to verify token usage stays under 1,500 tokens (Groq 8k TPM limit never hit).
- Test long-term memory recall ("what is my name?").
- Test hybrid RRF keyword search for exact code symbol queries.
- Test `--verbose` telemetry output.
