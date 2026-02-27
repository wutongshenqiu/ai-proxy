use crate::common;
use ai_proxy_core::error::ProxyError;
use ai_proxy_core::provider::*;
use async_trait::async_trait;

pub struct OpenAICompatExecutor {
    pub name: String,
    pub default_base_url: String,
    pub format: Format,
    pub global_proxy: Option<String>,
}

/// Check if the auth record uses the Responses API wire format.
fn use_responses_api(auth: &AuthRecord) -> bool {
    auth.wire_api == ai_proxy_core::provider::WireApi::Responses
}

/// Convert a Chat Completions request body to Responses API format.
fn chat_to_responses(payload: &[u8]) -> Result<Vec<u8>, ProxyError> {
    let mut v: serde_json::Value =
        serde_json::from_slice(payload).map_err(|e| ProxyError::BadRequest(e.to_string()))?;

    let obj = v
        .as_object_mut()
        .ok_or_else(|| ProxyError::BadRequest("expected JSON object".into()))?;

    // messages -> input
    if let Some(messages) = obj.remove("messages") {
        // Extract system messages as instructions
        if let Some(arr) = messages.as_array() {
            let mut instructions: Vec<String> = Vec::new();
            let mut input = Vec::new();
            for msg in arr {
                if msg.get("role").and_then(|r| r.as_str()) == Some("system") {
                    if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                        instructions.push(content.to_string());
                    }
                } else {
                    input.push(msg.clone());
                }
            }
            if !instructions.is_empty() && !obj.contains_key("instructions") {
                obj.insert(
                    "instructions".into(),
                    serde_json::Value::String(instructions.join("\n")),
                );
            }
            obj.insert("input".into(), serde_json::Value::Array(input));
        } else {
            obj.insert("input".into(), messages);
        }
    }

    // max_tokens -> max_output_tokens
    if let Some(max_tokens) = obj.remove("max_tokens")
        && !obj.contains_key("max_output_tokens")
    {
        obj.insert("max_output_tokens".into(), max_tokens);
    }

    // Remove Chat Completions-specific fields that Responses API doesn't accept
    obj.remove("stream");

    serde_json::to_vec(obj).map_err(|e| ProxyError::Internal(e.to_string()))
}

/// Convert a Responses API response to Chat Completions format.
fn responses_to_chat(payload: &[u8]) -> Result<bytes::Bytes, ProxyError> {
    let v: serde_json::Value =
        serde_json::from_slice(payload).map_err(|e| ProxyError::Internal(e.to_string()))?;

    // Extract content from output[].content[].text
    let mut content = String::new();
    if let Some(output) = v.get("output").and_then(|o| o.as_array()) {
        for item in output {
            if item.get("type").and_then(|t| t.as_str()) == Some("message")
                && let Some(contents) = item.get("content").and_then(|c| c.as_array())
            {
                for c in contents {
                    if c.get("type").and_then(|t| t.as_str()) == Some("output_text")
                        && let Some(text) = c.get("text").and_then(|t| t.as_str())
                    {
                        content.push_str(text);
                    }
                }
            }
        }
    }

    let model = v.get("model").and_then(|m| m.as_str()).unwrap_or("unknown");
    let id = v.get("id").and_then(|i| i.as_str()).unwrap_or("");
    let created = v.get("created_at").and_then(|c| c.as_u64()).unwrap_or(0);

    // Extract usage
    let usage = v
        .get("usage")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let prompt_tokens = usage
        .get("input_tokens")
        .and_then(|t| t.as_u64())
        .unwrap_or(0);
    let completion_tokens = usage
        .get("output_tokens")
        .and_then(|t| t.as_u64())
        .unwrap_or(0);

    let finish_reason = match v.get("status").and_then(|s| s.as_str()) {
        Some("completed") => "stop",
        Some("incomplete") => "length",
        _ => "stop",
    };

    let chat_response = serde_json::json!({
        "id": format!("chatcmpl-{id}"),
        "object": "chat.completion",
        "created": created,
        "model": model,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": content,
            },
            "finish_reason": finish_reason,
        }],
        "usage": {
            "prompt_tokens": prompt_tokens,
            "completion_tokens": completion_tokens,
            "total_tokens": prompt_tokens + completion_tokens,
        }
    });

    serde_json::to_vec(&chat_response)
        .map(bytes::Bytes::from)
        .map_err(|e| ProxyError::Internal(e.to_string()))
}

