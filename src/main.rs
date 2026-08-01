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

    // 3. Set up providers: Groq first, local Ollama as fallback.
    //    GROQ_MODEL / OLLAMA_MODEL let you override the defaults below.
    let groq_model = std::env::var("GROQ_MODEL").unwrap_or_else(|_| "llama-3.3-70b-versatile".into());
    let _ollama_model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2".into());




    let mut providers = Vec::new();
    match LlmClient::groq(groq_model) {
        Ok(client) => providers.push(client),
        Err(e) => eprintln!("Groq API key not configured ({e})."),
    }
    // Ollama fallback disabled per user configuration:
    // providers.push(LlmClient::ollama(ollama_model));


    let agent = Agent::new(store, embedder, providers);

    // 4. Simple REPL over stdin.
    eprintln!("Ready. Type a question (Ctrl+D to exit).");
    let stdin = std::io::stdin();
    let mut line = String::new();
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

        match agent.ask(query).await {
            Ok(answer) => println!("{answer}"),
            Err(e) => eprintln!("Error: {e}"),
        }
    }

    Ok(())
}
