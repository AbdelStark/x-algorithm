use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct XApiClient {
    client: reqwest::Client,
    api_base: String,
    auth: XApiAuth,
}

#[derive(Clone)]
enum XApiAuth {
    Bearer(String),
    OAuthClientCredentials {
        client_id: String,
        client_secret: String,
        token_url: String,
        scope: Option<String>,
        auth_mode: OAuthAuthMode,
        client_type: Option<String>,
        token_cache: Arc<Mutex<Option<OAuthTokenCache>>>,
    },
}

#[derive(Clone)]
struct OAuthTokenCache {
    access_token: String,
    expires_at: Instant,
}

#[derive(Clone, Copy)]
enum OAuthAuthMode {
    Basic,
    Body,
}

impl XApiClient {
    pub fn from_env() -> Option<Self> {
        let api_base = env::var("X_API_BASE").unwrap_or_else(|_| "https://api.twitter.com/2".to_string());
        let client = reqwest::Client::new();

        if let Ok(bearer_token) = env::var("X_API_BEARER_TOKEN") {
            return Some(Self {
                client,
                api_base,
                auth: XApiAuth::Bearer(decode_bearer(bearer_token)),
            });
        }

        if let (Ok(client_id), Ok(client_secret)) = (
            env::var("X_OAUTH_CLIENT_ID"),
            env::var("X_OAUTH_CLIENT_SECRET"),
        ) {
            let token_url = env::var("X_OAUTH_TOKEN_URL")
                .unwrap_or_else(|_| "https://api.twitter.com/2/oauth2/token".to_string());
            let scope = env::var("X_OAUTH_SCOPE").ok().filter(|value| !value.trim().is_empty());
            let auth_mode = match env::var("X_OAUTH_AUTH_MODE")
                .unwrap_or_else(|_| "basic".to_string())
                .to_lowercase()
                .as_str()
            {
                "basic" => OAuthAuthMode::Basic,
                _ => OAuthAuthMode::Body,
            };
            let client_type = env::var("X_OAUTH_CLIENT_TYPE")
                .ok()
                .filter(|value| !value.trim().is_empty());
            return Some(Self {
                client,
                api_base,
                auth: XApiAuth::OAuthClientCredentials {
                    client_id,
                    client_secret,
                    token_url,
                    scope,
                    auth_mode,
                    client_type,
                    token_cache: Arc::new(Mutex::new(None)),
                },
            });
        }

        None
    }

    pub async fn fetch_user_by_username(&self, username: &str) -> Result<XUserProfile, String> {
        let token = self.bearer_token().await?;
        self.fetch_user_by_username_with_token(username, &token).await
    }

