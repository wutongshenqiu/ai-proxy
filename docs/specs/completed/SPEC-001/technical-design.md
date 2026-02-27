# Technical Design: Multi-Provider Routing & Credential Management

| Field     | Value                                         |
|-----------|-----------------------------------------------|
| Spec ID   | SPEC-001                                      |
| Title     | Multi-Provider Routing & Credential Management |
| Author    | AI Proxy Team                                 |
| Status    | Completed                                     |
| Created   | 2025-01-01                                    |
| Updated   | 2025-01-01                                    |

## Overview

This spec covers the core routing and credential management layer that enables the proxy to dispatch requests to multiple AI providers. The system is built around three key abstractions: `Format` (provider type), `AuthRecord` (a resolved credential with model/routing metadata), and `CredentialRouter` (the routing engine that selects credentials). See SPEC-001 PRD for requirements.

## Module Structure

```
crates/core/src/
  provider.rs     # Format, AuthRecord, ModelEntry, ProviderExecutor trait
  config.rs       # ProviderKeyEntry, RoutingStrategy, RoutingConfig
crates/provider/src/
  routing.rs      # CredentialRouter
  lib.rs          # ExecutorRegistry, build_registry()
  openai.rs       # OpenAI executor (wraps OpenAICompatExecutor with OpenAI defaults)
  claude.rs       # Claude executor
  gemini.rs       # Gemini executor
  openai_compat.rs # OpenAI-compatible executor
```

## Key Types

### Format Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Format {
    OpenAI,
    Claude,
    Gemini,
    OpenAICompat,
}
```

Four variants representing each supported provider format. Implements `FromStr` (accepting `"openai"`, `"claude"`, `"gemini"`, `"openai-compat"` / `"openai_compat"`) and `Display`.

### AuthRecord

```rust
pub struct AuthRecord {
    pub id: String,                          // UUID, generated per config reload
    pub provider: Format,                     // Which provider format
    pub api_key: String,                      // The upstream API key
    pub base_url: Option<String>,            // Custom base URL override
    pub proxy_url: Option<String>,           // Per-credential HTTP proxy
    pub headers: HashMap<String, String>,    // Extra headers to send upstream
    pub models: Vec<ModelEntry>,             // Allowed models (empty = all)
    pub excluded_models: Vec<String>,        // Glob patterns to exclude
    pub prefix: Option<String>,              // Model name prefix (e.g., "team-a/")
    pub disabled: bool,                      // Administratively disabled
    pub cooldown_until: Option<Instant>,     // Temporary cooldown expiry
    pub cloak: Option<CloakConfig>,          // Claude-specific header cloaking
    pub wire_api: WireApi,                   // Chat vs Responses API wire format
}
```

Key methods on `AuthRecord`:

| Method | Behavior |
|--------|----------|
| `supports_model(model)` | Strips prefix, checks glob match against `models` list (empty list = accept all), then verifies not in `excluded_models` |
| `resolve_model_id(model)` | Strips prefix, resolves alias to real model ID. If the model matches an alias, returns the corresponding `ModelEntry.id` |
| `strip_prefix(model)` | Removes the credential's prefix from the model name. Returns original if prefix doesn't match |
| `prefixed_model_id(model_id)` | Prepends the credential's prefix to a model ID for display/routing |
| `is_model_excluded(model)` | Checks model against `excluded_models` using glob pattern matching |
| `is_available()` | Returns `false` if `disabled` or if `cooldown_until` is in the future |
| `base_url_or_default(default)` | Returns `base_url` (trimmed of trailing `/`) or the provided default |

### ModelEntry

```rust
pub struct ModelEntry {
    pub id: String,              // Real model ID (e.g., "claude-sonnet-4-20250514")
    pub alias: Option<String>,   // Optional alias (e.g., "sonnet")
}
```

### ProviderKeyEntry (Config)

```rust
pub struct ProviderKeyEntry {
    pub api_key: String,
    pub base_url: Option<String>,
    pub proxy_url: Option<String>,
    pub prefix: Option<String>,
    pub models: Vec<ModelMapping>,
    pub excluded_models: Vec<String>,
    pub headers: HashMap<String, String>,
    pub disabled: bool,
    pub name: Option<String>,          // Human-readable label for logging
    pub cloak: CloakConfig,
    pub wire_api: WireApi,
}
```

Config sanitization (`sanitize_entries`): removes entries with empty `api_key`, deduplicates by `api_key`, strips trailing `/` from `base_url`, lowercases header keys.

### RoutingStrategy

```rust
pub enum RoutingStrategy {
    RoundRobin,   // Rotate across available credentials
    FillFirst,    // Always pick the first available credential
}
```

Configured via `routing.strategy` in YAML config. Default: `RoundRobin`.

## CredentialRouter

The `CredentialRouter` is the central routing engine. It holds all credentials grouped by `Format` and selects the appropriate credential for each request.

```rust
pub struct CredentialRouter {
    credentials: RwLock<HashMap<Format, Vec<AuthRecord>>>,
    counters: RwLock<HashMap<String, AtomicUsize>>,
    strategy: RwLock<RoutingStrategy>,
}
```

### `pick(provider, model, tried)` Algorithm

1. Acquire read lock on `credentials`
2. Get the `Vec<AuthRecord>` for the given `Format`
3. Filter to candidates where: `is_available() == true`, `supports_model(model) == true`, and `id` not in `tried`
4. If no candidates remain, return `None`
5. Select based on strategy:
   - **FillFirst**: return the first candidate (index 0)
   - **RoundRobin**: compute key `"{provider}:{model}"`, fetch or create an `AtomicUsize` counter for that key, `fetch_add(1, Relaxed)`, pick `candidates[counter % candidates.len()]`

### `mark_unavailable(auth_id, duration)`

Sets `cooldown_until = Instant::now() + duration` on the matching `AuthRecord` (matched by `auth.id`). Subsequent calls to `is_available()` will return `false` until the cooldown expires.

### `update_from_config(config)`

Rebuilds credentials from config. For each provider array (`claude_api_key`, `openai_api_key`, `gemini_api_key`, `openai_compatibility`), converts `ProviderKeyEntry` to `AuthRecord` via `build_auth_record()`. Preserves `cooldown_until` from existing credentials matched by `api_key + format`. Also updates the routing strategy.

### `resolve_providers(model)`

Returns a `Vec<Format>` of all provider formats that have at least one available credential supporting the given model. Used for multi-provider model resolution.

### `all_models()`

Iterates all available credentials and collects `ModelInfo` entries (using alias if present, deduplicating by model ID). Used by the `/v1/models` endpoint.

### `model_has_prefix(model)`

Checks if any available credential with a prefix matches the given model. Used by `force_model_prefix` to reject unprefixed requests.

## ExecutorRegistry

```rust
pub struct ExecutorRegistry {
    executors: HashMap<String, Arc<dyn ProviderExecutor>>,
}
```

Built by `build_registry(global_proxy)`, which registers four executors:

| Key | Executor | Native Format | Default Base URL |
|-----|----------|---------------|------------------|
| `"openai"` | `OpenAICompatExecutor` (via `openai::new_openai_executor()`) | `Format::OpenAI` | `https://api.openai.com` |
| `"claude"` | `ClaudeExecutor` | `Format::Claude` | `https://api.anthropic.com` |
| `"gemini"` | `GeminiExecutor` | `Format::Gemini` | `https://generativelanguage.googleapis.com` |
| `"openai-compat"` | `OpenAICompatExecutor` | `Format::OpenAICompat` | (empty -- must be provided in config) |

