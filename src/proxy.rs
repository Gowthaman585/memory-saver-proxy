// ============================================================================
// MODULE: Transparent Proxy Pipe (proxy.rs)
// Intercepts incoming HTTP requests, logs conversation history to SQLite,
// applies prompt truncation/token budgeting, forwards requests to Ollama,
// and captures assistant responses with latency tracking.
// ============================================================================
use axum::{
    body::Body,
    extract::Request,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

// Structure to match incoming AI chat payloads (Ollama / OpenAI format)
#[derive(Deserialize, Serialize, Debug)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct ChatRequest {
    messages: Vec<ChatMessage>,
    // 🛠 FIX: keep every other field (model, stream, options...) so that
    // re-serializing the request for Ollama does not drop them.
    #[serde(flatten)]
    extra: serde_json::Map<String, serde_json::Value>,
}

// Struct to capture the assistant's response from Ollama
#[derive(Deserialize, Debug)]
struct ChatResponse {
    message: ChatMessage,
}

/// Helper to estimate total tokens in a list of messages based on character length (~4 chars per token)
fn estimate_tokens(messages: &[ChatMessage]) -> usize {
    let total_chars: usize = messages.iter().map(|m| m.content.len()).sum();
    total_chars / 4
}

/// 🛠 FIX: Ollama streams by default (one JSON object per line), so parsing the whole
/// body as a single JSON fails. This reads line by line and joins the content chunks.
/// Works for both streaming and non-streaming (`"stream": false`) responses.
fn extract_reply(bytes: &[u8]) -> Option<(String, String)> {
    let text = std::str::from_utf8(bytes).ok()?;
    let mut role = "assistant".to_string();
    let mut content = String::new();
    let mut found = false;

    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        if let Ok(chunk) = serde_json::from_str::<ChatResponse>(line) {
            role = chunk.message.role;
            content.push_str(&chunk.message.content);
            found = true;
        }
    }

    if found { Some((role, content)) } else { None }
}

