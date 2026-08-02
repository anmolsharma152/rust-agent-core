# Pure Rust Agent Core Architecture

This guide describes the pure Rust autonomous AI agent engine.

## Core Features
1. **Zero External Vector DB**: Embedded ONNX model embeddings calculated locally.
2. **Multi-Format Ingestion**: Supports `.txt`, `.md`, `.pdf`, `.csv`, and `.json`.
3. **Binary Vector Disk Cache**: Serializes pre-computed embeddings into `.vector_cache.bin` using SHA-256 hashes for instant startup.
4. **Resilient LPU Engine**: Executes tool calls on Groq (`qwen/qwen3.6-27b`) with sub-second LPU latencies.