#[async_trait]
impl ProviderExecutor for OpenAICompatExecutor {
    fn identifier(&self) -> &str {
        &self.name
    }

    fn native_format(&self) -> Format {
        self.format
    }

    fn default_base_url(&self) -> &str {
        &self.default_base_url
    }

    async fn execute(
        &self,
        auth: &AuthRecord,
        request: ProviderRequest,
    ) -> Result<ProviderResponse, ProxyError> {
        let client = common::build_client(auth, self.global_proxy.as_deref())?;
        let base_url = auth.base_url_or_default(&self.default_base_url);

        let (url, body) = if use_responses_api(auth) {
            (
                format!("{base_url}/v1/responses"),
                chat_to_responses(&request.payload)?,
            )
        } else {
            (
                format!("{base_url}/v1/chat/completions"),
                request.payload.to_vec(),
            )
        };

        let mut req = client
            .post(&url)
            .header("authorization", format!("Bearer {}", auth.api_key))
            .header("content-type", "application/json")
            .body(body);

        for (k, v) in &auth.headers {
            req = req.header(k.as_str(), v.as_str());
        }

        let (resp_body, headers) = common::handle_response(req.send().await?).await?;

        // Convert Responses API response back to Chat Completions format
        let payload = if use_responses_api(auth) {
            responses_to_chat(&resp_body)?
        } else {
            resp_body
        };

        Ok(ProviderResponse { payload, headers })
    }

    async fn execute_stream(
        &self,
        auth: &AuthRecord,
        request: ProviderRequest,
    ) -> Result<StreamResult, ProxyError> {
        if use_responses_api(auth) {
            // Responses API: execute non-streaming, then emit as streaming chunks.
            let response = self.execute(auth, request).await?;
            let v: serde_json::Value = serde_json::from_slice(&response.payload)
                .map_err(|e| ProxyError::Internal(e.to_string()))?;

            let content = v
                .get("choices")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("message"))
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str())
                .unwrap_or("");
            let model = v.get("model").and_then(|m| m.as_str()).unwrap_or("unknown");
            let id = v.get("id").and_then(|i| i.as_str()).unwrap_or("");
            let created = v.get("created").and_then(|c| c.as_u64()).unwrap_or(0);

            // Emit: role chunk, content chunk, finish chunk, [DONE]
            let role_chunk = serde_json::json!({
                "id": id, "object": "chat.completion.chunk", "created": created, "model": model,
                "choices": [{"index": 0, "delta": {"role": "assistant", "content": ""}, "finish_reason": null}]
            });
            let content_chunk = serde_json::json!({
                "id": id, "object": "chat.completion.chunk", "created": created, "model": model,
                "choices": [{"index": 0, "delta": {"content": content}, "finish_reason": null}]
            });
            let usage = v.get("usage").cloned().unwrap_or(serde_json::json!({}));
            let stop_chunk = serde_json::json!({
                "id": id, "object": "chat.completion.chunk", "created": created, "model": model,
                "choices": [{"index": 0, "delta": {}, "finish_reason": "stop"}],
                "usage": usage,
            });

            let chunks: Vec<Result<StreamChunk, ProxyError>> = vec![
                Ok(StreamChunk {
                    event_type: None,
                    data: role_chunk.to_string(),
                }),
                Ok(StreamChunk {
                    event_type: None,
                    data: content_chunk.to_string(),
                }),
                Ok(StreamChunk {
                    event_type: None,
                    data: stop_chunk.to_string(),
                }),
                Ok(StreamChunk {
                    event_type: None,
                    data: "[DONE]".to_string(),
                }),
            ];
            return Ok(StreamResult {
                headers: response.headers,
                stream: Box::pin(futures::stream::iter(chunks)),
            });
        }

        let client = common::build_client(auth, self.global_proxy.as_deref())?;
        let base_url = auth.base_url_or_default(&self.default_base_url);
        let url = format!("{base_url}/v1/chat/completions");

        let mut req = client
            .post(&url)
            .header("authorization", format!("Bearer {}", auth.api_key))
            .header("content-type", "application/json")
            .body(request.payload.to_vec());

        for (k, v) in &auth.headers {
            req = req.header(k.as_str(), v.as_str());
        }

        common::handle_stream_response(req.send().await?).await
    }

    fn supported_models(&self, auth: &AuthRecord) -> Vec<ModelInfo> {
        common::supported_models_from_auth(auth, &self.name, &self.name)
    }
}
