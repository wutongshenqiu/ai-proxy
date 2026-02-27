use ai_proxy_core::error::ProxyError;
use serde_json::{json, Value};

pub fn translate_request(
    model: &str,
    raw_json: &[u8],
    _stream: bool,
) -> Result<Vec<u8>, ProxyError> {
    let req: Value = serde_json::from_slice(raw_json)?;

    // 1. Extract system messages -> systemInstruction
    let system_instruction = extract_system_instruction(&req);

    // 2. Convert messages -> contents
    let contents = convert_messages(&req)?;

    // 3. Convert tools
    let tools = convert_tools(&req);

    // 4. Build generationConfig
    let generation_config = build_generation_config(&req);

    // Build Gemini request
    let mut gemini_req = json!({
        "contents": contents,
    });

    if let Some(si) = system_instruction {
        gemini_req["systemInstruction"] = si;
    }
    if let Some(gc) = generation_config {
        gemini_req["generationConfig"] = gc;
    }
    if let Some(tools) = tools {
        gemini_req["tools"] = tools;
    }

    // model is used in URL routing, not in the body for Gemini
    let _ = model;

    serde_json::to_vec(&gemini_req).map_err(|e| ProxyError::Translation(e.to_string()))
}

fn extract_system_instruction(req: &Value) -> Option<Value> {
    let messages = req.get("messages")?.as_array()?;
    let mut system_parts = Vec::new();

    for msg in messages {
        if msg.get("role").and_then(|r| r.as_str()) == Some("system") {
            if let Some(content) = msg.get("content") {
                match content {
                    Value::String(s) => {
                        system_parts.push(json!({"text": s}));
                    }
                    Value::Array(parts) => {
                        for part in parts {
                            if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                system_parts.push(json!({"text": text}));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    if system_parts.is_empty() {
        None
    } else {
        Some(json!({
            "parts": system_parts,
        }))
    }
}

fn convert_messages(req: &Value) -> Result<Vec<Value>, ProxyError> {
    let messages = req
        .get("messages")
        .and_then(|m| m.as_array())
        .ok_or_else(|| ProxyError::Translation("missing messages field".to_string()))?;

    let mut contents: Vec<Value> = Vec::new();

    for msg in messages {
        let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("");

        if role == "system" {
            continue;
        }

        if role == "tool" {
            // Convert to functionResponse part
            let name = msg
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("function");
            let content_text = msg
                .get("content")
                .and_then(|c| c.as_str())
                .unwrap_or("");

            // Try to parse content as JSON, fallback to wrapping in result object
            let response_val = serde_json::from_str::<Value>(content_text)
                .unwrap_or(json!({"result": content_text}));

            let part = json!({
                "functionResponse": {
                    "name": name,
                    "response": response_val,
                }
            });

            // Merge with previous user content if last was also user/function
            if let Some(last) = contents.last_mut() {
                if last.get("role").and_then(|r: &Value| r.as_str()) == Some("user") {
                    if let Some(parts) = last.get_mut("parts").and_then(|p: &mut Value| p.as_array_mut()) {
                        parts.push(part);
                        continue;
                    }
                }
            }

            contents.push(json!({
                "role": "user",
                "parts": [part],
            }));
            continue;
        }

        let gemini_role = match role {
            "assistant" => "model",
            _ => "user",
        };

        let parts = convert_content_to_parts(msg)?;

        // If the role matches the previous message, merge parts
        if let Some(last) = contents.last_mut() {
            if last.get("role").and_then(|r: &Value| r.as_str()) == Some(gemini_role) {
                if let Some(existing_parts) = last.get_mut("parts").and_then(|p: &mut Value| p.as_array_mut())
                {
                    existing_parts.extend(parts);
                    continue;
                }
            }
        }

        contents.push(json!({
            "role": gemini_role,
            "parts": parts,
        }));
    }

    Ok(contents)
}

fn convert_content_to_parts(msg: &Value) -> Result<Vec<Value>, ProxyError> {
    let mut parts = Vec::new();

    // Handle text/multipart content
    if let Some(content) = msg.get("content") {
        match content {
            Value::String(s) => {
                parts.push(json!({"text": s}));
            }
            Value::Array(content_parts) => {
                for part in content_parts {
                    let part_type = part.get("type").and_then(|t| t.as_str()).unwrap_or("");
                    match part_type {
                        "text" => {
                            let text = part.get("text").and_then(|t| t.as_str()).unwrap_or("");
                            parts.push(json!({"text": text}));
                        }
                        "image_url" => {
                            if let Some(url_obj) = part.get("image_url") {
                                let url = url_obj
                                    .get("url")
                                    .and_then(|u| u.as_str())
                                    .unwrap_or("");
                                if let Some(inline) = convert_image_url_to_inline(url) {
                                    parts.push(inline);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Value::Null => {}
            _ => {}
        }
    }

    // Handle tool_calls in assistant messages -> functionCall parts
    if let Some(tool_calls) = msg.get("tool_calls").and_then(|tc| tc.as_array()) {
        for tc in tool_calls {
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
            let args: Value = serde_json::from_str(arguments_str).unwrap_or(json!({}));

            parts.push(json!({
                "functionCall": {
                    "name": name,
                    "args": args,
                }
            }));
        }
    }

    if parts.is_empty() {
        parts.push(json!({"text": ""}));
    }

    Ok(parts)
}

fn convert_image_url_to_inline(url: &str) -> Option<Value> {
    if let Some(rest) = url.strip_prefix("data:") {
        let parts: Vec<&str> = rest.splitn(2, ',').collect();
        if parts.len() == 2 {
            let meta = parts[0];
            let data = parts[1];
            let mime_type = meta.split(';').next().unwrap_or("image/png");
            return Some(json!({
                "inlineData": {
                    "mimeType": mime_type,
                    "data": data,
                }
            }));
        }
    }
    // Non-base64 URLs cannot be directly sent as inline data to Gemini
    // Return as text reference for now
    Some(json!({"text": format!("[image: {}]", url)}))
}

fn convert_tools(req: &Value) -> Option<Value> {
    let tools = req.get("tools")?.as_array()?;
    let mut function_declarations = Vec::new();

    for tool in tools {
        if let Some(func) = tool.get("function") {
            let name = func.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let description = func
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("");
            let parameters = func.get("parameters").cloned();

            let mut decl = json!({
                "name": name,
                "description": description,
            });
            if let Some(params) = parameters {
                decl["parameters"] = params;
            }

            function_declarations.push(decl);
        }
    }

    if function_declarations.is_empty() {
        None
    } else {
        Some(json!([{
            "functionDeclarations": function_declarations,
        }]))
    }
}

fn build_generation_config(req: &Value) -> Option<Value> {
    let mut config = json!({});
    let mut has_any = false;

    if let Some(temp) = req.get("temperature") {
        config["temperature"] = temp.clone();
        has_any = true;
    }
    if let Some(top_p) = req.get("top_p") {
        config["topP"] = top_p.clone();
        has_any = true;
    }
    if let Some(max) = req.get("max_tokens").or(req.get("max_completion_tokens")) {
        config["maxOutputTokens"] = max.clone();
        has_any = true;
    }
    if let Some(stop) = req.get("stop") {
        match stop {
            Value::String(s) => {
                config["stopSequences"] = json!([s]);
                has_any = true;
            }
            Value::Array(_) => {
                config["stopSequences"] = stop.clone();
                has_any = true;
            }
            _ => {}
        }
    }

    if has_any {
        Some(config)
    } else {
        None
    }
}
