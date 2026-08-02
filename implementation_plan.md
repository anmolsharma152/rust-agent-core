# Rust Agent Core Master Implementation Plan

This master implementation plan details the full architecture and technical roadmap for expanding `rust-agent-core` into an autonomous, self-contained Rust AI Agent Engine and cross-platform CLI product.

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

## Master Roadmap & Technical Phases

### Phase 1: Tool-Calling Stabilization & Groq Qwen 3.6 27B Upgrade [COMPLETED]
- Set `temperature: 0.0` for deterministic parameter generation.
- Added `maxLength: 120` to `search_documents` parameters schema.
- Added `list_documents_tool()`, `web_search_tool()`, `Message::assistant()`, and exponential backoff retry loop in `src/llm.rs`.
- Configured Groq (`qwen/qwen3.6-27b`) as primary provider with OpenRouter and Gemini fallbacks in `src/main.rs`.
- Multi-turn conversation memory maintained across REPL interactions in `src/agent.rs`.

### Phase 2: Multi-Format Ingestion, Sliding-Window Chunking & Disk Vector Cache [COMPLETED]
- Multi-format file parsing added for `.txt`, `.md` (Markdown), `.pdf` (PDF pages), `.csv` (Row-to-text key-values), and `.json` files.
- Sliding-window text chunker (300 words with 50-word sliding overlap) implemented in `src/store.rs`.
- Binary disk vector cache (`.vector_cache.bin`) using SHA-256 file checksums implemented in `src/store.rs` for 0-second instant startup.

### Phase 3: System & Web Tool Expansion [COMPLETED]
- Implemented `list_dir`, `read_file`, `write_file`, sandboxed `run_command`, and keyless/API `web_search` tools in `src/agent.rs`.

---

### Phase 4: Interactive Safety & Permission Guardrails [CURRENT / ACTIVE]
#### [MODIFY] [src/agent.rs](file:///home/anmol/Projects/rust-agent-core/src/agent.rs) & [src/main.rs](file:///home/anmol/Projects/rust-agent-core/src/main.rs)
- **Execution Modes**:
  - `--safe-mode` (default): Prompts user for interactive confirmation `[y/N]` on `stderr`/`stdin` before executing mutating tools (`run_command`, `write_file`).
  - `--read-only`: Auditing mode that automatically blocks destructive file writes and shell execution.
  - `--yolo`: Autonomous execution mode without interactive confirmation prompts.

---

### Phase 5: Hybrid Retrieval (BM25 Keyword + Vector RERANK) [UPCOMING]
#### [MODIFY] [src/store.rs](file:///home/anmol/Projects/rust-agent-core/src/store.rs)
- **BM25 Keyword Indexing**: Implement in-memory BM25 tokenizer alongside vector embeddings to catch exact code symbols (e.g. `cosine_similarity`, variable names, exact error codes).
- **Reciprocal Rank Fusion (RRF)**: Blend BM25 keyword ranks with Cosine Similarity vector ranks for state-of-the-art retrieval precision.

---

### Phase 6: Session Persistence & Memory Checkpoints [UPCOMING]
#### [NEW] [src/session.rs](file:///home/anmol/Projects/rust-agent-core/src/session.rs)
- **Session Checkpoints**: Save conversation `Vec<Message>` state to `~/.config/rust-agent-core/sessions/<session_id>.json`.
- **CLI Commands**:
  - `rust-agent-core --resume <session_id>`: Restores conversation history.
  - `rust-agent-core --list-sessions`: Displays saved conversation checkpoints.

---

### Phase 7: Async Sub-Agent Worker Swarms (`spawn_subagent`) [UPCOMING]
#### [MODIFY] [src/agent.rs](file:///home/anmol/Projects/rust-agent-core/src/agent.rs)
- **Sub-Agent Delegation**: Add `spawn_subagent` function schema allowing the main agent loop to spawn background worker tasks via `tokio::spawn` and `tokio::sync::mpsc` channels.
- **Parallel Work**: Concurrently execute web searches, document retrievals, and shell commands in parallel worker tasks.

---

### Phase 8: Cross-Platform Packaging & Distribution [UPCOMING]
#### [MODIFY] [Cargo.toml](file:///home/anmol/Projects/rust-agent-core/Cargo.toml) & GitHub Actions
- **`cargo-dist` Integration**: Automate binary releases for Linux (x86_64, aarch64), macOS (Apple Silicon / Intel), and Windows.
- **Package Managers**: Publish to Crates.io (`cargo install rust-agent-core`), Arch AUR (`PKGBUILD`), and GitHub Releases tarballs.

---

## Verification Plan

### Automated Tests
- Run `cargo check` to verify schema typing and crate linkage.
- Run `cargo build --release` to compile optimized single-binary release.

### Manual Verification
- Test `--safe-mode` interactive prompt `[y/N]` when agent calls `run_command` or `write_file`.
- Test `--read-only` flag blocking mutating tools with permission error.
- Test `--yolo` flag executing tools autonomously without prompts.
- Test document retrieval across mixed `.pdf`, `.csv`, `.md`, `.json`, and `.txt` files.
- Test 0-second instant startup from `.vector_cache.bin`.
- Test live web search queries.
