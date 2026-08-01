use anyhow::{anyhow, Result};

use crate::embeddings::Embedder;
use crate::llm::{search_documents_tool, LlmClient, Message};
use crate::store::DocStore;

const MAX_TURNS: usize = 5;
const SYSTEM_PROMPT: &str = "You are a helpful assistant with access to local documents via the search_documents tool. \
Use search whenever the user asks something answerable by local documents, then answer based on the retrieved context.";


pub struct Agent {
    store: DocStore,
    embedder: Embedder,
    /// Tried in order for each new query: primary first (Groq), then fallback (Ollama).
    providers: Vec<LlmClient>,
}

impl Agent {
    pub fn new(store: DocStore, embedder: Embedder, providers: Vec<LlmClient>) -> Self {
        Self { store, embedder, providers }
    }

    /// Runs one user query through the agent loop. Tries each provider in
    /// order, falling back to the next one only if the *first* call to a
    /// provider fails (network error, auth error, rate limit, etc). This
    /// keeps a single query's tool-calling turns on one model family rather
    /// than mixing providers mid-conversation.
    pub async fn ask(&self, user_query: &str) -> Result<String> {
        let messages = vec![Message::system(SYSTEM_PROMPT), Message::user(user_query)];

        let mut last_err = None;
        for (i, provider) in self.providers.iter().enumerate() {
            match self.run_loop(provider, messages.clone()).await {
                Ok(answer) => return Ok(answer),
                Err(e) => {
                    let is_last = i == self.providers.len() - 1;
                    eprintln!(
                        "[agent] {} failed on first call ({e}){}",
                        provider.name,
                        if is_last { "" } else { " — falling back to next provider" }
                    );
                    last_err = Some(e);
                }
            }
        }
        Err(last_err.unwrap_or_else(|| anyhow!("no providers configured")))
    }

    async fn run_loop(&self, provider: &LlmClient, mut messages: Vec<Message>) -> Result<String> {
        let tools = [search_documents_tool()];

        for _turn in 0..MAX_TURNS {
            let reply = provider.chat(&messages, Some(&tools)).await?;

            let Some(tool_calls) = reply.tool_calls.clone() else {
                // No tool call — this is the final answer.
                return Ok(reply.content.unwrap_or_default());
            };

            // The model wants to call one or more tools. Record its turn, then
            // append one tool-result message per call before looping back.
            messages.push(reply.clone());

            for call in tool_calls {
                if call.function.name != "search_documents" {
                    messages.push(Message::tool_result(
                        call.id,
                        format!("error: unknown tool '{}'", call.function.name),
                    ));
                    continue;
                }

                let args: serde_json::Value = serde_json::from_str(&call.function.arguments)
                    .unwrap_or(serde_json::json!({}));
                let query = args.get("query").and_then(|v| v.as_str()).unwrap_or(user_query_fallback(&messages));
                let top_k = args
                    .get("top_k")
                    .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
                    .unwrap_or(3) as usize;


                let result_text = match self.retrieve(query, top_k) {
                    Ok(text) => text,
                    Err(e) => format!("error running search: {e}"),
                };
                messages.push(Message::tool_result(call.id, result_text));
            }
        }

        Err(anyhow!("hit MAX_TURNS ({MAX_TURNS}) without a final answer"))
    }

    /// Blocking embedding + similarity search. For a single-user CLI this is
    /// fine to run inline; a server handling concurrent requests should run
    /// this via `tokio::task::spawn_blocking` instead so it doesn't stall
    /// other requests on the same worker thread.
    fn retrieve(&self, query: &str, top_k: usize) -> Result<String> {
        let query_embedding = self.embedder.embed_query(query)?;
        let hits = self.store.search(&query_embedding, top_k);

        if hits.is_empty() {
            return Ok("No documents found.".to_string());
        }

        let formatted = hits
            .iter()
            .map(|(doc, score)| format!("[{} | score={:.3}]\n{}", doc.source, score, doc.text))
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");
        Ok(formatted)
    }
}

/// Best-effort fallback if the model calls the tool without a `query` field —
/// just reuse the original user message so retrieval still does something sane.
fn user_query_fallback(messages: &[Message]) -> &str {
    messages
        .iter()
        .find(|m| m.role == "user")
        .and_then(|m| m.content.as_deref())
        .unwrap_or("")
}
