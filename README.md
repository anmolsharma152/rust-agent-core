# rust-rag-agent

A minimal, high-performance Agentic RAG CLI written in Rust.

- **Local CPU Embeddings** — Powered by `fastembed` (ONNX Runtime, BGE-small-en model) for vectorizing `./docs/*.txt` and incoming queries locally. No external embedding API required.
- **Agentic Function Calling** — Groq LPUs (`llama-3.3-70b-versatile`) dynamically decide when to call the `search_documents` tool based on prompt context.
- **Automatic `.env` Setup** — Loads API credentials and model configurations automatically at startup using `dotenvy`.
- **Architecture Documentation** — See [ARCHITECTURE.md](file:///home/anmol/Projects/rust-rag-agent/ARCHITECTURE.md) for full module specifications and sequence diagrams.

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

Or run the compiled binary directly:

```bash
./target/release/rust-rag-agent
```

---

## How It Works

1. On startup, `fastembed` downloads the BGE-small-en model (~130MB ONNX model from Hugging Face on first run, cached locally afterwards).
2. All `.txt` files in `./docs/` are embedded into an in-memory vector store (`DocStore`).
3. You type questions at the `> ` terminal prompt.
4. If a question can be answered by local documents, Groq automatically invokes `search_documents`, retrieves matching passages via cosine similarity, and synthesizes the answer.

---

## Project Structure

```text
rust-rag-agent/
├── Cargo.toml          # Crate manifest & dependencies
├── .env                # Local API keys & configuration
├── .gitignore          # Ignored build outputs and credentials
├── ARCHITECTURE.md     # System design & sequence diagrams
├── README.md           # Quickstart guide
├── docs/               # Local document corpus (.txt)
│   ├── groq_ollama.txt
│   ├── rag_basics.txt
│   └── rust_vs_python.txt
└── src/                # Rust source modules
    ├── main.rs         # Entry point & REPL loop
    ├── agent.rs        # Agentic tool execution loop
    ├── llm.rs          # OpenAI-compatible API client
    ├── embeddings.rs   # FastEmbed ONNX embedding wrapper
    └── store.rs        # In-memory vector store & cosine search
```

---

## Model & Environment Overrides

You can customize models in `.env` or via shell environment variables:

```bash
export GROQ_MODEL=llama-3.3-70b-versatile
export OLLAMA_MODEL=llama3.2
```
