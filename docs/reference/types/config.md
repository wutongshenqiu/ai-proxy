# Configuration Types Reference

All configuration types used for YAML config parsing and runtime settings.

**Source:** `crates/core/src/config.rs`, `crates/core/src/payload.rs`, `crates/core/src/cloak.rs`

---

## Config

The root configuration struct. Loaded from YAML via `Config::load()`. Uses `#[serde(rename_all = "kebab-case", default)]`.

```rust
pub struct Config {
    pub host: String,
    pub port: u16,
    pub tls: TlsConfig,
    pub api_keys: Vec<String>,
    #[serde(skip)]
    pub api_keys_set: HashSet<String>,    // built from api_keys during sanitize()
    pub proxy_url: Option<String>,
    pub debug: bool,
    pub logging_to_file: bool,
    pub log_dir: Option<String>,
    pub routing: RoutingConfig,
    pub request_retry: u32,
    pub max_retry_interval: u64,
    pub connect_timeout: u64,
    pub request_timeout: u64,
    pub streaming: StreamingConfig,
    pub body_limit_mb: usize,
    pub retry: RetryConfig,
    pub payload: PayloadConfig,
    pub passthrough_headers: Vec<String>,
    pub claude_header_defaults: HashMap<String, String>,
    pub force_model_prefix: bool,
    pub non_stream_keepalive_secs: u64,
    pub claude_api_key: Vec<ProviderKeyEntry>,
    pub openai_api_key: Vec<ProviderKeyEntry>,
    pub gemini_api_key: Vec<ProviderKeyEntry>,
    pub openai_compatibility: Vec<ProviderKeyEntry>,
}
```

### Field defaults

| Field | Type | Default | YAML key |
|-------|------|---------|----------|
| `host` | `String` | `"0.0.0.0"` | `host` |
| `port` | `u16` | `8317` | `port` |
| `tls` | `TlsConfig` | disabled | `tls` |
| `api_keys` | `Vec<String>` | `[]` | `api-keys` |
| `proxy_url` | `Option<String>` | `None` | `proxy-url` |
| `debug` | `bool` | `false` | `debug` |
| `logging_to_file` | `bool` | `false` | `logging-to-file` |
| `log_dir` | `Option<String>` | `None` | `log-dir` |
| `routing` | `RoutingConfig` | round-robin | `routing` |
| `request_retry` | `u32` | `3` | `request-retry` |
| `max_retry_interval` | `u64` | `30` | `max-retry-interval` |
| `connect_timeout` | `u64` | `30` | `connect-timeout` |
| `request_timeout` | `u64` | `300` | `request-timeout` |
| `streaming` | `StreamingConfig` | see below | `streaming` |
| `body_limit_mb` | `usize` | `10` | `body-limit-mb` |
| `retry` | `RetryConfig` | see below | `retry` |
| `payload` | `PayloadConfig` | empty | `payload` |
| `passthrough_headers` | `Vec<String>` | `[]` | `passthrough-headers` |
| `claude_header_defaults` | `HashMap<String, String>` | `{}` | `claude-header-defaults` |
| `force_model_prefix` | `bool` | `false` | `force-model-prefix` |
| `non_stream_keepalive_secs` | `u64` | `0` (disabled) | `non-stream-keepalive-secs` |
| `claude_api_key` | `Vec<ProviderKeyEntry>` | `[]` | `claude-api-key` |
| `openai_api_key` | `Vec<ProviderKeyEntry>` | `[]` | `openai-api-key` |
| `gemini_api_key` | `Vec<ProviderKeyEntry>` | `[]` | `gemini-api-key` |
| `openai_compatibility` | `Vec<ProviderKeyEntry>` | `[]` | `openai-compatibility` |

### Key methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `load` | `fn load(path: &str) -> Result<Self, anyhow::Error>` | Reads YAML, deserializes, sanitizes, and validates. |
| `all_provider_keys` | `fn all_provider_keys(&self) -> impl Iterator<Item = &ProviderKeyEntry>` | Iterates all provider key entries across all provider types. |

### Sanitization (on load)

- Entries with empty `api_key` are removed.
- Duplicate entries (by `api_key`) are deduplicated.
- Trailing slashes are stripped from `base_url`.
- Header keys are normalized to lowercase.
- `api_keys_set` is built from `api_keys` for O(1) lookups.

---

