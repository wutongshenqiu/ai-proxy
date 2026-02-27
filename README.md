# ai-proxy

Multi-provider AI API gateway written in Rust. Routes requests across Claude, OpenAI, Gemini, and any OpenAI-compatible provider with automatic credential rotation, format translation, and streaming support.

## Features

- **Multi-provider**: Claude, OpenAI, Gemini, and OpenAI-compatible (DeepSeek, Groq, etc.)
- **Format translation**: Send OpenAI-format requests, get routed to any provider transparently
- **Credential rotation**: Round-robin or fill-first strategy across multiple API keys per provider
- **Streaming**: SSE passthrough with keepalive, bootstrap retry, and cross-format stream translation
- **Responses API**: Transparent Chat Completions ↔ OpenAI Responses API conversion via `wire-api: responses`
- **Retry & cooldown**: Automatic retry with exponential backoff, per-credential cooldowns for 429/5xx/network errors
- **Hot reload**: Config file watcher — update credentials without restart
- **TLS**: Optional HTTPS with rustls
- **Cloaking**: Request masquerading for Claude API compliance
- **Payload rules**: Per-model field overrides, defaults, and filters

## Quick Start

```bash
# Build
cargo build --release

# Configure
cp config.example.yaml config.yaml
# Edit config.yaml with your API keys

# Run
./target/release/ai-proxy --config config.yaml
```

The server starts on `http://0.0.0.0:8317` by default.

## Usage

### Chat Completions (OpenAI format)

```bash
curl http://localhost:8317/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-sonnet-4-5",
    "messages": [{"role": "user", "content": "Hello"}],
    "max_tokens": 100
  }'
```

### Claude Messages API (passthrough)

```bash
curl http://localhost:8317/v1/messages \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-sonnet-4-5",
    "messages": [{"role": "user", "content": "Hello"}],
    "max_tokens": 100
  }'
```

### Streaming

Add `"stream": true` to any request.

### Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/v1/models` | GET | List available models |
| `/v1/chat/completions` | POST | OpenAI Chat Completions (routes to any provider) |
| `/v1/messages` | POST | Claude Messages API (Claude-only) |
| `/v1/responses` | POST | OpenAI Responses API (OpenAI/compat only) |
| `/admin/config` | GET | Current config (redacted) |
| `/admin/metrics` | GET | Request metrics |
| `/admin/models` | GET | All models with provider info |

## Configuration

See [`config.example.yaml`](config.example.yaml) for all options. Key sections:

```yaml
# Server
host: "0.0.0.0"
port: 8317

# Routing strategy: round-robin | fill-first
routing:
  strategy: round-robin

# Retry
request-retry: 3

# Provider credentials (multiple keys per provider)
claude-api-key:
  - api-key: "sk-ant-..."
    models:
      - id: "claude-sonnet-4-5-20250929"
        alias: "claude-sonnet-4-5"

openai-api-key:
  - api-key: "sk-..."

gemini-api-key:
  - api-key: "..."

# OpenAI-compatible providers
openai-compatibility:
  - api-key: "..."
    base-url: "https://api.deepseek.com"
    prefix: "deepseek/"
    models:
      - id: "deepseek-chat"
```

### Per-key Options

| Option | Description |
|--------|-------------|
| `api-key` | API key (required) |
| `base-url` | Custom API endpoint |
| `proxy-url` | Per-key proxy (overrides global; `""` = direct) |
| `prefix` | Model name prefix for routing (e.g., `"teamA/"`) |
| `models` | Available models with optional aliases |
| `excluded-models` | Models to exclude (supports `*` glob) |
| `headers` | Custom HTTP headers |
| `wire-api` | `chat` (default) or `responses` (OpenAI Responses API) |
| `disabled` | Disable this credential |

### Environment Variables

Override config values via environment or `.env` file:

```
AI_PROXY_CONFIG=config.yaml
AI_PROXY_HOST=0.0.0.0
AI_PROXY_PORT=8317
AI_PROXY_LOG_LEVEL=info
RUST_LOG=ai_proxy=debug
```

## Architecture

```
ai-proxy (binary)
├── ai-proxy-core       # Config, provider traits, error types, cloaking, metrics
├── ai-proxy-provider   # Claude/Gemini/OpenAI executors, credential routing
├── ai-proxy-translator # Cross-format request/response translation
└── ai-proxy-server     # Axum HTTP server, dispatch with retry, SSE streaming
```

## Requirements

- Rust 1.85+ (Edition 2024)

## License

MIT
