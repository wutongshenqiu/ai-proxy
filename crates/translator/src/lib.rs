pub mod claude_to_openai;
pub mod gemini_to_openai;
pub mod openai_to_claude;
pub mod openai_to_gemini;

use ai_proxy_core::error::ProxyError;
use ai_proxy_core::provider::Format;
use std::collections::HashMap;

/// State accumulated during stream translation.
#[derive(Debug, Default)]
pub struct TranslateState {
    pub response_id: String,
    pub model: String,
    pub created: i64,
    pub current_tool_call_index: i32,
    pub current_content_index: i32,
    pub sent_role: bool,
    pub input_tokens: u64,
}

pub type RequestTransformFn =
    fn(model: &str, raw_json: &[u8], stream: bool) -> Result<Vec<u8>, ProxyError>;

pub type StreamTransformFn = fn(
    model: &str,
    original_req: &[u8],
    event_type: Option<&str>,
    data: &[u8],
    state: &mut TranslateState,
) -> Result<Vec<String>, ProxyError>;

pub type NonStreamTransformFn =
    fn(model: &str, original_req: &[u8], data: &[u8]) -> Result<String, ProxyError>;

pub struct ResponseTransform {
    pub stream: StreamTransformFn,
    pub non_stream: NonStreamTransformFn,
}

pub struct TranslatorRegistry {
    requests: HashMap<(Format, Format), RequestTransformFn>,
    responses: HashMap<(Format, Format), ResponseTransform>,
}

impl Default for TranslatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TranslatorRegistry {
    pub fn new() -> Self {
        Self {
            requests: HashMap::new(),
            responses: HashMap::new(),
        }
    }

    pub fn register(
        &mut self,
        from: Format,
        to: Format,
        request: RequestTransformFn,
        response: ResponseTransform,
    ) {
        self.requests.insert((from, to), request);
        self.responses.insert((from, to), response);
    }

    pub fn translate_request(
        &self,
        from: Format,
        to: Format,
        model: &str,
        raw_json: &[u8],
        stream: bool,
    ) -> Result<Vec<u8>, ProxyError> {
        if from == to {
            // Even for passthrough, replace the model name (alias → actual ID)
            return replace_model_in_payload(raw_json, model);
        }
        match self.requests.get(&(from, to)) {
            Some(f) => f(model, raw_json, stream),
            None => Ok(raw_json.to_vec()),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn translate_stream(
        &self,
        from: Format,
        to: Format,
        model: &str,
        orig_req: &[u8],
        event_type: Option<&str>,
        data: &[u8],
        state: &mut TranslateState,
    ) -> Result<Vec<String>, ProxyError> {
        if from == to {
            let line = String::from_utf8_lossy(data).to_string();
            // Pass through [DONE] sentinel and raw data as-is
            return Ok(vec![line]);
        }
        // Skip [DONE] sentinel for translation paths (translators produce their own)
        if data == b"[DONE]" {
            return Ok(vec!["[DONE]".to_string()]);
        }
        match self.responses.get(&(from, to)) {
            Some(rt) => (rt.stream)(model, orig_req, event_type, data, state),
            None => {
                let line = String::from_utf8_lossy(data).to_string();
                Ok(vec![line])
            }
        }
    }

    pub fn translate_non_stream(
        &self,
        from: Format,
        to: Format,
        model: &str,
        orig_req: &[u8],
        data: &[u8],
    ) -> Result<String, ProxyError> {
        if from == to {
            return Ok(String::from_utf8_lossy(data).to_string());
        }
        match self.responses.get(&(from, to)) {
            Some(rt) => (rt.non_stream)(model, orig_req, data),
            None => Ok(String::from_utf8_lossy(data).to_string()),
        }
    }

    pub fn has_response_translator(&self, from: Format, to: Format) -> bool {
        from != to && self.responses.contains_key(&(from, to))
    }
}

/// Replace the "model" field in a JSON payload with the resolved model name.
fn replace_model_in_payload(raw_json: &[u8], model: &str) -> Result<Vec<u8>, ProxyError> {
    let mut val: serde_json::Value = serde_json::from_slice(raw_json)?;
    if let Some(obj) = val.as_object_mut()
        && obj.contains_key("model")
    {
        obj.insert(
            "model".to_string(),
            serde_json::Value::String(model.to_string()),
        );
    }
    serde_json::to_vec(&val).map_err(|e| ProxyError::Translation(e.to_string()))
}

pub fn build_registry() -> TranslatorRegistry {
    let mut reg = TranslatorRegistry::new();

    // OpenAI -> Claude request translation, Claude -> OpenAI response translation
    reg.register(
        Format::OpenAI,
        Format::Claude,
        openai_to_claude::translate_request,
        ResponseTransform {
            stream: claude_to_openai::translate_stream,
            non_stream: claude_to_openai::translate_non_stream,
        },
    );

    // OpenAI -> Gemini request translation, Gemini -> OpenAI response translation
    reg.register(
        Format::OpenAI,
        Format::Gemini,
        openai_to_gemini::translate_request,
        ResponseTransform {
            stream: gemini_to_openai::translate_stream,
            non_stream: gemini_to_openai::translate_non_stream,
        },
    );

    reg
}
