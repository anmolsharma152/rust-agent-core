# Rust RAG Agent Master Implementation Plan

This implementation plan incorporates all technical decisions, architecture choices, and bug fixes discussed across recent sessions for `rust-rag-agent`.

---

## Architectural Decision Summary

> [!IMPORTANT]
> **Single Binary Constraint**: The application will strictly remain a 100% self-contained single Rust binary. External database daemons (Qdrant, PostgreSQL, Milvus, Docker) are excluded.

> [!TIP]
> **Recommended LLM Engine (Qwen 2.5)**: Default model updated to `qwen-2.5-coder-32b` or `qwen-2.5-72b-instruct` on Groq to eliminate Llama 3.3 raw tag syntax corruptions (`<function=...`) and tool over-triggering.

---

## Technical Roadmap & Proposed Changes

### Phase 1: Tool-Calling Stabilization & Qwen 2.5 Upgrade

#### [MODIFY] [llm.rs](file:///home/anmol/Projects/rust-rag-agent/src/llm.rs)
- **Fix Syntax Corruption**: Set `temperature: 0.0` for deterministic parameter generation.
- **Query Length Constraints**: Add `maxLength: 120` to `search_documents` parameters schema to prevent 10,000-word runaway string loops.
- **Add `list_documents_tool()`**: Define a dedicated tool returning the full array of ingested filenames in `DocStore` so the LLM doesn't rely on `search_documents(top_k=3)` for file metadata.

#### [MODIFY] [agent.rs](file:///home/anmol/Projects/rust-rag-agent/src/agent.rs)
- **Clean System Prompt**: Remove manual tool syntax text from `SYSTEM_PROMPT` to prevent double-prompting tag corruption.
- **Dispatch `list_documents`**: Add execution handler returning the list of document titles.

#### [MODIFY] [.env](file:///home/anmol/Projects/rust-rag-agent/.env)
- Update default model to `GROQ_MODEL=qwen-2.5-coder-32b` for top-tier function calling precision.

---

### Phase 2: Multi-Format Ingestion & Sliding-Window Chunking

#### [MODIFY] [Cargo.toml](file:///home/anmol/Projects/rust-rag-agent/Cargo.toml)
Add parsing crates:
- `pdf-extract` for PDF page text extraction.
- `csv` for tabular row-to-text formatting (`Header: Value`).
- `pulldown-cmark` for Markdown parsing.

#### [MODIFY] [store.rs](file:///home/anmol/Projects/rust-rag-agent/src/store.rs)
- **Multi-Format Loader**: Parse `.txt`, `.md`, `.pdf`, `.csv`, and `.json` files in `./docs/`.
- **Sliding-Window Chunker**: Split long documents into 300–400 word passages with 50-word sliding overlap.
- **Binary Disk Cache**: Implement `./docs/.vector_cache.bin` storing SHA-256 checksums and pre-computed vectors for instant startup without re-embedding.

---

### Phase 3: System & Web Search Tool Expansion

#### [MODIFY] [llm.rs](file:///home/anmol/Projects/rust-rag-agent/src/llm.rs) & [agent.rs](file:///home/anmol/Projects/rust-rag-agent/src/agent.rs)
- **Web Search Tool (`web_search`)**: Integrate Tavily / Brave Search API for live internet queries.
- **System Tools (Optional)**: Add `list_dir`, `read_file`, `write_file`, and sandboxed `run_command` handlers.

---

### Phase 4: Documentation & Synchronization

#### [MODIFY] [README.md](file:///home/anmol/Projects/rust-rag-agent/README.md) & [ARCHITECTURE.md](file:///home/anmol/Projects/rust-rag-agent/ARCHITECTURE.md)
Update user guides, sequence diagrams, and Qwen model setup instructions.

#### [MODIFY] [implementation_plan.md](file:///home/anmol/Projects/rust-rag-agent/implementation_plan.md)
Keep project root implementation plan 100% in sync with Antigravity artifact copy.

---

## Verification Plan

### Automated Tests
- Run `cargo check` to verify schema typing and crate linkage.
- Run `cargo build --release` to verify single-binary compilation.

### Manual Verification
- Test `list_documents` query in CLI REPL.
- Test queries asking for general world facts to verify Qwen answers directly without over-triggering tools.
- Test loading mixed `.pdf`, `.csv`, `.md`, and `.txt` files from `./docs/`.