    pub async fn fetch_user_by_username_with_token(
        &self,
        username: &str,
        token: &str,
    ) -> Result<XUserProfile, String> {
        let response = self
            .client
            .get(format!(
                "{}/users/by/username/{}",
                self.api_base.trim_end_matches('/'),
                username
            ))
            .query(&[("user.fields", "public_metrics,created_at,verified,protected")])
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await
            .map_err(|err| format!("X API request failed: {}", err))?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| String::new());
            let detail = error_body.trim();
            if detail.is_empty() {
                return Err(format!("X API error: {}", status));
            }
            return Err(format!("X API error: {} {}", status, detail));
        }

        let body: XUserResponse = response
            .json()
            .await
            .map_err(|err| format!("X API response parse failed: {}", err))?;

        let user = body
            .data
            .ok_or_else(|| "X API response missing user data".to_string())?;

        Ok(XUserProfile::from(user))
    }

    pub async fn fetch_me_with_token(&self, token: &str) -> Result<XUserProfile, String> {
        let response = self
            .client
            .get(format!(
                "{}/users/me",
                self.api_base.trim_end_matches('/')
            ))
            .query(&[("user.fields", "public_metrics,created_at,verified,protected")])
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await
            .map_err(|err| format!("X API request failed: {}", err))?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| String::new());
            let detail = error_body.trim();
            if detail.is_empty() {
                return Err(format!("X API error: {}", status));
            }
            return Err(format!("X API error: {} {}", status, detail));
        }

        let body: XUserResponse = response
            .json()
            .await
            .map_err(|err| format!("X API response parse failed: {}", err))?;

        let user = body
            .data
            .ok_or_else(|| "X API response missing user data".to_string())?;

        Ok(XUserProfile::from(user))
    }

    async fn bearer_token(&self) -> Result<String, String> {
        match &self.auth {
            XApiAuth::Bearer(token) => Ok(token.clone()),
            XApiAuth::OAuthClientCredentials {
                client_id,
                client_secret,
                token_url,
                scope,
                auth_mode,
                client_type,
                token_cache,
            } => {
                let now = Instant::now();
                {
                    let guard = token_cache.lock().await;
                    if let Some(cache) = guard.as_ref() {
                        if now < cache.expires_at {
                            return Ok(cache.access_token.clone());
                        }
                    }
                }

                let token = self
                    .fetch_oauth_token(
                        client_id,
                        client_secret,
                        token_url,
                        scope.as_deref(),
                        *auth_mode,
                        client_type.as_deref(),
                    )
                    .await?;
                let mut guard = token_cache.lock().await;
                *guard = Some(token.clone());
                Ok(token.access_token)
            }
        }
    }

    async fn fetch_oauth_token(
        &self,
        client_id: &str,
        client_secret: &str,
        token_url: &str,
        scope: Option<&str>,
        auth_mode: OAuthAuthMode,
        client_type: Option<&str>,
    ) -> Result<OAuthTokenCache, String> {
        let mut params = vec![("grant_type".to_string(), "client_credentials".to_string())];
        params.push(("client_id".to_string(), client_id.to_string()));
        params.push(("client_secret".to_string(), client_secret.to_string()));
        if let Some(value) = client_type {
            params.push(("client_type".to_string(), value.to_string()));
        }
        if let Some(scope_value) = scope {
            params.push(("scope".to_string(), scope_value.to_string()));
        }

        let mut request = self.client.post(token_url);
        if matches!(auth_mode, OAuthAuthMode::Basic) {
            request = request.basic_auth(client_id, Some(client_secret));
        }

        let response = request
            .form(&params)
            .send()
            .await
            .map_err(|err| format!("X OAuth token request failed: {}", err))?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| String::new());
            let detail = error_body.trim();
            if detail.is_empty() {
                return Err(format!("X OAuth token error: {}", status));
            }
            return Err(format!("X OAuth token error: {} {}", status, detail));
        }

        let body: OAuthTokenResponse = response
            .json()
            .await
            .map_err(|err| format!("X OAuth token parse failed: {}", err))?;

        let expires_in = body.expires_in.unwrap_or(3600);
        let expires_at = Instant::now() + Duration::from_secs(expires_in.saturating_sub(30));
        Ok(OAuthTokenCache {
            access_token: body.access_token,
            expires_at,
        })
    }
}

fn decode_bearer(value: String) -> String {
    if value.contains('%') {
        match urlencoding::decode(&value) {
            Ok(decoded) => decoded.into_owned(),
            Err(_) => value,
        }
    } else {
        value
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct XUserProfile {
    pub id: String,
    pub username: String,
    pub name: String,
    pub created_at: Option<String>,
    pub verified: Option<bool>,
    pub protected: Option<bool>,
    pub followers: u64,
    pub following: u64,
}

impl From<XUser> for XUserProfile {
    fn from(user: XUser) -> Self {
        Self {
            id: user.id,
            username: user.username,
            name: user.name,
            created_at: user.created_at,
            verified: user.verified,
            protected: user.protected,
            followers: user
                .public_metrics
                .as_ref()
                .map(|metrics| metrics.followers_count)
                .unwrap_or(0),
            following: user
                .public_metrics
                .as_ref()
                .map(|metrics| metrics.following_count)
                .unwrap_or(0),
        }
    }
}

#[derive(Deserialize)]
struct XUserResponse {
    data: Option<XUser>,
}

#[derive(Deserialize)]
struct XUser {
    id: String,
    username: String,
    name: String,
    created_at: Option<String>,
    verified: Option<bool>,
    protected: Option<bool>,
    public_metrics: Option<XPublicMetrics>,
}

#[derive(Deserialize)]
struct XPublicMetrics {
    followers_count: u64,
    following_count: u64,
}

#[derive(Deserialize)]
struct OAuthTokenResponse {
    access_token: String,
    expires_in: Option<u64>,
}
