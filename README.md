# rust-rag-agent

A minimal, ultra-fast Autonomous AI Agent written in Rust.

- **100% Single Binary Architecture** — Self-contained. Requires **no Docker containers**, PostgreSQL, Qdrant, or external database daemons.
- **Custom Agent Loop (No `rig-core`)** — Built with a custom async loop in `src/agent.rs` for 100% fine-grained control and zero framework bloat.
- **No MCP Servers Needed** — All tools (RAG, file system, web search, cloud APIs) run natively inside Rust or via direct REST endpoints (`reqwest`).
- **Qwen 2.5 Recommended Engine** — Optimally tuned for `qwen-2.5-coder-32b` or `qwen-2.5-72b-instruct` on Groq for top-tier function calling precision.
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
    ├── agent.rs        # Custom async agent loop & tool dispatcher
    ├── llm.rs          # OpenAI-compatible API client & tool schemas
    ├── embeddings.rs   # FastEmbed ONNX embedding wrapper
    └── store.rs        # Multi-format vector store & cosine search
```

---

## Master Roadmap

- **Phase 1**: Tool-calling syntax stabilization & Qwen 2.5 engine integration.
- **Phase 2**: Multi-format ingestion (`.pdf`, `.csv`, `.md`), sliding-window chunking, and `.vector_cache.bin` disk persistence.
- **Phase 3**: Native `web_search` (Tavily/Brave API), file system tools (`list_dir`, `read_file`, `write_file`), and Composio REST integration.
