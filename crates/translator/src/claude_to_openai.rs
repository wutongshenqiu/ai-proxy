use crate::TranslateState;
use ai_proxy_core::error::ProxyError;
use serde_json::{Value, json};

pub fn translate_non_stream(
    _model: &str,
    _original_req: &[u8],
    data: &[u8],
) -> Result<String, ProxyError> {
    let resp: Value = serde_json::from_slice(data)?;

    let id = format!(
        "chatcmpl-{}",
        resp.get("id").and_then(|v| v.as_str()).unwrap_or("unknown")
    );
    let model = resp
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let created = chrono::Utc::now().timestamp();

    // Extract text content and tool_use blocks
    let mut text_parts = Vec::new();
    let mut tool_calls = Vec::new();
    let mut tool_call_index = 0u32;

    if let Some(content) = resp.get("content").and_then(|c| c.as_array()) {
        for block in content {
            let block_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
            match block_type {
                "text" => {
                    if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                        text_parts.push(text.to_string());
                    }
                }
                "tool_use" => {
                    let tc_id = block
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let name = block
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let input = block.get("input").cloned().unwrap_or(json!({}));
                    let arguments = serde_json::to_string(&input).unwrap_or_default();

                    tool_calls.push(json!({
                        "id": tc_id,
                        "type": "function",
                        "function": {
                            "name": name,
                            "arguments": arguments,
                        },
                        "index": tool_call_index,
                    }));
                    tool_call_index += 1;
                }
                _ => {}
            }
        }
    }

    // Map stop_reason to finish_reason
    let finish_reason = match resp.get("stop_reason").and_then(|v| v.as_str()) {
        Some("end_turn") => "stop",
        Some("max_tokens") => "length",
        Some("tool_use") => "tool_calls",
        Some("stop_sequence") => "stop",
        _ => "stop",
    };

    let content_str = text_parts.join("");
    let content_val = if content_str.is_empty() && !tool_calls.is_empty() {
        Value::Null
    } else {
        Value::String(content_str)
    };

    let mut message = json!({
        "role": "assistant",
        "content": content_val,
    });

    if !tool_calls.is_empty() {
        message["tool_calls"] = Value::Array(tool_calls);
    }

    // Map usage
    let usage = if let Some(u) = resp.get("usage") {
        let input_tokens = u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
        let output_tokens = u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
        Some(json!({
            "prompt_tokens": input_tokens,
            "completion_tokens": output_tokens,
            "total_tokens": input_tokens + output_tokens,
        }))
    } else {
        None
    };

    let mut openai_resp = json!({
        "id": id,
        "object": "chat.completion",
        "created": created,
        "model": model,
        "choices": [{
            "index": 0,
            "message": message,
            "finish_reason": finish_reason,
        }],
    });

    if let Some(usage) = usage {
        openai_resp["usage"] = usage;
    }

    serde_json::to_string(&openai_resp).map_err(|e| ProxyError::Translation(e.to_string()))
}

