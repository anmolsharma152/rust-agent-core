# Rust RAG Agent Architecture & Design Specification

## 1. Overview

`rust-rag-agent` is a lightweight, high-performance Agentic RAG CLI application built in Rust. It combines:
- **Local CPU Vector Embeddings**: Powered by `fastembed` (ONNX Runtime, BGE-small-en model).
- **Agentic Multi-Turn Tool Calling**: Driven by Groq LPUs (`llama-3.3-70b-versatile`) via OpenAI-compatible REST APIs.
- **In-Memory Vector Search**: Pure Rust cosine similarity search over local documents.

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
             |                                     |
             | Vectors                             | REST (OpenAI API)
             v                                     v
  +----------+----------+               +----------+----------+
  | FastEmbed ONNX Model|               | Groq Cloud API      |
  | (src/embeddings.rs) |               | (Llama 3.3 70B)     |
  +---------------------+               +---------------------+
```

---

## 2. Module Breakdown

### `src/main.rs`
- **Role**: Application entry point & configuration manager.
- **Key Responsibilities**:
  - Initializes `.env` variables via `dotenvy`.
  - Downloads/loads the `fastembed` ONNX model.
  - Loads and vectorizes all text files in `./docs/`.
  - Registers LLM providers (Groq API primary).
  - Drives the interactive terminal REPL loop.

### `src/agent.rs`
- **Role**: Agent execution loop (`Agent`, `run_loop`, `retrieve`).
- **Key Responsibilities**:
  - Manages multi-turn conversation history (`Vec<Message>`).
  - Passes available `ToolDef` schemas to the LLM.
  - Intercepts tool calls issued by the LLM (e.g. `search_documents`).
  - Executes document retrieval and feeds formatted context back to the LLM.
  - Limits execution to a maximum of 5 turns per query.

### `src/llm.rs`
- **Role**: Universal OpenAI-compatible API Client (`LlmClient`).
- **Key Responsibilities**:
  - Encapsulates HTTP chat requests (`/v1/chat/completions`) using `reqwest` and `serde`.
  - Serializes `ToolDef` function calling schemas (`search_documents`).
  - Handles bearer authorization and error response parsing.

### `src/embeddings.rs`
- **Role**: Local Vector Embedding Service (`Embedder`).
- **Key Responsibilities**:
  - Wraps `fastembed::TextEmbedding` (`EmbeddingModel::BGESmallENV15`).
  - Appends BGE task prefixes: `passage: ` for indexing and `query: ` for search vectors.
  - Operates 100% locally on CPU without network I/O after initial model download.

### `src/store.rs`
- **Role**: In-Memory Vector Store (`DocStore`).
- **Key Responsibilities**:
  - Loads `.txt` (and `.md`) files from `./docs/`.
  - Stores document text alongside pre-computed 384-dimensional `f32` vectors.
  - Computes exact cosine similarity scores (`dot / (norm_a * norm_b)`).
  - Returns top $K$ ranked matches.

---

## 3. Environment & Configuration

All environment variables can be set in a `.env` file in the project root:

```env
# Required for Cloud Inference:
GROQ_API_KEY=gsk_...

# Optional Model Overrides:
GROQ_MODEL=llama-3.3-70b-versatile
OLLAMA_MODEL=llama3.2
```

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

    User->>Agent: Send query (e.g. "Explain RAG basics")
    Agent->>LLM: POST /chat/completions (messages + tools)
    LLM-->>Agent: Return ToolCall (search_documents query="RAG basics")
    Agent->>Embedder: embed_query("RAG basics")
    Embedder-->>Agent: Return query vector [f32; 384]
    Agent->>Store: search(query_vector, top_k=3)
    Store-->>Agent: Return top 3 doc passages + similarity scores
    Agent->>LLM: POST /chat/completions (tool result message)
    LLM-->>Agent: Return final synthesized answer
    Agent-->>User: Display answer in terminal REPL
```