Lookup methods:
- `get(name)` -- by string key
- `get_by_format(format)` -- finds first executor whose `native_format()` matches

### ProviderExecutor Trait

```rust
#[async_trait]
pub trait ProviderExecutor: Send + Sync {
    fn identifier(&self) -> &str;
    fn native_format(&self) -> Format;
    fn default_base_url(&self) -> &str;
    async fn execute(&self, auth: &AuthRecord, request: ProviderRequest) -> Result<ProviderResponse, ProxyError>;
    async fn execute_stream(&self, auth: &AuthRecord, request: ProviderRequest) -> Result<StreamResult, ProxyError>;
    fn supported_models(&self, auth: &AuthRecord) -> Vec<ModelInfo>;
}
```

## Configuration

```yaml
routing:
  strategy: round-robin  # or "fill-first"

force-model-prefix: false  # When true, reject requests without a model prefix

claude-api-key:
  - api-key: "sk-ant-xxx"
    base-url: "https://api.anthropic.com"
    prefix: "anthropic/"
    models:
      - id: "claude-sonnet-4-20250514"
        alias: "sonnet"
    excluded-models:
      - "*preview*"
    headers:
      x-custom: "value"
    disabled: false
    name: "primary-claude-key"

openai-api-key:
  - api-key: "sk-xxx"
    models:
      - id: "gpt-4o"

gemini-api-key:
  - api-key: "AIza..."
    models:
      - id: "gemini-2.0-flash"

openai-compatibility:
  - api-key: "sk-deepseek-xxx"
    base-url: "https://api.deepseek.com/v1"
    models:
      - id: "deepseek-chat"
```

## Provider Compatibility

| Provider | Supported | Notes |
|----------|-----------|-------|
| OpenAI   | Yes | Native OpenAI format; Chat and Responses wire APIs supported |
| Claude   | Yes | Anthropic Messages API; header cloaking supported via `cloak` config |
| Gemini   | Yes | Google AI `generateContent` / `streamGenerateContent` endpoints |
| OpenAI-compat | Yes | Any provider exposing an OpenAI-compatible API; requires `base-url` in config |

## Test Strategy

- **Unit tests:** `Config::default()` values, `sanitize_entries` behavior (dedup, empty key removal, URL normalization, header lowercasing), YAML deserialization round-trip
- **Integration tests:** `CredentialRouter::pick()` with round-robin and fill-first strategies, cooldown behavior, model alias resolution, prefix routing, excluded model patterns
