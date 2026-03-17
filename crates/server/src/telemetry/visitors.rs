use super::span_data::RequestSpanData;
use prism_core::request_record::AttemptSummary;
use tracing::field::{Field, Visit};

/// Visitor for recording fields from a `gateway.request` span into `RequestSpanData`.
pub struct RequestSpanVisitor<'a> {
    pub data: &'a mut RequestSpanData,
}

impl<'a> RequestSpanVisitor<'a> {
    pub fn new(data: &'a mut RequestSpanData) -> Self {
        Self { data }
    }

    fn set_optional_string(slot: &mut Option<String>, value: &str) {
        if value.is_empty() {
            *slot = None;
        } else {
            *slot = Some(value.to_string());
        }
    }
}

impl Visit for RequestSpanVisitor<'_> {
    fn record_str(&mut self, field: &Field, value: &str) {
        match field.name() {
            "request_id" => self.data.request_id = value.to_string(),
            "method" => self.data.method = value.to_string(),
            "path" => self.data.path = value.to_string(),
            "requested_model" => Self::set_optional_string(&mut self.data.requested_model, value),
            "request_body" => Self::set_optional_string(&mut self.data.request_body, value),
            "upstream_request_body" => {
                Self::set_optional_string(&mut self.data.upstream_request_body, value)
            }
            "provider" => Self::set_optional_string(&mut self.data.provider, value),
            "model" => Self::set_optional_string(&mut self.data.model, value),
            "credential_name" => Self::set_optional_string(&mut self.data.credential_name, value),
            "response_body" => Self::set_optional_string(&mut self.data.response_body, value),
            "stream_content_preview" => {
                Self::set_optional_string(&mut self.data.stream_content_preview, value);
            }
            "error" => Self::set_optional_string(&mut self.data.error, value),
            "error_type" => Self::set_optional_string(&mut self.data.error_type, value),
            "api_key_id" => Self::set_optional_string(&mut self.data.api_key_id, value),
            "tenant_id" => Self::set_optional_string(&mut self.data.tenant_id, value),
            "client_ip" => Self::set_optional_string(&mut self.data.client_ip, value),
            "client_region" => Self::set_optional_string(&mut self.data.client_region, value),
            _ => {}
        }
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        match field.name() {
            "status" => self.data.status = value as u16,
            "latency_ms" => self.data.latency_ms = value,
            "total_attempts" => self.data.total_attempts = value as u32,
            "usage_input" => self.data.usage_input = Some(value),
            "usage_output" => self.data.usage_output = Some(value),
            "usage_cache_read" => self.data.usage_cache_read = Some(value),
            "usage_cache_creation" => self.data.usage_cache_creation = Some(value),
            _ => {}
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        // tracing may pass integers as i64
        self.record_u64(field, value as u64);
    }

    fn record_i128(&mut self, field: &Field, value: i128) {
        self.record_u64(field, value as u64);
    }

    fn record_u128(&mut self, field: &Field, value: u128) {
        self.record_u64(field, value as u64);
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        if field.name() == "cost" {
            self.data.cost = Some(value);
        }
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        if field.name() == "stream" {
            self.data.stream = value;
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let rendered = format!("{value:?}");
        let rendered = rendered.trim_matches('"');
        match field.name() {
            "request_id" => self.data.request_id = rendered.to_string(),
            "method" => self.data.method = rendered.to_string(),
            "path" => self.data.path = rendered.to_string(),
            "requested_model" => {
                Self::set_optional_string(&mut self.data.requested_model, rendered)
            }
            "provider" => Self::set_optional_string(&mut self.data.provider, rendered),
            "model" => Self::set_optional_string(&mut self.data.model, rendered),
            "credential_name" => {
                Self::set_optional_string(&mut self.data.credential_name, rendered)
            }
            "response_body" => Self::set_optional_string(&mut self.data.response_body, rendered),
            "stream_content_preview" => {
                Self::set_optional_string(&mut self.data.stream_content_preview, rendered)
            }
            "error" => Self::set_optional_string(&mut self.data.error, rendered),
            "error_type" => Self::set_optional_string(&mut self.data.error_type, rendered),
            "api_key_id" => Self::set_optional_string(&mut self.data.api_key_id, rendered),
            "tenant_id" => Self::set_optional_string(&mut self.data.tenant_id, rendered),
            "client_ip" => Self::set_optional_string(&mut self.data.client_ip, rendered),
            "client_region" => Self::set_optional_string(&mut self.data.client_region, rendered),
            _ => {}
        }
    }
}

/// Visitor for recording fields from a `gateway.attempt` span into `AttemptSummary`.
pub struct AttemptSpanVisitor<'a> {
    pub data: &'a mut AttemptSummary,
}

impl<'a> AttemptSpanVisitor<'a> {
    pub fn new(data: &'a mut AttemptSummary) -> Self {
        Self { data }
    }
}

impl Visit for AttemptSpanVisitor<'_> {
    fn record_str(&mut self, field: &Field, value: &str) {
        match field.name() {
            "provider" => self.data.provider = value.to_string(),
            "model" => self.data.model = value.to_string(),
            "credential_name" => self.data.credential_name = Some(value.to_string()),
            "error" => self.data.error = Some(value.to_string()),
            "error_type" => self.data.error_type = Some(value.to_string()),
            _ => {}
        }
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        match field.name() {
            "attempt_index" => self.data.attempt_index = value as u32,
            "status" => self.data.status = Some(value as u16),
            "latency_ms" => self.data.latency_ms = value,
            _ => {}
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.record_u64(field, value as u64);
    }

    fn record_i128(&mut self, field: &Field, value: i128) {
        self.record_u64(field, value as u64);
    }

    fn record_u128(&mut self, field: &Field, value: u128) {
        self.record_u64(field, value as u64);
    }

    fn record_bool(&mut self, _field: &Field, _value: bool) {}
    fn record_f64(&mut self, _field: &Field, _value: f64) {}

    fn record_debug(&mut self, _field: &Field, _value: &dyn std::fmt::Debug) {}
}
