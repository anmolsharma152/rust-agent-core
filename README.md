# rust-rag-agent

A minimal, ultra-fast Agentic RAG CLI written in Rust.

- **100% Single Binary Architecture** — Completely self-contained. Requires **no Docker containers**, PostgreSQL, Qdrant, or external database daemons.
- **Local CPU Embeddings** — Powered by `fastembed` (ONNX Runtime, BGE-small-en model) for vectorizing `./docs/` files and queries locally on CPU.
- **Multi-Format Document Support** — Indexes `.txt`, `.md`, `.pdf`, `.csv`, and `.json` files seamlessly.
- **Sliding-Window Text Chunking** — Automatically chunks long documents into 300–400 word passages to maximize vector search accuracy.
- **Agentic Function Calling** — Driven by Groq LPUs (`llama-3.3-70b-versatile`) via OpenAI-compatible tool definitions.
- **Automatic `.env` Setup** — Loads API keys and configurations automatically via `dotenvy`.
- **Architecture Documentation** — See [ARCHITECTURE.md](file:///home/anmol/Projects/rust-rag-agent/ARCHITECTURE.md) for full sequence diagrams and module designs.

---

## Quick Start

### 1. Configure Credentials

Create a `.env` file in the project root:

```env
GROQ_API_KEY=gsk_...
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

1. On startup, `fastembed` downloads/loads the BGE-small-en model (~130MB ONNX model cached locally).
2. All `.txt`, `.md`, `.pdf`, `.csv`, and `.json` files in `./docs/` are parsed, chunked, and embedded into an in-memory vector store (`DocStore`).
3. You type questions at the `> ` terminal prompt.
4. Groq dynamically decides when to call `search_documents`, retrieves relevant document passages via cosine similarity, and synthesizes the answer.

---

## Project Structure

```text
rust-rag-agent/
├── Cargo.toml          # Crate manifest & dependencies
├── .env                # Local API keys & configuration
├── .gitignore          # Ignored build outputs and credentials
├── ARCHITECTURE.md     # Single-binary architecture & diagrams
├── README.md           # Quickstart guide
├── implementation_plan.md # Roadmap & implementation plan
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

## Technical Specs & Roadmap

- **Vector Engine**: Embedded Rust vector store with `.vector_cache.bin` disk persistence.
- **Document Chunking**: 300–400 words per chunk with 50-word sliding overlap.
- **Web Search Integration**: Optional Tavily / Brave Search API integration.
