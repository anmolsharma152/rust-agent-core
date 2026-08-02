# rust-agent-core

A lightweight, self-contained Autonomous AI Agent Engine written in pure Rust.

- **100% Standalone Single Binary** — Self-contained execution. Requires **no Docker containers**, PostgreSQL, Qdrant, or external database daemons.
- **Custom Agent Loop (No `rig-core`)** — Built with a custom async loop in `src/agent.rs` for 100% fine-grained control and zero framework bloat.
- **No MCP Subprocesses Required** — All capabilities (document retrieval, file system tools, web search, system commands, cloud REST APIs) run natively inside Rust or via direct HTTP REST endpoints (`reqwest`).
- **Multi-Cloud Resilience Pipeline** — Automatic failover across `Groq` (Qwen 3.6 27B / Llama 3.3 70B), `OpenRouter`, and `Gemini` with exponential backoff retry logic.
- **Local CPU Embeddings** — Powered by `fastembed` (ONNX Runtime, BGE-small-en model) for vectorizing `./docs/` files and queries locally on CPU.
- **Multi-Format Document Support** — Indexes `.txt`, `.md`, `.pdf`, `.csv`, and `.json` files seamlessly.
- **Native Tool Suite** — Includes `search_documents`, `list_documents`, `web_search` (Tavily/Brave), file tools, and Composio REST API integration.
- **Automatic `.env` Setup** — Loads API keys and configurations automatically via `dotenvy`.

---

## Quick Start

### 1. Configure Credentials

Create a `.env` file in the project root:

```env
GROQ_API_KEY=gsk_...
GROQ_MODEL=qwen/qwen3.6-27b

OPENROUTER_API_KEY=sk-or-v1-...
OPENROUTER_MODEL=meta-llama/llama-3.3-70b-instruct

GEMINI_API_KEY=...
GEMINI_MODEL=gemini-2.0-flash
```

*(Get your Groq key at [console.groq.com](https://console.groq.com)).*

### 2. Build & Run

```bash
# Build optimized release binary
cargo build --release

# Run interactive CLI REPL
cargo run --release
```

Or run the single compiled binary directly:

```bash
./target/release/rust-agent-core
```

---

## Project Structure

```text
rust-agent-core/
├── Cargo.toml             # Crate manifest & dependencies
├── .env                   # Local API keys & model overrides
├── .gitignore             # Ignored build outputs and credentials
├── ARCHITECTURE.md        # Standalone architecture & sequence flow
├── README.md              # Project overview & quickstart guide
├── implementation_plan.md    # Master roadmap & implementation plan
├── nano_gguf_roadmap.md      # Blueprint for GGUF CPU Tensor Engine
├── mini_redis_roadmap.md     # Blueprint for Go RESP TCP Server
├── docs/                  # Local document corpus (.txt, .pdf, .csv, .md)
│   ├── groq_ollama.txt
│   ├── rag_basics.txt
│   └── rust_vs_python.txt
└── src/                   # Rust source modules
    ├── main.rs            # Entry point & REPL loop
    ├── agent.rs           # Custom async agent loop & tool dispatcher
    ├── llm.rs             # OpenAI-compatible API client & tool schemas
    ├── embeddings.rs      # FastEmbed ONNX embedding wrapper
    └── store.rs           # Multi-format vector store & cosine search
```

---

## Master Roadmap

- **Phase 1 (Complete)**: Tool-calling stabilization, Groq Qwen 3.6 engine integration, multi-provider failover, and multi-turn conversation memory.
- **Phase 2**: Multi-format ingestion (`.pdf`, `.csv`, `.md`), sliding-window chunking, and `.vector_cache.bin` disk persistence.
- **Phase 3**: Native `web_search` (Tavily/Brave API), file system tools (`list_dir`, `read_file`, `write_file`), and Composio REST integration.