## TlsConfig

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case", default)]
pub struct TlsConfig {
    pub enable: bool,
    pub cert: Option<String>,
    pub key: Option<String>,
}
```

| Field | Type | Default | YAML key |
|-------|------|---------|----------|
| `enable` | `bool` | `false` | `enable` |
| `cert` | `Option<String>` | `None` | `cert` |
| `key` | `Option<String>` | `None` | `key` |

Validation: if `enable` is `true`, both `cert` and `key` must be set.

### YAML example

```yaml
tls:
  enable: true
  cert: /path/to/cert.pem
  key: /path/to/key.pem
```

---

## RoutingConfig

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct RoutingConfig {
    pub strategy: RoutingStrategy,
}
```

| Field | Type | Default | YAML key |
|-------|------|---------|----------|
| `strategy` | `RoutingStrategy` | `RoundRobin` | `strategy` |

### YAML example

```yaml
routing:
  strategy: round-robin
```

---

## StreamingConfig

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct StreamingConfig {
    pub keepalive_seconds: u64,
    pub bootstrap_retries: u32,
}
```

| Field | Type | Default | YAML key | Description |
|-------|------|---------|----------|-------------|
| `keepalive_seconds` | `u64` | `15` | `keepalive-seconds` | SSE keepalive interval during streaming. |
| `bootstrap_retries` | `u32` | `1` | `bootstrap-retries` | Max retries before first byte is sent to client. |

### YAML example

```yaml
streaming:
  keepalive-seconds: 15
  bootstrap-retries: 1
```

---

## RetryConfig

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub max_backoff_secs: u64,
    pub cooldown_429_secs: u64,
    pub cooldown_5xx_secs: u64,
    pub cooldown_network_secs: u64,
}
```

| Field | Type | Default | YAML key | Description |
|-------|------|---------|----------|-------------|
| `max_retries` | `u32` | `3` | `max-retries` | Maximum retry attempts across all providers. |
| `max_backoff_secs` | `u64` | `30` | `max-backoff-secs` | Cap for exponential backoff with jitter. |
| `cooldown_429_secs` | `u64` | `60` | `cooldown-429-secs` | Cooldown duration for rate-limited (429) credentials. Overridden by `Retry-After` header when present. |
| `cooldown_5xx_secs` | `u64` | `15` | `cooldown-5xx-secs` | Cooldown duration for 5xx errors. Overridden by `Retry-After` header when present. |
| `cooldown_network_secs` | `u64` | `10` | `cooldown-network-secs` | Cooldown duration for network errors (timeout, connection failure). |

### YAML example

```yaml
retry:
  max-retries: 3
  max-backoff-secs: 30
  cooldown-429-secs: 60
  cooldown-5xx-secs: 15
  cooldown-network-secs: 10
```

---

## PayloadConfig

**Source:** `crates/core/src/payload.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PayloadConfig {
    #[serde(default)]
    pub default: Vec<PayloadRule>,
    #[serde(default)]
    pub r#override: Vec<PayloadRule>,
    #[serde(default)]
    pub filter: Vec<FilterRule>,
}
```

| Field | Type | Default | YAML key | Behavior |
|-------|------|---------|----------|----------|
| `default` | `Vec<PayloadRule>` | `[]` | `default` | Set values only if the field is missing from the request. |
| `override` | `Vec<PayloadRule>` | `[]` | `override` | Always set values, overwriting existing ones. |
| `filter` | `Vec<FilterRule>` | `[]` | `filter` | Remove fields from the request payload. |

Processing order: defaults -> overrides -> filters.

---

## PayloadRule

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PayloadRule {
    pub models: Vec<ModelMatcher>,
    pub params: serde_json::Map<String, Value>,
}
```

| Field | Type | YAML key | Description |
|-------|------|----------|-------------|
| `models` | `Vec<ModelMatcher>` | `models` | Model patterns to match (supports glob). |
| `params` | `Map<String, Value>` | `params` | Dot-separated paths to JSON values (e.g., `"reasoning.effort": "high"`). |

### ModelMatcher

```rust
pub struct ModelMatcher {
    pub name: String,
    pub protocol: Option<String>,
}
```

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Glob pattern for model name (e.g., `"gemini-*"`, `"*"`). |
| `protocol` | `Option<String>` | Optional target protocol filter (e.g., `"openai"`, `"claude"`). |

---

## FilterRule

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FilterRule {
    pub models: Vec<ModelMatcher>,
    pub params: Vec<String>,
}
```

