mod agent;
mod embeddings;
mod llm;
mod store;

use agent::{Agent, ExecutionMode};
use embeddings::Embedder;
use llm::LlmClient;
use store::DocStore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let args: Vec<String> = std::env::args().collect();
    let mode = if args.iter().any(|a| a == "--yolo") {
        ExecutionMode::Yolo
    } else if args.iter().any(|a| a == "--read-only") {
        ExecutionMode::ReadOnly
    } else {
        ExecutionMode::Safe
    };

    match mode {
        ExecutionMode::Safe => eprintln!("[safety] Mode: Safe (prompts [y/N] before running shell commands or modifying files)"),
        ExecutionMode::ReadOnly => eprintln!("[safety] Mode: Read-Only (file writes and shell execution are disabled)"),
        ExecutionMode::Yolo => eprintln!("[safety] Mode: Autonomous YOLO (all tools execute without confirmation)"),
    }

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

    let agent = Agent::new(store, embedder, providers, mode);

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

        match query.to_lowercase().as_str() {
            "exit" | "quit" | ":q" => {
                eprintln!("Goodbye!");
                break;
            }
            "clear" => {
                print!("\x1B[2J\x1B[1;1H");
                use std::io::Write;
                std::io::stdout().flush().ok();
                continue;
            }
            "help" => {
                println!("Available commands:");
                println!("  exit, quit, :q  - Exit the REPL session");
                println!("  clear           - Clear terminal screen");
                println!("  help            - Display this help message");
                println!("  <query>         - Ask a question or run an autonomous task");
                continue;
            }
            _ => {}
        }

        match agent.ask_chat(&mut history, query).await {
            Ok(answer) => println!("{answer}"),
            Err(e) => eprintln!("Error: {e}"),
        }
    }


    Ok(())
}
