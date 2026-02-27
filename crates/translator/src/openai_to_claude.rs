use ai_proxy_core::error::ProxyError;
use serde_json::{Value, json};

pub fn translate_request(
    model: &str,
    raw_json: &[u8],
    stream: bool,
) -> Result<Vec<u8>, ProxyError> {
    let mut req: Value = serde_json::from_slice(raw_json)?;

    // 1. Extract system messages from messages array
    let system_text = extract_system_messages(&mut req);

    // 2. Convert messages to Claude format
    let messages = convert_messages(&req)?;

    // 3. Convert tools
    let tools = convert_tools(&req);

    // 4. Determine max_tokens
    let max_tokens = req
        .get("max_tokens")
        .or_else(|| req.get("max_completion_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(8192);

    // 5. Convert stop sequences
    let stop_sequences = convert_stop_sequences(&req);

    // Build Claude request
    let mut claude_req = json!({
        "model": model,
        "messages": messages,
        "max_tokens": max_tokens,
    });

    if !system_text.is_empty() {
        claude_req["system"] = Value::String(system_text);
    }

    if let Some(temp) = req.get("temperature") {
        claude_req["temperature"] = temp.clone();
    }
    if let Some(top_p) = req.get("top_p") {
        claude_req["top_p"] = top_p.clone();
    }
    if let Some(tools) = tools {
        claude_req["tools"] = tools;
    }
    if let Some(stop) = stop_sequences {
        claude_req["stop_sequences"] = stop;
    }
    if stream {
        claude_req["stream"] = Value::Bool(true);
    }

    // Forward extended thinking (thinking/budget_tokens) if present
    if let Some(thinking) = req.get("thinking") {
        claude_req["thinking"] = thinking.clone();
    }

    // Forward tool_choice if present
    if let Some(tc) = req.get("tool_choice") {
        claude_req["tool_choice"] = convert_tool_choice(tc);
    }

    serde_json::to_vec(&claude_req).map_err(|e| ProxyError::Translation(e.to_string()))
}

fn extract_system_messages(req: &mut Value) -> String {
    let mut system_parts = Vec::new();
    if let Some(messages) = req.get("messages").and_then(|m| m.as_array()) {
        for msg in messages {
            if msg.get("role").and_then(|r| r.as_str()) == Some("system")
                && let Some(content) = msg.get("content")
            {
                match content {
                    Value::String(s) => system_parts.push(s.clone()),
                    Value::Array(parts) => {
                        for part in parts {
                            if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                system_parts.push(text.to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    system_parts.join("\n\n")
}

fn convert_messages(req: &Value) -> Result<Vec<Value>, ProxyError> {
    let messages = req
        .get("messages")
        .and_then(|m| m.as_array())
        .ok_or_else(|| ProxyError::Translation("missing messages field".to_string()))?;

    let mut claude_messages: Vec<Value> = Vec::new();

    for msg in messages {
        let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("");

        if role == "system" {
            continue;
        }

        if role == "tool" {
            // Convert tool result message to user message with tool_result content block
            let tool_call_id = msg
                .get("tool_call_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let content_text = match msg.get("content") {
                Some(Value::String(s)) => s.clone(),
                _ => String::new(),
            };

            let tool_result = json!({
                "type": "tool_result",
                "tool_use_id": tool_call_id,
                "content": content_text,
            });

            // Check if the last message is from the "user" role - merge tool results
            if let Some(last) = claude_messages.last_mut()
                && last.get("role").and_then(|r: &Value| r.as_str()) == Some("user")
                && let Some(content) = last.get_mut("content")
                && let Some(arr) = content.as_array_mut()
            {
                arr.push(tool_result);
                continue;
            }

            claude_messages.push(json!({
                "role": "user",
                "content": [tool_result],
            }));
            continue;
        }

        if role == "assistant" {
            let mut content_blocks = Vec::new();

            // Handle text content
            if let Some(content) = msg.get("content") {
                match content {
                    Value::String(s) if !s.is_empty() => {
                        content_blocks.push(json!({"type": "text", "text": s}));
                    }
                    _ => {}
                }
            }

            // Handle tool_calls -> tool_use blocks
            if let Some(tool_calls) = msg.get("tool_calls").and_then(|tc| tc.as_array()) {
                for tc in tool_calls {
                    let id = tc.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    let name = tc
                        .get("function")
                        .and_then(|f| f.get("name"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("");
                    let arguments_str = tc
                        .get("function")
                        .and_then(|f| f.get("arguments"))
                        .and_then(|a| a.as_str())
                        .unwrap_or("{}");
                    let input: Value = serde_json::from_str(arguments_str).unwrap_or(json!({}));

                    content_blocks.push(json!({
                        "type": "tool_use",
                        "id": id,
                        "name": name,
                        "input": input,
                    }));
                }
            }

            if content_blocks.is_empty() {
                content_blocks.push(json!({"type": "text", "text": ""}));
            }

            claude_messages.push(json!({
                "role": "assistant",
                "content": content_blocks,
            }));
            continue;
        }

        // User messages
        let claude_content = convert_user_content(msg.get("content"));
        claude_messages.push(json!({
            "role": "user",
            "content": claude_content,
        }));
    }

    Ok(claude_messages)
}

fn convert_user_content(content: Option<&Value>) -> Value {
    match content {
        Some(Value::String(s)) => Value::String(s.clone()),
        Some(Value::Array(parts)) => {
            let mut blocks = Vec::new();
            for part in parts {
                let part_type = part.get("type").and_then(|t| t.as_str()).unwrap_or("");
                match part_type {
                    "text" => {
                        let text = part.get("text").and_then(|t| t.as_str()).unwrap_or("");
                        blocks.push(json!({"type": "text", "text": text}));
                    }
                    "image_url" => {
                        if let Some(url_obj) = part.get("image_url") {
                            let url = url_obj.get("url").and_then(|u| u.as_str()).unwrap_or("");
                            if let Some(image_block) = convert_image_url(url) {
                                blocks.push(image_block);
                            }
                        }
                    }
                    _ => {}
                }
            }
            Value::Array(blocks)
        }
        _ => Value::String(String::new()),
    }
}

fn convert_image_url(url: &str) -> Option<Value> {
    // Handle base64 data URLs: data:image/png;base64,<data>
    if let Some(rest) = url.strip_prefix("data:") {
        let parts: Vec<&str> = rest.splitn(2, ',').collect();
        if parts.len() == 2 {
            let meta = parts[0]; // e.g., "image/png;base64"
            let data = parts[1];
            let media_type = meta.split(';').next().unwrap_or("image/png");
            return Some(json!({
                "type": "image",
                "source": {
                    "type": "base64",
                    "media_type": media_type,
                    "data": data,
                }
            }));
        }
    }
    // For regular URLs, use the url source type
    Some(json!({
        "type": "image",
        "source": {
            "type": "url",
            "url": url,
        }
    }))
}

fn convert_tools(req: &Value) -> Option<Value> {
    let tools = req.get("tools")?.as_array()?;
    let claude_tools: Vec<Value> = tools
        .iter()
        .filter_map(|tool| {
            let func = tool.get("function")?;
            let name = func.get("name")?.as_str()?;
            let description = func
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("");
            let parameters = func
                .get("parameters")
                .cloned()
                .unwrap_or(json!({"type": "object", "properties": {}}));
            Some(json!({
                "name": name,
                "description": description,
                "input_schema": parameters,
            }))
        })
        .collect();

    if claude_tools.is_empty() {
        None
    } else {
        Some(Value::Array(claude_tools))
    }
}

fn convert_stop_sequences(req: &Value) -> Option<Value> {
    let stop = req.get("stop")?;
    match stop {
        Value::String(s) => Some(json!([s])),
        Value::Array(_) => Some(stop.clone()),
        _ => None,
    }
}

fn convert_tool_choice(tc: &Value) -> Value {
    match tc {
        Value::String(s) => match s.as_str() {
            "none" => json!({"type": "none"}),
            "auto" => json!({"type": "auto"}),
            "required" => json!({"type": "any"}),
            _ => json!({"type": "auto"}),
        },
        Value::Object(obj) => {
            if let Some(func) = obj.get("function")
                && let Some(name) = func.get("name").and_then(|n| n.as_str())
            {
                return json!({"type": "tool", "name": name});
            }
            json!({"type": "auto"})
        }
        _ => json!({"type": "auto"}),
    }
}
