use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A single chat message. `tool_call_id` is only set on Tool-role messages
/// (the reply to a specific tool call the model made).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String, // "system" | "user" | "assistant" | "tool"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn system(text: impl Into<String>) -> Self {
        Self { role: "system".into(), content: Some(text.into()), tool_calls: None, tool_call_id: None }
    }
    pub fn user(text: impl Into<String>) -> Self {
        Self { role: "user".into(), content: Some(text.into()), tool_calls: None, tool_call_id: None }
    }
    pub fn tool_result(tool_call_id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            role: "tool".into(),
            content: Some(text.into()),
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String, // JSON-encoded string, per the OpenAI tool-calling spec
}

/// Describes a callable tool, in OpenAI's function-calling schema.
#[derive(Debug, Clone, Serialize)]
pub struct ToolDef {
    #[serde(rename = "type")]
    pub kind: &'static str, // always "function"
    pub function: FunctionDef,
}

#[derive(Debug, Clone, Serialize)]
pub struct FunctionDef {
    pub name: String,
    pub description: String,
    pub parameters: Value, // JSON Schema
}

pub fn search_documents_tool() -> ToolDef {
    ToolDef {
        kind: "function",
        function: FunctionDef {
            name: "search_documents".into(),
            description: "Search the local document store for passages relevant to a query. \
                           Use this whenever the user asks something that might be answered by \
                           the indexed documents."
                .into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query, e.g. the user's question or a key phrase from it."
                    }
                },
                "required": ["query"]
            }),

        },
    }
}

#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [Message],
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<&'a [ToolDef]>,
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

/// A minimal OpenAI-compatible chat client. Works against Groq
/// (`https://api.groq.com/openai/v1`) and Ollama's OpenAI-compatible
/// endpoint (`http://localhost:11434/v1`) unchanged.
pub struct LlmClient {
    pub name: &'static str,
    base_url: String,
    api_key: Option<String>,
    model: String,
    http: reqwest::Client,
}

impl LlmClient {
    pub fn groq(model: impl Into<String>) -> Result<Self> {
        let api_key = std::env::var("GROQ_API_KEY")
            .map_err(|_| anyhow!("GROQ_API_KEY is not set"))?;
        Ok(Self {
            name: "groq",
            base_url: "https://api.groq.com/openai/v1".into(),
            api_key: Some(api_key),
            model: model.into(),
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(20))
                .build()?,
        })
    }

    pub fn ollama(model: impl Into<String>) -> Self {
        Self {
            name: "ollama",
            base_url: std::env::var("OLLAMA_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:11434/v1".into()),
            api_key: None,
            model: model.into(),
            // Ollama runs on-machine, but local inference can still be slow
            // on CPU-only hardware, so give it much more headroom than Groq.
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build().unwrap(),
        }
    }

    pub async fn chat(&self, messages: &[Message], tools: Option<&[ToolDef]>) -> Result<Message> {
        let url = format!("{}/chat/completions", self.base_url);
        let body = ChatRequest { model: &self.model, messages, tools, temperature: 0.0 };


        let mut req = self.http.post(&url).json(&body);
        if let Some(key) = &self.api_key {
            req = req.bearer_auth(key);
        }

        let resp = req.send().await.map_err(|e| anyhow!("[{}] request failed: {e}", self.name))?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("[{}] HTTP {status}: {text}", self.name));
        }

        let parsed: ChatResponse = resp
            .json()
            .await
            .map_err(|e| anyhow!("[{}] failed to parse response: {e}", self.name))?;

        parsed
            .choices
            .into_iter()
            .next()
            .map(|c| c.message)
            .ok_or_else(|| anyhow!("[{}] empty choices in response", self.name))
    }
}
