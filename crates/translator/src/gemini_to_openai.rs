use crate::TranslateState;
use ai_proxy_core::error::ProxyError;
use serde_json::{json, Value};

pub fn translate_non_stream(
    _model: &str,
    _original_req: &[u8],
    data: &[u8],
) -> Result<String, ProxyError> {
    let resp: Value = serde_json::from_slice(data)?;
    let created = chrono::Utc::now().timestamp();
    let id = format!("chatcmpl-{}", uuid::Uuid::new_v4());

    let model = resp
        .get("modelVersion")
        .and_then(|v| v.as_str())
        .unwrap_or("gemini")
        .to_string();

    // Extract first candidate
    let candidate = resp
        .get("candidates")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first());

    let (content_str, tool_calls, finish_reason) = if let Some(candidate) = candidate {
        let parts = candidate
            .get("content")
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.as_array());

        let mut text_parts = Vec::new();
        let mut tool_calls = Vec::new();
        let mut tc_index = 0u32;

        if let Some(parts) = parts {
            for part in parts {
                if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                    text_parts.push(text.to_string());
                } else if let Some(fc) = part.get("functionCall") {
                    let name = fc
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("")
                        .to_string();
                    let args = fc.get("args").cloned().unwrap_or(json!({}));
                    let arguments = serde_json::to_string(&args).unwrap_or_default();
                    let tc_id = format!("call_{}", uuid::Uuid::new_v4());

                    tool_calls.push(json!({
                        "id": tc_id,
                        "type": "function",
                        "function": {
                            "name": name,
                            "arguments": arguments,
                        },
                        "index": tc_index,
                    }));
                    tc_index += 1;
                }
            }
        }

        let finish = match candidate.get("finishReason").and_then(|v| v.as_str()) {
            Some("STOP") => "stop",
            Some("MAX_TOKENS") => "length",
            Some("SAFETY") => "content_filter",
            Some("RECITATION") => "content_filter",
            _ => "stop",
        };

        (text_parts.join(""), tool_calls, finish)
    } else {
        (String::new(), Vec::new(), "stop")
    };

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
    let usage = if let Some(u) = resp.get("usageMetadata") {
        let prompt = u
            .get("promptTokenCount")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let completion = u
            .get("candidatesTokenCount")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let total = u
            .get("totalTokenCount")
            .and_then(|v| v.as_u64())
            .unwrap_or(prompt + completion);
        Some(json!({
            "prompt_tokens": prompt,
            "completion_tokens": completion,
            "total_tokens": total,
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
    _event_type: Option<&str>,
    data: &[u8],
    state: &mut TranslateState,
) -> Result<Vec<String>, ProxyError> {
    let resp: Value = serde_json::from_slice(data)?;
    let mut chunks = Vec::new();

    // Initialize state if needed
    if state.response_id.is_empty() {
        state.response_id = format!("chatcmpl-{}", uuid::Uuid::new_v4());
        state.created = chrono::Utc::now().timestamp();
        state.current_tool_call_index = -1;

        // Emit initial role chunk
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
        chunks.push(serde_json::to_string(&chunk)?);
    }

    // Extract candidate
    let candidate = resp
        .get("candidates")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first());

    if let Some(candidate) = candidate {
        // Update model from response if available
        if let Some(model_ver) = resp.get("modelVersion").and_then(|v| v.as_str()) {
            state.model = model_ver.to_string();
        }

        let parts = candidate
            .get("content")
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.as_array());

        if let Some(parts) = parts {
            for part in parts {
                if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
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
                } else if let Some(fc) = part.get("functionCall") {
                    state.current_tool_call_index += 1;
                    let name = fc
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("")
                        .to_string();
                    let args = fc.get("args").cloned().unwrap_or(json!({}));
                    let arguments = serde_json::to_string(&args).unwrap_or_default();
                    let tc_id = format!("call_{}", uuid::Uuid::new_v4());

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
                                        "arguments": arguments,
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

        // Check for finish_reason
        if let Some(finish) = candidate.get("finishReason").and_then(|v| v.as_str()) {
            let finish_reason = match finish {
                "STOP" => "stop",
                "MAX_TOKENS" => "length",
                "SAFETY" => "content_filter",
                "RECITATION" => "content_filter",
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
            if let Some(u) = resp.get("usageMetadata") {
                let prompt = u.get("promptTokenCount").and_then(|v| v.as_u64()).unwrap_or(0);
                let completion = u.get("candidatesTokenCount").and_then(|v| v.as_u64()).unwrap_or(0);
                chunk["usage"] = json!({
                    "prompt_tokens": prompt,
                    "completion_tokens": completion,
                    "total_tokens": prompt + completion,
                });
            }

            chunks.push(serde_json::to_string(&chunk)?);
            chunks.push("[DONE]".to_string());
        }
    }

    Ok(chunks)
}
