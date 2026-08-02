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
    pub fn assistant(text: impl Into<String>) -> Self {
        Self { role: "assistant".into(), content: Some(text.into()), tool_calls: None, tool_call_id: None }
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
                        "description": "Short search keywords (max 100 characters).",
                        "maxLength": 100
                    }
                },
                "required": ["query"]
            }),
        },
    }
}

pub fn list_documents_tool() -> ToolDef {
    ToolDef {
        kind: "function",
        function: FunctionDef {
            name: "list_documents".into(),
            description: "List all filenames currently indexed in the local document store. \
                           Use this whenever the user asks what documents are embedded or available."
                .into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
    }
}

pub fn web_search_tool() -> ToolDef {
    ToolDef {
        kind: "function",
        function: FunctionDef {
            name: "web_search".into(),
            description: "Search the live web for recent events, news, or real-time information."
                .into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Short web search keywords (max 100 characters).",
                        "maxLength": 100
                    }
                },
                "required": ["query"]
            }),
        },
    }
}

pub fn list_dir_tool() -> ToolDef {
    ToolDef {
        kind: "function",
        function: FunctionDef {
            name: "list_dir".into(),
            description: "List files and subdirectories in a local directory.".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Relative or absolute directory path (defaults to '.' if omitted)."
                    }
                },
                "required": []
            }),
        },
    }
}

pub fn read_file_tool() -> ToolDef {
    ToolDef {
        kind: "function",
        function: FunctionDef {
            name: "read_file".into(),
            description: "Read text content of a file from disk.".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path to read."
                    }
                },
                "required": ["path"]
            }),
        },
    }
}

pub fn write_file_tool() -> ToolDef {
    ToolDef {
        kind: "function",
        function: FunctionDef {
            name: "write_file".into(),
            description: "Create or overwrite a file on disk with specified content.".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Target file path."
                    },
                    "content": {
                        "type": "string",
                        "description": "Text content to write."
                    }
                },
                "required": ["path", "content"]
            }),
        },
    }
}

pub fn run_command_tool() -> ToolDef {
    ToolDef {
        kind: "function",
        function: FunctionDef {
            name: "run_command".into(),
            description: "Execute a bash command line on the local system.".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The exact shell command line string to execute."
                    }
                },
                "required": ["command"]
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

    pub fn openrouter(model: impl Into<String>) -> Result<Self> {
        let api_key = std::env::var("OPENROUTER_API_KEY")
            .map_err(|_| anyhow!("OPENROUTER_API_KEY is not set"))?;
        Ok(Self {
            name: "openrouter",
            base_url: "https://openrouter.ai/api/v1".into(),
            api_key: Some(api_key),
            model: model.into(),
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()?,
        })
    }

    pub fn gemini(model: impl Into<String>) -> Result<Self> {
        let api_key = std::env::var("GEMINI_API_KEY")
            .map_err(|_| anyhow!("GEMINI_API_KEY is not set"))?;
        Ok(Self {
            name: "gemini",
            base_url: "https://generativelanguage.googleapis.com/v1beta/openai".into(),
            api_key: Some(api_key),
            model: model.into(),
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()?,
        })
    }


    #[allow(dead_code)]
    pub fn ollama(model: impl Into<String>) -> Self {
        Self {
            name: "ollama",
            base_url: std::env::var("OLLAMA_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:11434/v1".into()),
            api_key: None,
            model: model.into(),
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build().unwrap(),
        }
    }

    pub async fn chat(&self, messages: &[Message], tools: Option<&[ToolDef]>) -> Result<Message> {
        let url = format!("{}/chat/completions", self.base_url);
        let body = ChatRequest { model: &self.model, messages, tools, temperature: 0.0 };

        let max_retries = 3;
        let mut last_error = anyhow!("Unknown API error");

        for attempt in 0..=max_retries {
            let mut req = self.http.post(&url).json(&body);
            if let Some(key) = &self.api_key {
                req = req.bearer_auth(key);
            }

            let resp = match req.send().await {
                Ok(r) => r,
                Err(e) => {
                    last_error = anyhow!("[{}] request failed: {e}", self.name);
                    if attempt < max_retries {
                        let backoff_secs = 1 << attempt;
                        eprintln!("[{}] Network error ({e}) — retrying in {backoff_secs}s...", self.name);
                        tokio::time::sleep(std::time::Duration::from_secs(backoff_secs)).await;
                        continue;
                    }
                    return Err(last_error);
                }
            };

            let status = resp.status();

            if status.is_success() {
                let parsed: ChatResponse = resp
                    .json()
                    .await
                    .map_err(|e| anyhow!("[{}] failed to parse response: {e}", self.name))?;

                return parsed
                    .choices
                    .into_iter()
                    .next()
                    .map(|c| c.message)
                    .ok_or_else(|| anyhow!("[{}] empty choices in response", self.name));
            }

            // Handle Rate Limiting (429) or Service Unavailable (503/502/504) with Backoff
            if (status.as_u16() == 429 || status.is_server_error()) && attempt < max_retries {
                let retry_after_header = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok());

                let wait_secs = retry_after_header.unwrap_or(1 << attempt);
                let text = resp.text().await.unwrap_or_default();
                eprintln!(
                    "[{}] HTTP {status} (rate limited/server busy) — retrying in {wait_secs}s... (details: {text})",
                    self.name
                );
                tokio::time::sleep(std::time::Duration::from_secs(wait_secs)).await;
                continue;
            }

            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("[{}] HTTP {status}: {text}", self.name));
        }

        Err(last_error)
    }
}