/// Main handler for all proxied traffic. It reads the incoming request,
/// logs messages, truncates context for VRAM efficiency, and forwards to Ollama.
pub async fn proxy_handler(req: Request) -> impl IntoResponse {
    let client = reqwest::Client::new();
    
    // Target backend: Ollama's default local server address
    let backend_url = format!("http://127.0.0.1:11434{}", req.uri().path());

    // Extract request components to reconstruct for the backend
    let method = req.method().clone();
    let headers = req.headers().clone();
    
    // Read the incoming request body into raw bytes
    let body_bytes = match axum::body::to_bytes(req.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => return Response::builder().status(500).body(Body::from("Failed to read request body")).unwrap(),
    };

    let mut modified_body_bytes = body_bytes.clone();
    
    // 🔍 Single-pass parsing, logging, and VRAM budgeting
    if let Ok(mut chat_req) = serde_json::from_slice::<ChatRequest>(&body_bytes) {
        // Log all incoming messages to SQLite (full history preserved!)
        let mut latest_user_query = String::new();
        
        if let Ok(conn) = crate::db::init_db() {
            for msg in &chat_req.messages {
                println!("💾 Logging message to DB [{}]: {}", msg.role, msg.content);
                if msg.role == "user" {
                    latest_user_query = msg.content.clone();
                }
                if let Err(e) = crate::db::insert_message(&conn, &msg.role, &msg.content, None) {
                    eprintln!("❌ DB insert error: {}", e);
                }
            }

            // 🧠 AUGMENTED MEMORY RETRIEVAL (FTS5 Search Toggle)
            let fts_enabled = std::env::var("FTS_ENABLED")
                .map(|v| v != "false" && v != "0")
                .unwrap_or(true);

            if fts_enabled && !latest_user_query.is_empty() {
                if let Ok(past_memories) = crate::db::search_messages(&conn, &latest_user_query, 3) {
                    if !past_memories.is_empty() {
                        println!("🧠 FTS5 Memory Retrieved: found {} relevant past snippets.", past_memories.len());
                        
                        let memory_context = format!(
                            "[Long-Term Augmented Memory Context]:\n{}",
                            past_memories.join("\n---\n")
                        );

                        // Inject the retrieved memory as a system message or context helper
                        chat_req.messages.insert(
                            0,
                            ChatMessage {
                                role: "system".to_string(),
                                content: memory_context,
                            },
                        );
                    }
                }
            } else if !fts_enabled {
                println!("ℹ️ FTS5 Memory Retrieval is DISABLED via FTS_ENABLED=false");
            }
        }

        // ✂ BUDGETING / TRUNCATION LOGIC WITH CONFIGURABLE ENV LIMITS
        let max_total_messages: usize = std::env::var("MAX_CONTEXT_MESSAGES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(6);

        let max_tokens: usize = std::env::var("MAX_CONTEXT_TOKENS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(2048);

        if chat_req.messages.len() > max_total_messages {
            let has_system = chat_req.messages.first().map_or(false, |m| m.role == "system");
            
            if has_system {
                println!("🛡 System prompt/memory detected! Preserving it in the context window.");
                let system_msg = chat_req.messages.remove(0);
                
                let max_history = if max_total_messages > 1 { max_total_messages - 1 } else { 0 };
                if chat_req.messages.len() > max_history {
                    let split_index = chat_req.messages.len() - max_history;
                    chat_req.messages = chat_req.messages.split_off(split_index);
                }
                
                chat_req.messages.insert(0, system_msg);
            } else {
                println!(
                    "✂  Truncating context from {} to last {} messages to save VRAM",
                    chat_req.messages.len(),
                    max_total_messages
                );
                let split_index = chat_req.messages.len() - max_total_messages;
                chat_req.messages = chat_req.messages.split_off(split_index);
            }
        }

        // Enforce token budget across messages while preserving system prompt/memory if present
        let mut estimated_tokens = estimate_tokens(&chat_req.messages);
        if estimated_tokens > max_tokens {
            println!(
                "⚠ Payload token estimate (~{} tokens) exceeds max limit ({} tokens). Pruning older history...",
                estimated_tokens, max_tokens
            );

            let has_system = chat_req.messages.first().map_or(false, |m| m.role == "system");
            let system_msg = if has_system {
                Some(chat_req.messages.remove(0))
            } else {
                None
            };

            while chat_req.messages.len() > 1 && estimate_tokens(&chat_req.messages) > max_tokens {
                chat_req.messages.remove(0);
            }

            if let Some(sys) = system_msg {
                chat_req.messages.insert(0, sys);
            }

            estimated_tokens = estimate_tokens(&chat_req.messages);
            println!("✂  Context pruned to ~{} estimated tokens.", estimated_tokens);
        }

        // Re-serialize the modified request back into JSON bytes for Ollama
        if let Ok(new_bytes) = serde_json::to_vec(&chat_req) {
            modified_body_bytes = axum::body::Bytes::from(new_bytes);
        }
    }

    // Build the outgoing request to Ollama using `modified_body_bytes`
    let mut forward_req = client.request(method, &backend_url);
    for (key, value) in headers.iter() {
        if key == "content-length" || key == "host" {
            continue;
        }
        forward_req = forward_req.header(key, value);
    }
    forward_req = forward_req.body(modified_body_bytes);

    // ⏱ Start timer for latency measurement
    let start_time = std::time::Instant::now();

    // Send the request and handle the backend's response
    match forward_req.send().await {
        Ok(resp) => {
            let status = resp.status();
            let resp_headers = resp.headers().clone();
            
            let resp_bytes = match resp.bytes().await {
                Ok(bytes) => bytes,
                Err(_) => return Response::builder().status(500).body(Body::from("Failed to read backend response")).unwrap(),
            };

            let latency_ms = start_time.elapsed().as_millis() as u64;

            // 🔍 Log incoming ASSISTANT response to DB with latency!
            if let Some((role, content)) = extract_reply(&resp_bytes) {
                if let Ok(conn) = crate::db::init_db() {
                    println!("⏱ Ollama latency: {}ms (response saved to DB)", latency_ms);
                    if let Err(e) = crate::db::insert_message(&conn, &role, &content, Some(latency_ms)) {
                        eprintln!("❌ DB insert error: {}", e);
                    }
                }
            }

            let mut response_builder = Response::builder().status(status);
            for (key, value) in resp_headers.iter() {
                if key == "transfer-encoding" {
                    continue;
                }
                response_builder = response_builder.header(key, value);
            }

            response_builder.body(Body::from(resp_bytes)).unwrap()
        }
        Err(e) => {
            let latency_ms = start_time.elapsed().as_millis() as u64;
            eprintln!("❌ Proxy forwarding error after {}ms: {}", latency_ms, e);
            Response::builder().status(502).body(Body::from(format!("Bad Gateway: {}", e))).unwrap()
        }
    }
}
