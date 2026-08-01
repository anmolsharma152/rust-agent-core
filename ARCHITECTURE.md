# Rust RAG Agent Architecture & Design Specification

## 1. Overview & Core Philosophy

`rust-rag-agent` is a lightweight, high-performance Agentic RAG system built in Rust. 

### Core Architectural Mandate: Single Binary Execution
- **Zero Runtime Dependencies**: Does **NOT** require Docker, PostgreSQL, Qdrant, or any external vector database servers.
- **Embedded Local AI Engine**: Runs local vector embeddings on CPU via `fastembed` (ONNX Runtime, BGE-small-en model).
- **In-Process Vector Store & Cache**: Performs fast cosine similarity vector search in CPU memory, with binary disk caching (`.vector_cache.bin`) for instant persistence across restarts.
- **Cloud/Local Agentic Tool Calling**: Driven by Groq LPUs (`llama-3.3-70b-versatile`) with optional local Ollama fallback.

```
                      +-------------------+
                      |   User (Terminal) |
                      +---------+---------+
                                | Query
                                v
                      +---------+---------+
                      |   Agent REPL      |
                      |   (src/main.rs)   |
                      +---------+---------+
                                |
             +------------------+------------------+
             |                                     |
             v                                     v
  +----------+----------+               +----------+----------+
  | Local Document Store|               |  LLM Provider       |
  |  (src/store.rs)     |               |  (src/llm.rs)       |
  +----------+----------+               +----------+----------+
  | Multi-Format Loader |               | Groq Cloud API      |
  | (.txt, .md, .pdf,   |               | (Llama 3.3 70B)     |
  |  .csv, .json)       |               +---------------------+
  | Text Chunker (400w) |                          |
  | Vector Cache (.bin) |                          v
  +----------+----------+               +---------------------+
             |                          | Web Search API      |
             | Vectors                  | (Tavily / Brave)    |
             v                          +---------------------+
  +----------+----------+
  | FastEmbed ONNX Model|
  | (src/embeddings.rs) |
  +---------------------+
```

---

## 2. Multi-Format Ingestion & Chunking Pipeline

### Multi-Format Extraction
- **`.txt` / `.md`**: Loaded via standard UTF-8 file readers (`pulldown-cmark` for Markdown parsing).
- **`.pdf`**: Text extracted page-by-page using `pdf-extract`.
- **`.csv`**: Tabular rows transformed into human-readable key-value strings (`Header1: Value1 | Header2: Value2`) via `csv` crate.
- **`.json`**: Structured objects converted to formatted context strings.

### Sliding-Window Text Chunker
To fit within the BGE embedding model's 512-token context limit:
1. Long documents are split into **300–400 word passages**.
2. A **50-word sliding overlap** is maintained between adjacent passages to prevent context loss across boundaries.

### Persistent Binary Disk Cache
- On startup, `DocStore` computes SHA-256 hashes of all files in `./docs/`.
- If checksums match `./docs/.vector_cache.bin`, vectors are reloaded instantly from disk without running embedding inferences.

---

## 3. Module Breakdown

### `src/main.rs`
- **Role**: Entry point & environment setup.
- **Responsibilities**: Loads `.env` via `dotenvy`, loads/caches vector store from `./docs/`, registers Groq API provider, and runs terminal REPL.

### `src/agent.rs`
- **Role**: Agent execution loop (`Agent`, `run_loop`, `retrieve`).
- **Responsibilities**: Manages multi-turn conversation messages, dispatches LLM tool calls (`search_documents`, `web_search`), and feeds results back to the LLM.

### `src/llm.rs`
- **Role**: Universal OpenAI-compatible REST Client (`LlmClient`).
- **Responsibilities**: Handles `/v1/chat/completions` API calls, formats `ToolDef` schemas, and parses model responses.

### `src/embeddings.rs`
- **Role**: Local Vector Embedding Service (`Embedder`).
- **Responsibilities**: Wraps `fastembed::TextEmbedding` (`BGESmallENV15`), prefixing `passage: ` for indexing and `query: ` for retrieval vectors.

### `src/store.rs`
- **Role**: In-Memory Vector Engine & Disk Cache (`DocStore`).
- **Responsibilities**: Reads multi-format files, applies text chunking, stores 384-dimensional `f32` vectors, and calculates exact cosine similarity.

---

## 4. Execution Flow Diagram

```mermaid
sequenceDiagram
    autonumber
    actor User
    participant Agent as Agent (agent.rs)
    participant LLM as Groq LLM (llm.rs)
    participant Store as Vector Store (store.rs)
    participant Embedder as FastEmbed (embeddings.rs)

    User->>Agent: Send query (e.g. "Summarize quarterly numbers from report.pdf")
    Agent->>LLM: POST /chat/completions (messages + tools)
    LLM-->>Agent: Return ToolCall (search_documents query="quarterly numbers")
    Agent->>Embedder: embed_query("quarterly numbers")
    Embedder-->>Agent: Return query vector [f32; 384]
    Agent->>Store: search(query_vector, top_k=3)
    Store-->>Agent: Return top matching PDF chunks + similarity scores
    Agent->>LLM: POST /chat/completions (tool result message)
    LLM-->>Agent: Return final synthesized answer
    Agent-->>User: Display answer in terminal REPL
```
