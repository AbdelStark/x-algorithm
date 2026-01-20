use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Clone)]
pub struct XApiClient {
    client: reqwest::Client,
    bearer_token: String,
    api_base: String,
}

impl XApiClient {
    pub fn from_env() -> Option<Self> {
        let bearer_token = env::var("X_API_BEARER_TOKEN").ok()?;
        let api_base = env::var("X_API_BASE").unwrap_or_else(|_| "https://api.twitter.com/2".to_string());
        let client = reqwest::Client::new();
        Some(Self {
            client,
            bearer_token: decode_bearer(bearer_token),
            api_base,
        })
    }

    pub async fn fetch_user_by_username(&self, username: &str) -> Result<XUserProfile, String> {
        let url = format!(
            "{}/users/by/username/{}",
            self.api_base.trim_end_matches('/'),
            username
        );
        let response = self
            .client
            .get(url)
            .query(&[("user.fields", "public_metrics,created_at,verified,protected")])
            .header(AUTHORIZATION, format!("Bearer {}", self.bearer_token))
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
