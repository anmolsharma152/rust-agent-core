# Rust Agent Master Implementation Plan

This master implementation plan details the architecture and roadmap for expanding `rust-rag-agent` into a fully autonomous, custom-built Rust AI Agent.

---

## Architectural Decision Summary

> [!IMPORTANT]
> **Custom Agent Loop (No `rig-core`)**: The system will use a custom-built, lightweight async agent loop in `src/agent.rs`. Third-party frameworks like `rig-core` are excluded to maintain 100% fine-grained control and zero framework overhead.

> [!IMPORTANT]
> **No MCP Servers Needed**: The agent will NOT use Model Context Protocol (MCP) servers or IPC subprocesses. All tools (web search, document search, system commands, API integrations) will be implemented as native Rust functions (`ToolDef` schemas in `src/llm.rs`) or called via direct REST APIs.

> [!NOTE]
> **Single Binary Execution**: The application strictly remains a 100% self-contained single Rust binary. External database daemons (Qdrant, PostgreSQL, Milvus, Docker) are excluded.

> [!TIP]
> **Recommended LLM Engine (Qwen 2.5)**: Default model set to `qwen-2.5-coder-32b` or `qwen-2.5-72b-instruct` on Groq to eliminate Llama 3.3 raw tag syntax corruptions (`<function=...`) and tool over-triggering.

---

## Tool Suite Specification (Native Rust & REST APIs)

The Custom Agent Loop will support a comprehensive, native tool suite:

1. **`list_documents`**: Returns the array of ingested filenames in `DocStore` (prevents vector search hallucinations when users ask for file metadata).
2. **`search_documents`**: RAG vector similarity search over chunked document passages (`maxLength: 120` query constraint).
3. **`web_search`**: Live internet queries via Tavily API or Brave Search API.
4. **`list_dir`**: Browse local project directories (`std::fs::read_dir`).
5. **`read_file`**: Read text from specific files on disk (`std::fs::read_to_string`).
6. **`write_file`**: Create or update local project files.
7. **`run_command`**: Execute sandboxed terminal/bash commands (`tokio::process::Command`).
8. **REST API Integrations (Composio / Cloud APIs)**: Direct REST calls using `reqwest` for external services (Gmail, GitHub, Notion) without MCP server subprocesses.

---

## Technical Roadmap & Proposed Changes

### Phase 1: Tool-Calling Stabilization & Qwen 2.5 Upgrade

#### [MODIFY] [llm.rs](file:///home/anmol/Projects/rust-rag-agent/src/llm.rs)
- **Fix Syntax Corruption**: Set `temperature: 0.0` for deterministic parameter generation.
- **Query Length Constraints**: Add `maxLength: 120` to `search_documents` parameters schema.
- **Add `list_documents_tool()`**: Dedicated tool schema returning ingested filenames.
- **Add `web_search_tool()`**: Tool schema for Tavily/Brave live web search.

#### [MODIFY] [agent.rs](file:///home/anmol/Projects/rust-rag-agent/src/agent.rs)
- **Clean System Prompt**: Remove manual tool syntax text from `SYSTEM_PROMPT` to prevent double-prompting tag corruption.
- **Multi-Tool Dispatcher**: Handle `list_documents`, `search_documents`, `web_search`, and file system tools inside the custom `run_loop`.

#### [MODIFY] [.env](file:///home/anmol/Projects/rust-rag-agent/.env)
- Set default model to `GROQ_MODEL=qwen-2.5-coder-32b` for top-tier function calling precision.

---

### Phase 2: Multi-Format Ingestion & Sliding-Window Chunking

#### [MODIFY] [Cargo.toml](file:///home/anmol/Projects/rust-rag-agent/Cargo.toml)
Add lightweight parsing crates:
- `pdf-extract` for PDF page text extraction.
- `csv` for tabular row-to-text formatting (`Header: Value`).
- `pulldown-cmark` for Markdown parsing.

#### [MODIFY] [store.rs](file:///home/anmol/Projects/rust-rag-agent/src/store.rs)
- **Multi-Format Loader**: Parse `.txt`, `.md`, `.pdf`, `.csv`, and `.json` files in `./docs/`.
- **Sliding-Window Chunker**: Split long documents into 300–400 word passages with 50-word sliding overlap.
- **Binary Disk Cache**: Implement `./docs/.vector_cache.bin` storing SHA-256 checksums and pre-computed vectors for instant startup without re-embedding.

---

### Phase 3: System & API Tool Expansion

#### [MODIFY] [src/agent.rs](file:///home/anmol/Projects/rust-rag-agent/src/agent.rs) & [src/llm.rs](file:///home/anmol/Projects/rust-rag-agent/src/llm.rs)
- Implement `list_dir`, `read_file`, `write_file`, and sandboxed `run_command` tools.
- Add REST client for external cloud service integrations (Composio / GitHub / Gmail REST endpoints).

---

## Verification Plan

### Automated Tests
- Run `cargo check` to verify schema typing and crate linkage.
- Run `cargo build --release` to verify single-binary compilation.

### Manual Verification
- Test `list_documents` query in CLI REPL.
- Test queries asking for general world facts to verify Qwen answers directly without over-triggering tools.
- Test web search queries using live Tavily / Brave API calls.
- Test loading mixed `.pdf`, `.csv`, `.md`, and `.txt` files from `./docs/`.
