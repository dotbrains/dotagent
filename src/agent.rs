use crate::tools::{execute_tool_call, to_openai_tools, ToolSpec};
use anyhow::{anyhow, bail, Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashSet;
use std::io::{self, Write};

const SYSTEM_PROMPT: &str = "You are a helpful coding agent with access to tools for reading, listing, and editing files in the user's working directory. Use the tools whenever they would let you answer more accurately than guessing. Prefer reading a file over asking the user to paste its contents. When editing, make the smallest change that satisfies the request. Keep replies short.";

#[derive(Debug, Deserialize)]
struct ChatCompletionsResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: AssistantMessage,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct AssistantMessage {
    role: String,
    #[serde(default)]
    content: Option<Value>,
    #[serde(default)]
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ToolCall {
    id: String,
    #[serde(rename = "type")]
    kind: String,
    function: ToolFunctionCall,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ToolFunctionCall {
    name: String,
    arguments: String,
}

pub fn run_agent(tools: Vec<ToolSpec>) -> Result<()> {
    let _ = ctrlc::set_handler(|| {
        println!("Goodbye!\n");
        std::process::exit(0);
    });

    let api_key = std::env::var("OPENAI_API_KEY").context("OPENAI_API_KEY is not set")?;
    let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-5".to_string());
    let openai_tools = to_openai_tools(&tools);
    let known_tool_names: HashSet<&str> = tools.iter().map(|tool| tool.name).collect();
    let client = Client::builder()
        .build()
        .context("failed to build HTTP client")?;

    let mut messages: Vec<Value> = vec![json!({
        "role": "system",
        "content": SYSTEM_PROMPT
    })];

    println!("Chat with {model} (type 'exit' or 'quit' or use Ctrl-C to quit)\n");

    loop {
        print!("You: ");
        io::stdout().flush().context("failed to flush stdout")?;

        let mut user_input = String::new();
        let bytes_read = io::stdin()
            .read_line(&mut user_input)
            .context("failed to read user input")?;

        if bytes_read == 0 {
            println!("Goodbye!\n");
            return Ok(());
        }

        let trimmed = user_input.trim();
        if matches!(
            trimmed.to_ascii_lowercase().as_str(),
            "exit" | "quit" | ":q"
        ) {
            println!("Goodbye!\n");
            return Ok(());
        }

        if trimmed.is_empty() {
            continue;
        }

        messages.push(json!({
            "role": "user",
            "content": trimmed
        }));
        println!();

        loop {
            let request_body = json!({
                "model": model,
                "messages": messages,
                "tools": openai_tools
            });

            let response = client
                .post("https://api.openai.com/v1/chat/completions")
                .bearer_auth(&api_key)
                .json(&request_body)
                .send()
                .context("failed to call OpenAI API")?;

            let status = response.status();
            if !status.is_success() {
                let body = response.text().unwrap_or_default();
                bail!("OpenAI API error ({status}): {body}");
            }

            let parsed: ChatCompletionsResponse = response
                .json()
                .context("failed to parse OpenAI chat completion response")?;

            let assistant_message = parsed
                .choices
                .into_iter()
                .next()
                .ok_or_else(|| anyhow!("OpenAI API returned no choices"))?
                .message;

            messages.push(assistant_message_to_history_value(&assistant_message));

            let tool_calls = assistant_message.tool_calls.unwrap_or_default();
            if tool_calls.is_empty() {
                let text = extract_text_content(assistant_message.content.as_ref());
                println!("Agent: {text}\n");
                break;
            }

            for call in tool_calls {
                if call.kind != "function" {
                    continue;
                }

                println!(
                    "Tool: {}({})\n",
                    call.function.name, call.function.arguments
                );

                let result = if !known_tool_names.contains(call.function.name.as_str()) {
                    format!("ERROR: Unknown tool: {}", call.function.name)
                } else {
                    match execute_tool_call(&call.function.name, &call.function.arguments) {
                        Ok(result) => result,
                        Err(err) => {
                            let err_msg = format!("ERROR: {err}");
                            eprintln!("{err_msg}\n");
                            err_msg
                        }
                    }
                };

                messages.push(json!({
                    "role": "tool",
                    "tool_call_id": call.id,
                    "content": result
                }));
            }
        }
    }
}

fn assistant_message_to_history_value(message: &AssistantMessage) -> Value {
    let mut payload = Map::new();
    payload.insert("role".to_string(), Value::String("assistant".to_string()));
    payload.insert(
        "content".to_string(),
        message.content.clone().unwrap_or(Value::Null),
    );

    if let Some(tool_calls) = &message.tool_calls {
        if !tool_calls.is_empty() {
            payload.insert(
                "tool_calls".to_string(),
                serde_json::to_value(tool_calls).unwrap_or(Value::Null),
            );
        }
    }

    Value::Object(payload)
}

fn extract_text_content(content: Option<&Value>) -> String {
    match content {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(text)) => text.clone(),
        Some(Value::Array(parts)) => {
            let text = parts
                .iter()
                .filter_map(|part| part.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join("");

            if text.is_empty() {
                Value::Array(parts.clone()).to_string()
            } else {
                text
            }
        }
        Some(other) => other.to_string(),
    }
}
