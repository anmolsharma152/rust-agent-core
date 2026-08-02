# Rust Agent Core Master Implementation Plan

This master implementation plan details the architecture and roadmap for expanding `rust-agent-core` into a fully autonomous, custom-built Rust AI Agent Engine and cross-platform CLI tool.

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

## Tool Suite Specification (Native Rust & REST APIs)

The Custom Agent Loop supports a comprehensive, native tool suite:

1. **`list_documents`**: Returns the array of ingested filenames in `DocStore` (prevents vector search hallucinations when users ask for file metadata).
2. **`search_documents`**: RAG vector similarity search over chunked document passages (`maxLength: 120` query constraint).
3. **`web_search`**: Live internet queries via keyless DuckDuckGo HTML scraping or optional Tavily API (`TAVILY_API_KEY`).
4. **`list_dir`**: Browse local project directories (`std::fs::read_dir`).
5. **`read_file`**: Read text from specific files on disk (`std::fs::read_to_string`).
6. **`write_file`**: Create or update local project files.
7. **`run_command`**: Execute sandboxed terminal/bash commands (`tokio::process::Command`).
8. **REST API Integrations**: Direct REST calls using `reqwest` for external cloud services (GitHub, Gmail, Notion).

---

## Technical Roadmap & Technical Phases

### Phase 1: Tool-Calling Stabilization & Qwen 3.6 27B Upgrade [COMPLETED]
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

### Phase 4: Interactive Safety & Permission Guardrails [UPCOMING]
#### [MODIFY] [src/main.rs](file:///home/anmol/Projects/rust-agent-core/src/main.rs) & [src/agent.rs](file:///home/anmol/Projects/rust-agent-core/src/agent.rs)
- **Execution Modes**:
  - `--safe-mode` (default): Prompts user for interactive confirmation `[y/N]` before executing mutating tools (`run_command`, `write_file`).
  - `--read-only`: Disables destructive file writes and terminal command execution for safe auditing.
  - `--yolo`: Autonomous execution without interactive prompts.

---

### Phase 5: Hybrid Retrieval (BM25 Keyword + Vector RERANK) [UPCOMING]
#### [MODIFY] [src/store.rs](file:///home/anmol/Projects/rust-agent-core/src/store.rs)
- **BM25 Keyword Indexing**: Implement in-memory BM25 index alongside vector embeddings to catch exact code symbols (e.g. `cosine_similarity`, variable names, exact error codes).
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
- **Package Managers**: Publish to Crates.io (`cargo install rust-agent-core`), Homebrew formula, and GitHub Releases tarballs.

---

## Verification Plan

### Automated Tests
- `cargo check`: Verify schema typing and crate linkage.
- `cargo test`: Unit tests for sliding-window chunker, BM25 tokenizer, and binary vector serialization.
- `cargo build --release`: Verify optimized single-binary compilation.

### Manual Verification
- Test interactive REPL chat with multi-turn memory.
- Test document retrieval across mixed `.pdf`, `.csv`, `.md`, `.json`, and `.txt` files.
- Test 0-second instant startup from `.vector_cache.bin`.
- Test live web search queries.
- Test file read/write and shell command execution.
