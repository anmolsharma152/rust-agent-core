# rust-rag-agent

A minimal, ultra-fast Agentic RAG CLI written in Rust.

- **100% Single Binary Architecture** — Self-contained. Requires **no Docker containers**, PostgreSQL, Qdrant, or external database daemons.
- **Qwen 2.5 Recommended Engine** — Optimally tuned for `qwen-2.5-coder-32b` or `qwen-2.5-72b-instruct` on Groq for top-tier function calling precision.
- **Local CPU Embeddings** — Powered by `fastembed` (ONNX Runtime, BGE-small-en model) for vectorizing `./docs/` files and queries locally on CPU.
- **Multi-Format Document Support** — Indexes `.txt`, `.md`, `.pdf`, `.csv`, and `.json` files seamlessly.
- **Dedicated Tool Suite** — Includes `search_documents`, `list_documents`, and optional `web_search`.
- **Automatic `.env` Setup** — Loads API keys and configurations automatically via `dotenvy`.
- **Architecture Documentation** — See [ARCHITECTURE.md](file:///home/anmol/Projects/rust-rag-agent/ARCHITECTURE.md) for full sequence diagrams and module designs.

---

## Quick Start

### 1. Configure Credentials

Create a `.env` file in the project root:

```env
GROQ_API_KEY=gsk_...
GROQ_MODEL=qwen-2.5-coder-32b
```

*(Get your key at [console.groq.com](https://console.groq.com)).*

### 2. Build & Run

```bash
# Build optimized release binary
cargo build --release

# Run interactive CLI REPL
cargo run --release
```

Or run the single compiled binary directly:

```bash
./target/release/rust-rag-agent
```

---

## How It Works

1. On startup, `fastembed` loads the local ONNX BGE-small-en embedding model.
2. All `.txt`, `.md`, `.pdf`, `.csv`, and `.json` files in `./docs/` are parsed, chunked, and embedded into an in-memory vector store (`DocStore`).
3. You type questions at the `> ` terminal prompt.
4. Qwen 2.5 dynamically decides whether to answer general facts directly or call `search_documents` / `list_documents`.

---

## Project Structure

```text
rust-rag-agent/
├── Cargo.toml          # Crate manifest & dependencies
├── .env                # Local API keys & model overrides
├── .gitignore          # Ignored build outputs and credentials
├── ARCHITECTURE.md     # Single-binary architecture & diagrams
├── README.md           # Quickstart guide
├── implementation_plan.md # Master roadmap & implementation plan
├── docs/               # Local document corpus (.txt, .pdf, .csv, .md)
│   ├── groq_ollama.txt
│   ├── rag_basics.txt
│   └── rust_vs_python.txt
└── src/                # Rust source modules
    ├── main.rs         # Entry point & REPL loop
    ├── agent.rs        # Agentic tool execution loop
    ├── llm.rs          # OpenAI-compatible API client
    ├── embeddings.rs   # FastEmbed ONNX embedding wrapper
    └── store.rs        # Multi-format vector store & cosine search
```

---

## Master Roadmap

- **Phase 1**: Tool-calling syntax stabilization & Qwen 2.5 engine integration.
- **Phase 2**: Multi-format ingestion (`.pdf`, `.csv`, `.md`), sliding-window chunking, and `.vector_cache.bin` disk persistence.
- **Phase 3**: Web search tool (`web_search`) and expanded system tools.
