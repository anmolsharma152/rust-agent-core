# nano-gguf: Minimal Quantized Tensor Inference Engine in Rust

A lightweight, high-performance C/Rust-level tensor execution runtime built from scratch to run quantized GGUF models (Llama/Qwen) on CPU without heavy third-party framework dependencies.

---

## 🎯 Architectural Philosophy

- **Zero Heavy Frameworks**: No `torch`, `onnxruntime`, or C++ `llama.cpp` wrappers. Pure Rust memory management and SIMD intrinsics.
- **Memory-Mapped Weight Loading (`mmap`)**: Zero-copy loading of GGUF model files directly into host memory pages.
- **Quantized Matrix Multiplication (INT8/INT4)**: Custom SIMD dot-product kernels (AVX2/NEON) for quantized tensor execution in the hot loop.
- **KV-Cache & Autoregressive Decoding**: Low-allocation key-value attention cache for continuous token generation.

---

## 🚀 Milestones & Implementation Stages

### Stage 1: GGUF File Format Parser
- Implement GGUF binary format parser according to official GGUF v3 specification.
- Parse header metadata, string key-value pairs, tensor descriptors (name, shape, data type, offset).
- Memory-map the model file using `memmap2` crate for zero-copy slice access.

### Stage 2: Quantization Kernels & SIMD Math
- Implement dequantization and quantized dot-product kernels:
  - `Q4_0` (4-bit quantization with block scale factors).
  - `Q8_0` (8-bit quantization for high-precision activation vectors).
- Accelerate matmul dot products using CPU SIMD intrinsics (`std::arch::x86_64` or `std::arch::aarch64`).

### Stage 3: Tensor Operations & Layer Primitives
- Implement fundamental tensor operations:
  - `RMSNorm` (Root Mean Square Normalization).
  - `Softmax` (Numerically stable softmax).
  - `RoPE` (Rotary Positional Embeddings).
  - `SiLU` / `SwiGLU` activation functions.

### Stage 4: Transformer Architecture Assembly
- Assemble the Transformer layer block:
  - Multi-Head Attention (MHA) / Grouped-Query Attention (GQA).
  - Feed-Forward Network (FFN) with SwiGLU.
  - Residual connections and layer normalization.

### Stage 5: Autoregressive KV-Cache & Generation Loop
- Implement stateful Key-Value (KV) Cache for past token positions to eliminate redundant attention computations.
- Implement token sampling strategies: Top-K, Top-P (nucleus), Temperature scaling, Greedy sampling.

### Stage 6: Byte-Pair Encoding (BPE) Tokenizer & CLI REPL
- Parse GGUF vocabulary metadata (BPE tokens, merge rules, special tokens like `<|im_start|>`, `<|endoftext|>`).
- Implement BPE tokenizer and decoder.
- Build interactive CLI terminal runner with streaming token output (`tok/s` benchmark reporter).

---

## 🧩 Core Data Structures (Rust Spec)

```rust
pub struct GgufHeader {
    pub magic: u32,
    pub version: u32,
    pub tensor_count: u64,
    pub metadata_kv_count: u64,
}

pub struct TensorInfo {
    pub name: String,
    pub shape: Vec<u64>,
    pub qtype: QuantType, // Q4_0, Q8_0, F32
    pub offset: u64,
}

pub struct KvCache {
    pub k: Vec<f32>, // [num_layers, max_seq_len, num_heads, head_dim]
    pub v: Vec<f32>,
}
```

---

## 🔗 Integration with `rust-rag-agent`

Once `nano-gguf` is built, it can be imported as a local library in `rust-rag-agent` to replace cloud API calls (`LlmClient`) with a 100% offline, self-contained local model runner on dual-core CPUs.
