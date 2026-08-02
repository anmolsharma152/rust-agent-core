mod agent;
mod embeddings;
mod llm;
mod store;

use agent::Agent;
use embeddings::Embedder;
use llm::LlmClient;
use store::DocStore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    // 1. Load the local embedding model (downloads ~130MB from Hugging Face
    //    on first run, then caches it — no network needed after that).
    eprintln!("Loading local embedding model...");
    let embedder = Embedder::new()?;

    // 2. Embed every .txt file under ./docs into an in-memory store.
    eprintln!("Indexing documents in ./docs ...");
    let store = DocStore::load_dir("docs", &embedder)?;

    // 3. Set up multi-provider failover pipeline:
    //    Primary: Groq (Qwen 3.6 27B - Flawless Tool Calling + Sub-second speed)
    //    Secondary: OpenRouter (Multi-model cloud failover)
    //    Tertiary: Gemini (Large-scale context)
    let groq_model = std::env::var("GROQ_MODEL").unwrap_or_else(|_| "qwen/qwen3.6-27b".into());
    let openrouter_model = std::env::var("OPENROUTER_MODEL").unwrap_or_else(|_| "meta-llama/llama-3.3-70b-instruct".into());
    let gemini_model = std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-2.0-flash".into());

    let mut providers = Vec::new();
    if let Ok(client) = LlmClient::groq(groq_model) {
        providers.push(client);
    }
    if let Ok(client) = LlmClient::openrouter(openrouter_model) {
        providers.push(client);
    }
    if let Ok(client) = LlmClient::gemini(gemini_model) {
        providers.push(client);
    }

    if providers.is_empty() {
        return Err(anyhow::anyhow!("No API keys configured. Please set GROQ_API_KEY, OPENROUTER_API_KEY, or GEMINI_API_KEY in .env"));
    }

    let agent = Agent::new(store, embedder, providers);

    // 4. Simple REPL over stdin with multi-turn conversation memory.
    eprintln!("Ready. Type a question (Ctrl+D to exit).");
    let stdin = std::io::stdin();
    let mut line = String::new();
    let mut history = Vec::new();

    loop {
        line.clear();
        eprint!("\n> ");
        use std::io::Write;
        std::io::stderr().flush().ok();
        if stdin.read_line(&mut line)? == 0 {
            break; // EOF
        }
        let query = line.trim();
        if query.is_empty() {
            continue;
        }

        match agent.ask_chat(&mut history, query).await {
            Ok(answer) => println!("{answer}"),
            Err(e) => eprintln!("Error: {e}"),
        }
    }


    Ok(())
}