| Field | Type | YAML key | Description |
|-------|------|----------|-------------|
| `models` | `Vec<ModelMatcher>` | `models` | Model patterns to match. |
| `params` | `Vec<String>` | `params` | Dot-separated paths to remove from payload (e.g., `"generationConfig.responseJsonSchema"`). |

### YAML example (payload section)

```yaml
payload:
  default:
    - models:
        - name: "gemini-*"
      params:
        generationConfig.thinkingConfig.thinkingBudget: 32768
  override:
    - models:
        - name: "gpt-*"
          protocol: openai
      params:
        reasoning.effort: "high"
  filter:
    - models:
        - name: "gemini-2.0-flash*"
      params:
        - generationConfig.responseJsonSchema
```

---

## ProviderKeyEntry

Per-credential configuration for a single API key.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProviderKeyEntry {
    pub api_key: String,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub proxy_url: Option<String>,
    #[serde(default)]
    pub prefix: Option<String>,
    #[serde(default)]
    pub models: Vec<ModelMapping>,
    #[serde(default)]
    pub excluded_models: Vec<String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub cloak: CloakConfig,
    #[serde(default)]
    pub wire_api: WireApi,
}
```

| Field | Type | Default | YAML key | Description |
|-------|------|---------|----------|-------------|
| `api_key` | `String` | required | `api-key` | Provider API key. Entries with empty keys are removed during sanitization. |
| `base_url` | `Option<String>` | `None` | `base-url` | Override provider base URL. Trailing slashes are stripped. |
| `proxy_url` | `Option<String>` | `None` | `proxy-url` | Per-credential proxy URL. Falls back to global `proxy_url`. |
| `prefix` | `Option<String>` | `None` | `prefix` | Model name prefix (e.g., `"openai/"`) for namespace isolation. |
| `models` | `Vec<ModelMapping>` | `[]` | `models` | Explicit model list. If empty, all models are accepted. |
| `excluded_models` | `Vec<String>` | `[]` | `excluded-models` | Glob patterns for models to exclude. |
| `headers` | `HashMap<String, String>` | `{}` | `headers` | Extra headers to inject on upstream requests. Keys normalized to lowercase. |
| `disabled` | `bool` | `false` | `disabled` | Disable this credential without removing it. |
| `name` | `Option<String>` | `None` | `name` | Human-readable name for logging/identification. |
| `cloak` | `CloakConfig` | `CloakMode::Never` | `cloak` | Claude cloaking configuration. Only used for Claude provider entries. |
| `wire_api` | `WireApi` | `Chat` | `wire-api` | Wire API format for OpenAI-compatible providers. |

### YAML example

```yaml
claude-api-key:
  - api-key: "sk-ant-xxx"
    base-url: "https://api.anthropic.com"
    prefix: "claude/"
    name: "primary-claude"
    models:
      - id: "claude-sonnet-4-20250514"
        alias: "sonnet"
    excluded-models:
      - "claude-2*"
    cloak:
      mode: auto
      strict-mode: false
      sensitive-words: ["secret"]
      cache-user-id: true
```

---

## ModelMapping

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ModelMapping {
    pub id: String,
    #[serde(default)]
    pub alias: Option<String>,
}
```

| Field | Type | Default | YAML key | Description |
|-------|------|---------|----------|-------------|
| `id` | `String` | required | `id` | Upstream model ID (supports glob patterns in matching). |
| `alias` | `Option<String>` | `None` | `alias` | Alternate name clients can use. Resolved to `id` before sending upstream. |

---

## CloakConfig

**Source:** `crates/core/src/cloak.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct CloakConfig {
    pub mode: CloakMode,
    pub strict_mode: bool,
    pub sensitive_words: Vec<String>,
    pub cache_user_id: bool,
}
```

| Field | Type | Default | YAML key | Description |
|-------|------|---------|----------|-------------|
| `mode` | `CloakMode` | `Never` | `mode` | When to apply cloaking. |
| `strict_mode` | `bool` | `false` | `strict-mode` | If true, replace user's system prompt entirely; if false, prepend cloak prompt. |
| `sensitive_words` | `Vec<String>` | `[]` | `sensitive-words` | Words to obfuscate by inserting zero-width spaces. |
| `cache_user_id` | `bool` | `false` | `cache-user-id` | Whether to cache the generated `user_id` per API key. |
