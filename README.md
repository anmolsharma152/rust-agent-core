# 🦀 rust-agent-core

> **A High-Performance Autonomous AI Agent Engine & Interactive CLI in Pure Rust.**
> 
> *100% self-contained single binary. Zero Docker containers. Zero vector database daemons. Sub-second LPU speed.*

---

[![Rust](https://img.shields.io/badge/Language-Pure%20Rust-orange.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org)
[![Groq LPU](https://img.shields.io/badge/LLM-Groq%20Qwen%203.6%2027B-blueviolet.svg?style=for-the-badge)](https://groq.com)
[![FastEmbed](https://img.shields.io/badge/Embeddings-Local%20ONNX%20CPU-brightgreen.svg?style=for-the-badge)](https://github.com/AnantB/fastembed-rs)
[![License](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](LICENSE)

---

## 💻 What is `rust-agent-core`?

`rust-agent-core` is an **autonomous, tool-calling AI Agent Engine and interactive terminal CLI** built from scratch in pure Rust. 

Unlike traditional Python agent frameworks (LangChain, AutoGen, CrewAI) that require complex virtual environments, heavy dependencies, Docker containers, and external vector database daemons (Qdrant, Milvus, PostgreSQL), `rust-agent-core` compiles into a **single, standalone binary** that runs everywhere with zero runtime setup.

```text
       ┌─────────────────────────────────────────────────────────┐
       │             Interactive Terminal REPL / CLI             │
       └────────────────────────────┬────────────────────────────┘
                                    │
                                    ▼
       ┌─────────────────────────────────────────────────────────┐
       │           Custom Async Agent Loop (src/agent.rs)        │
       └──────┬─────────────────────┬─────────────────────┬──────┘
              │                     │                     │
              ▼                     ▼                     ▼
      ┌───────────────┐     ┌───────────────┐     ┌───────────────┐
      │  Native RAG   │     │ System Tools  │     │ Live Web Search│
      │  Multi-Format │     │ File I/O &    │     │ DuckDuckGo /  │
      │ Vector Cache  │     │ Shell Exec    │     │ Tavily REST   │
      └───────────────┘     └───────────────┘     └───────────────┘
```

---

## ✨ Core Highlights & Key Features

* 🚀 **100% Standalone Single Binary**: Self-contained execution. Requires **no Docker**, **no Python**, and **no external vector database servers**.
* ⚡ **Sub-Second LPU Execution**: Powered by Groq's LPU Engine (`qwen/qwen3.6-27b`) with native token-level function calling, running turns in under 0.4 seconds.
* 📂 **Multi-Format Document Ingestion**: Ingests `.txt`, `.md` (Markdown), `.pdf` (PDF pages), `.csv` (Structured key-value tables), and `.json` files into a local semantic document store.
* 🧠 **0-Second Binary Disk Vector Cache (`.vector_cache.bin`)**: Computes SHA-256 file checksums across `./docs/`. Serializes pre-computed embeddings to disk for **instant 0-millisecond startup** on subsequent runs without re-embedding.
* ✂️ **Sliding-Window Document Chunker**: Automatically splits long documents into 300-word passages with 50-word sliding overlaps for high retrieval precision.
* 🛠️ **Built-in Native Tool Suite**:
  * `search_documents`: Fast local vector similarity search over embedded document passages.
  * `list_documents`: Inspects all ingested filenames in the document store.
  * `list_dir`: Browse local project directories.
  * `read_file` & `write_file`: Read and write files directly on disk.
  * `run_command`: Execute sandboxed bash shell commands on the local machine.
  * `web_search`: Live web search fallback via DuckDuckGo scraping (keyless) or Tavily API.
* 🛡️ **Multi-Cloud Failover & Rate Limit Backoff**: Built-in exponential backoff retry loop for catching `HTTP 429` rate limits, with automatic provider failover: **Groq** $\rightarrow$ **OpenRouter** $\rightarrow$ **Gemini**.

---

## 🖥️ Live Terminal Interactive CLI Demo

```text
$ ./target/release/rust-agent-core
Loading local embedding model...
Indexing documents in ./docs ...
[cache] Loaded 57 cached passages from docs/.vector_cache.bin (0ms)
Ready. Type a question (Ctrl+D to exit).

> list the documents that are embedded
Here are the documents currently indexed in the local store:
- 1.txt
- architecture_guide.md (.md)
- config_settings.json (.json)
- projects.csv (.csv)
- rag_basics.txt (.txt)

> what are the projects listed in projects.csv?
The `projects.csv` file lists the following three projects:
1. **rust-agent-core** — Language: Rust, Target: Autonomous Agent Engine
2. **nano-gguf** — Language: C++, Target: Edge LLM Tensor Engine
3. **mini-redis** — Language: Go, Target: Distributed Cache Server

> what is the latest stable version of Rust programming language as of today?
Executing live web search...
The current stable version of Rust is 1.95.0 (released July 2026).
```

---

## 🛠️ Installation & Building

### 1. Prerequisites
* [Rust toolchain](https://rustup.rs/) (v1.75 or higher)
* A free [Groq API Key](https://console.groq.com)

### 2. Configure Credentials
Create a `.env` file in the project directory (or copy from `.env.example`):

```bash
cp .env.example .env
```

Edit your `.env` with your API keys:

```env
# Primary Provider (Groq LPU - Sub-second inference)
GROQ_API_KEY=gsk_your_groq_key_here
GROQ_MODEL=qwen/qwen3.6-27b

# Secondary Fallback Provider (OpenRouter)
OPENROUTER_API_KEY=sk-or-v1-your_openrouter_key_here
OPENROUTER_MODEL=meta-llama/llama-3.3-70b-instruct

# Tertiary Fallback Provider (Gemini API)
GEMINI_API_KEY=your_gemini_key_here
GEMINI_MODEL=gemini-2.0-flash

# Optional: Web Search API Key (If omitted, keyless DuckDuckGo search is used)
# TAVILY_API_KEY=your_tavily_key_here
```

### 3. Build Release Binary

```bash
cargo build --release
```

The compiled release binary will be available at `./target/release/rust-agent-core`.

---

## ⚡ Usage Modes

### Interactive REPL Mode
Run the agent interactively in your terminal:

```bash
./target/release/rust-agent-core
```

### Non-Interactive / Scripting Pipeline Mode
Pipe queries directly into `rust-agent-core` from shell scripts or Linux pipelines:

```bash
echo "what are the projects listed in projects.csv?" | ./target/release/rust-agent-core
```

---

## 📁 Repository Structure

```text
rust-agent-core/
├── Cargo.toml             # Dependencies (.pdf, .csv, .md, sha2, bincode, fastembed)
├── .env                   # Local API keys & model overrides
├── .env.example           # Example credentials template
├── ARCHITECTURE.md        # Detailed sequence flow & system architecture
├── README.md              # Project overview & quickstart guide
├── implementation_plan.md    # Master technical roadmap
├── docs/                  # Local document corpus (.txt, .md, .pdf, .csv, .json)
│   ├── architecture_guide.md
│   ├── config_settings.json
│   └── projects.csv
└── src/                   # Pure Rust source modules
    ├── main.rs            # Entry point & interactive REPL loop
    ├── agent.rs           # Custom async agent loop & multi-tool dispatcher
    ├── llm.rs             # Multi-provider LLM client & function tool schemas
    ├── embeddings.rs      # FastEmbed local ONNX embedding model wrapper
    └── store.rs           # Multi-format document parser, chunker & binary disk cache
```

---

## 🛣️ Technical Roadmap

- [x] **Phase 1: Stabilization & Groq Qwen 3.6 27B Upgrade**
  - Native token-level function calling, exponential backoff retries, and multi-turn conversation memory.
- [x] **Phase 2: Multi-Format Ingestion & Disk Vector Cache**
  - Added `.pdf`, `.csv`, `.md`, `.json`, `.txt` parsing, 300-word sliding-window chunking, and `.vector_cache.bin` SHA-256 disk caching.
- [x] **Phase 3: System & Web Tools Expansion**
  - Built `list_dir`, `read_file`, `write_file`, sandboxed `run_command`, and keyless/API `web_search`.
- [ ] **Phase 4: Interactive Safety Guardrails**
  - `--safe-mode` (interactive prompt before executing commands), `--read-only`, and `--yolo` execution flags.
- [ ] **Phase 5: Hybrid Retrieval (BM25 + Vector RERANK)**
  - Combine BM25 keyword matching with Cosine Similarity vector search via Reciprocal Rank Fusion (RRF).
- [ ] **Phase 6: Session Persistence & Memory Checkpoints**
  - `--resume <session_id>` flag for restoring conversation checkpoints across terminal restarts.
- [ ] **Phase 7: Async Sub-Agent Worker Swarms**
  - Parallel sub-agent task execution (`spawn_subagent`) using Tokio async channels.
- [ ] **Phase 8: Cross-Platform Packaging & Distribution**
  - Automated binary releases (`cargo-dist`) for Linux, macOS, and Windows.

---

## 📜 License

Distributed under the MIT License. See [LICENSE](LICENSE) for details.