pub fn translate_stream(
    _model: &str,
    _original_req: &[u8],
    event_type: Option<&str>,
    data: &[u8],
    state: &mut TranslateState,
) -> Result<Vec<String>, ProxyError> {
    let event: Value = serde_json::from_slice(data)?;
    let mut chunks = Vec::new();

    match event_type {
        Some("message_start") => {
            if let Some(msg) = event.get("message") {
                state.response_id = format!(
                    "chatcmpl-{}",
                    msg.get("id").and_then(|v| v.as_str()).unwrap_or("unknown")
                );
                state.model = msg
                    .get("model")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                state.created = chrono::Utc::now().timestamp();
                state.current_content_index = -1;
                state.current_tool_call_index = -1;
                state.sent_role = false;
                state.input_tokens = msg
                    .get("usage")
                    .and_then(|u| u.get("input_tokens"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
            }

            // Emit initial chunk with role
            let chunk = json!({
                "id": state.response_id,
                "object": "chat.completion.chunk",
                "created": state.created,
                "model": state.model,
                "choices": [{
                    "index": 0,
                    "delta": {"role": "assistant", "content": ""},
                    "finish_reason": null,
                }],
            });
            state.sent_role = true;
            chunks.push(serde_json::to_string(&chunk)?);
        }

        Some("content_block_start") => {
            state.current_content_index += 1;

            if let Some(cb) = event.get("content_block") {
                let block_type = cb.get("type").and_then(|t| t.as_str()).unwrap_or("");
                if block_type == "tool_use" {
                    state.current_tool_call_index += 1;
                    let tc_id = cb
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let name = cb
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let chunk = json!({
                        "id": state.response_id,
                        "object": "chat.completion.chunk",
                        "created": state.created,
                        "model": state.model,
                        "choices": [{
                            "index": 0,
                            "delta": {
                                "tool_calls": [{
                                    "index": state.current_tool_call_index,
                                    "id": tc_id,
                                    "type": "function",
                                    "function": {
                                        "name": name,
                                        "arguments": "",
                                    },
                                }],
                            },
                            "finish_reason": null,
                        }],
                    });
                    chunks.push(serde_json::to_string(&chunk)?);
                }
            }
        }

        Some("content_block_delta") => {
            if let Some(delta) = event.get("delta") {
                let delta_type = delta.get("type").and_then(|t| t.as_str()).unwrap_or("");
                match delta_type {
                    "text_delta" => {
                        let text = delta.get("text").and_then(|t| t.as_str()).unwrap_or("");
                        let chunk = json!({
                            "id": state.response_id,
                            "object": "chat.completion.chunk",
                            "created": state.created,
                            "model": state.model,
                            "choices": [{
                                "index": 0,
                                "delta": {"content": text},
                                "finish_reason": null,
                            }],
                        });
                        chunks.push(serde_json::to_string(&chunk)?);
                    }
                    "input_json_delta" => {
                        let partial = delta
                            .get("partial_json")
                            .and_then(|t| t.as_str())
                            .unwrap_or("");
                        let chunk = json!({
                            "id": state.response_id,
                            "object": "chat.completion.chunk",
                            "created": state.created,
                            "model": state.model,
                            "choices": [{
                                "index": 0,
                                "delta": {
                                    "tool_calls": [{
                                        "index": state.current_tool_call_index,
                                        "function": {
                                            "arguments": partial,
                                        },
                                    }],
                                },
                                "finish_reason": null,
                            }],
                        });
                        chunks.push(serde_json::to_string(&chunk)?);
                    }
                    _ => {}
                }
            }
        }

        Some("message_delta") => {
            if let Some(delta) = event.get("delta") {
                let finish_reason = match delta.get("stop_reason").and_then(|v| v.as_str()) {
                    Some("end_turn") => "stop",
                    Some("max_tokens") => "length",
                    Some("tool_use") => "tool_calls",
                    Some("stop_sequence") => "stop",
                    _ => "stop",
                };

                let mut chunk = json!({
                    "id": state.response_id,
                    "object": "chat.completion.chunk",
                    "created": state.created,
                    "model": state.model,
                    "choices": [{
                        "index": 0,
                        "delta": {},
                        "finish_reason": finish_reason,
                    }],
                });

                // Include usage if available
                if let Some(usage) = event.get("usage") {
                    let output_tokens = usage
                        .get("output_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let input_tokens = state.input_tokens;
                    chunk["usage"] = json!({
                        "prompt_tokens": input_tokens,
                        "completion_tokens": output_tokens,
                        "total_tokens": input_tokens + output_tokens,
                    });
                }

                chunks.push(serde_json::to_string(&chunk)?);
            }
        }

        Some("message_stop") => {
            chunks.push("[DONE]".to_string());
        }

        _ => {
            // ping, content_block_stop, etc. - skip
        }
    }

    Ok(chunks)
}
