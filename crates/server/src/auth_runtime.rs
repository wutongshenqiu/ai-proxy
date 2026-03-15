use axum::http::StatusCode;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{Duration, Utc};
use prism_core::auth_profile::SharedOAuthTokenState;
use prism_core::error::ProxyError;
use prism_core::provider::AuthRecord;
use serde::Deserialize;
use sha2::{Digest, Sha256};

const CODEX_AUTH_URL: &str = "https://auth.openai.com/oauth/authorize";
const CODEX_TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
const CODEX_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";

#[derive(Debug, Clone)]
pub struct PendingCodexOauthSession {
    pub provider: String,
    pub profile_id: String,
    pub code_verifier: String,
    pub redirect_uri: String,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CodexOAuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: Option<String>,
    pub expires_at: Option<chrono::DateTime<Utc>>,
    pub account_id: Option<String>,
    pub email: Option<String>,
    pub last_refresh: chrono::DateTime<Utc>,
}

#[derive(Default)]
pub struct AuthRuntimeManager {
    refresh_skew_seconds: i64,
    codex_auth_url: String,
    codex_token_url: String,
    codex_client_id: String,
}

impl AuthRuntimeManager {
    pub fn new() -> Self {
        let codex_auth_url =
            std::env::var("PRISM_CODEX_AUTH_URL").unwrap_or_else(|_| CODEX_AUTH_URL.to_string());
        let codex_token_url =
            std::env::var("PRISM_CODEX_TOKEN_URL").unwrap_or_else(|_| CODEX_TOKEN_URL.to_string());
        let codex_client_id =
            std::env::var("PRISM_CODEX_CLIENT_ID").unwrap_or_else(|_| CODEX_CLIENT_ID.to_string());
        Self {
            refresh_skew_seconds: 120,
            codex_auth_url,
            codex_token_url,
            codex_client_id,
        }
    }

    pub fn with_codex_endpoints(auth_url: String, token_url: String, client_id: String) -> Self {
        Self {
            refresh_skew_seconds: 120,
            codex_auth_url: auth_url,
            codex_token_url: token_url,
            codex_client_id: client_id,
        }
    }

    pub async fn prepare_auth(
        &self,
        state: &crate::AppState,
        auth: &AuthRecord,
    ) -> Result<(), ProxyError> {
        if auth.auth_mode != prism_core::auth_profile::AuthMode::OpenaiCodexOauth {
            return Ok(());
        }
        let Some(shared) = auth.oauth_state.clone() else {
            return Ok(());
        };
        let should_refresh = {
            let guard = shared
                .read()
                .map_err(|e| ProxyError::Internal(format!("oauth state lock poisoned: {e}")))?;
            guard.refresh_token.is_empty() || !guard.expires_soon(self.refresh_skew_seconds)
        };
        if should_refresh {
            return Ok(());
        }

        let global_proxy = state.config.load().proxy_url.clone();
        let refreshed = self
            .refresh_codex_tokens(
                &state.http_client_pool,
                global_proxy.as_deref(),
                shared.clone(),
            )
            .await?;
        {
            let mut guard = shared
                .write()
                .map_err(|e| ProxyError::Internal(format!("oauth state lock poisoned: {e}")))?;
            guard.access_token = refreshed.access_token;
            guard.refresh_token = refreshed.refresh_token;
            guard.id_token = refreshed.id_token;
            guard.expires_at = refreshed.expires_at;
            guard.account_id = refreshed.account_id;
            guard.email = refreshed.email;
            guard.last_refresh = Some(refreshed.last_refresh);
        }
        Ok(())
    }

    pub fn generate_pkce() -> Result<(String, String), ProxyError> {
        let random: [u8; 96] = rand::random();
        let verifier = URL_SAFE_NO_PAD.encode(random);
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
        Ok((verifier, challenge))
    }

