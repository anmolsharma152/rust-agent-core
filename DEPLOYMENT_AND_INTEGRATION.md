# Deployment & Integration Architecture Blueprint

This document details the strategies and architecture for deploying `rust-agent-core` to the cloud and integrating it with external web applications, browser clients, and the **V5 CodexEngine** workspace platform.

---

## 🌐 1. WebAssembly (WASM) In-Browser Execution

### Overview
Compile `rust-agent-core` into WebAssembly (`wasm32-unknown-unknown` target via `wasm-pack`). The local ONNX embedding runtime (`fastembed-rs` or `ort` WASM backend), document chunker, and memory engine compile directly into a `.wasm` binary that runs inside the user's web browser or WebWorker.

### Architecture
```text
┌────────────────────────────────────────────────────────────────────────┐
│                        Web Browser / Web Application                   │
│                                                                        │
│   ┌────────────────────────────────────────────────────────────────┐   │
│   │               WASM Module (rust_agent_core.wasm)               │   │
│   │                                                                │   │
│   │   • Local ONNX Embeddings (WASM CPU / WebGPU)                  │   │
│   │   • Multi-Format Document Chunker & Memory Engine              │   │
│   │   • Client-side Tool Dispatcher                                │   │
│   └───────────────────────────────┬────────────────────────────────┘   │
│                                   │                                    │
└───────────────────────────────────┼────────────────────────────────────┘
                                    │ HTTPS REST
                                    ▼
                         Groq / OpenRouter / Gemini
```

### Key Advantages
- **Zero Server Hosting Costs**: Document parsing, vector embeddings, and tool state evaluation run 100% on the client CPU.
- **Client Privacy**: Local files are indexed locally inside browser storage (`IndexedDB`) without leaving the user's machine.
- **Web App Integration**: Can be imported as an NPM module (`@anmol/rust-agent-core-wasm`) inside React, Next.js, or Vue applications.

---

## ☁️ 2. Cloud HTTP REST & WebSocket Microservice (`Axum`)

### Overview
Wrap `rust-agent-core` with `axum` (Rust's high-performance async web framework) to turn the engine into a scalable, cloud-native HTTP API and WebSocket microservice.

### Proposed Endpoints
- **`POST /v1/chat/completions`**: OpenAI-compatible endpoint for drop-in LLM chat and tool execution.
- **`POST /v1/rag/search`**: High-speed vector similarity search over uploaded document corpora.
- **`WS /v1/agent/stream`**: Real-time WebSocket connection for streaming assistant tokens and live tool execution logs.

### Deployment Targets
- **Docker Container**: Compiles to a 15MB static Linux binary container consuming under 30MB of RAM.
- **Cloud Hosts**: Deployable with 1-click on **Fly.io**, **Render**, **Koyeb**, **AWS ECS**, or **DigitalOcean App Platform**.

---

## 💻 3. V5 CodexEngine Workspace Integration

### Overview
Integrate `rust-agent-core` into the persistent V5 CodexEngine workspace as an autonomous AI pair-programmer and file intelligence agent.

### Integration Patterns

#### Pattern A: Workspace CLI Sidecar (Backend IPC)
- V5 CodexEngine spawns `rust-agent-core` in background JSON mode (`rust-agent-core --json`).
- Communication happens over standard IPC pipes (`stdin`/`stdout`).
- The agent can inspect workspace project files, run local build/test commands (`run_command`), and apply code edits (`write_file`).

#### Pattern B: In-Browser WebWorker Engine (Frontend)
- The compiled `.wasm` engine is loaded into the V5 CodexEngine frontend WebWorker.
- Performs client-side semantic code search over open project tabs without requiring a dedicated backend server.

---

## 🛣️ Future Implementation Checklist

- [ ] Add `wasm-bindgen` annotations to `DocStore`, `Embedder`, and `Agent` structs.
- [ ] Create `wasm-pack` build script in `Cargo.toml`.
- [ ] Build optional `src/server.rs` module with `axum` for cloud HTTP/WebSocket deployment.
- [ ] Build `Dockerfile` for single-binary containerization.
- [ ] Create JSON RPC schema for V5 CodexEngine sidecar integration.