    pub fn build_codex_auth_url(&self, state: &str, challenge: &str, redirect_uri: &str) -> String {
        let params = [
            ("client_id", self.codex_client_id.as_str()),
            ("response_type", "code"),
            ("redirect_uri", redirect_uri),
            ("scope", "openid email profile offline_access"),
            ("state", state),
            ("code_challenge", challenge),
            ("code_challenge_method", "S256"),
            ("prompt", "login"),
            ("id_token_add_organizations", "true"),
            ("codex_cli_simplified_flow", "true"),
        ];
        let query = params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        format!("{}?{query}", self.codex_auth_url)
    }

    pub async fn exchange_codex_code(
        &self,
        pool: &prism_core::proxy::HttpClientPool,
        global_proxy: Option<&str>,
        code: &str,
        redirect_uri: &str,
        code_verifier: &str,
    ) -> Result<CodexOAuthTokens, String> {
        let client = pool
            .get_or_create(None, global_proxy, 30, 30)
            .map_err(|e| format!("failed to build oauth client: {e}"))?;
        self.exchange_codex_form(
            &client,
            &[
                ("grant_type", "authorization_code"),
                ("client_id", self.codex_client_id.as_str()),
                ("code", code),
                ("redirect_uri", redirect_uri),
                ("code_verifier", code_verifier),
            ],
        )
        .await
    }

    pub async fn refresh_codex_tokens(
        &self,
        pool: &prism_core::proxy::HttpClientPool,
        global_proxy: Option<&str>,
        shared: SharedOAuthTokenState,
    ) -> Result<CodexOAuthTokens, ProxyError> {
        let refresh_token = {
            let guard = shared
                .read()
                .map_err(|e| ProxyError::Internal(format!("oauth state lock poisoned: {e}")))?;
            guard.refresh_token.clone()
        };
        if refresh_token.is_empty() {
            return Err(ProxyError::Auth(
                "codex oauth profile missing refresh token".to_string(),
            ));
        }

        let client = pool
            .get_or_create(None, global_proxy, 30, 30)
            .map_err(|e| ProxyError::Internal(format!("failed to build oauth client: {e}")))?;
        self.exchange_codex_form(
            &client,
            &[
                ("client_id", self.codex_client_id.as_str()),
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token.as_str()),
                ("scope", "openid profile email"),
            ],
        )
        .await
        .map_err(ProxyError::Auth)
    }

    async fn exchange_codex_form(
        &self,
        client: &reqwest::Client,
        params: &[(&str, &str)],
    ) -> Result<CodexOAuthTokens, String> {
        #[derive(Deserialize)]
        struct TokenResp {
            access_token: String,
            refresh_token: Option<String>,
            id_token: Option<String>,
            expires_in: Option<i64>,
        }

        let form = params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let resp = client
            .post(&self.codex_token_url)
            .header("content-type", "application/x-www-form-urlencoded")
            .header("accept", "application/json")
            .body(form)
            .send()
            .await
            .map_err(|e| format!("oauth request failed: {e}"))?;
        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| format!("oauth read failed: {e}"))?;
        if status != StatusCode::OK {
            return Err(format!(
                "oauth exchange failed with status {}: {}",
                status, body
            ));
        }
        let token: TokenResp =
            serde_json::from_str(&body).map_err(|e| format!("invalid oauth response: {e}"))?;
        let (account_id, email) = token
            .id_token
            .as_deref()
            .and_then(parse_id_token_claims)
            .unwrap_or_default();
        Ok(CodexOAuthTokens {
            access_token: token.access_token,
            refresh_token: token.refresh_token.unwrap_or_default(),
            id_token: token.id_token,
            expires_at: token
                .expires_in
                .map(|secs| Utc::now() + Duration::seconds(secs)),
            account_id,
            email,
            last_refresh: Utc::now(),
        })
    }
}

fn parse_id_token_claims(id_token: &str) -> Option<(Option<String>, Option<String>)> {
    #[derive(Deserialize)]
    struct Claims {
        email: Option<String>,
        account_id: Option<String>,
        sub: Option<String>,
    }

    let payload = id_token.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD.decode(payload).ok()?;
    let claims: Claims = serde_json::from_slice(&bytes).ok()?;
    Some((claims.account_id.or(claims.sub), claims.email))
}
