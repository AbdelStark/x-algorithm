use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::config::ScoringConfig;
use crate::ActionProbs;

#[derive(Clone)]
pub struct PhoenixClient {
    endpoint: String,
    client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize)]
pub struct PostFeatures {
    pub post_id: String,
    pub author_id: String,
    pub text_hash: u64,
    pub author_hash: u64,
    pub product_surface: i32,
    pub video_duration_seconds: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RankingRequest {
    pub user_id: String,
    pub user_embedding: Option<Vec<f32>>,
    pub history_posts: Vec<PostFeatures>,
    pub history_actions: Vec<Vec<f32>>,
    pub candidates: Vec<PostFeatures>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CandidateScore {
    pub post_id: String,
    pub phoenix_scores: ActionProbs,
    pub weighted_score: f64,
    pub rank: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RankingResponse {
    pub scores: Vec<CandidateScore>,
}

impl PhoenixClient {
    pub fn from_config(config: &ScoringConfig) -> Result<Self, String> {
        let timeout = Duration::from_millis(config.phoenix.timeout_ms);
        PhoenixClient::new(config.phoenix.endpoint.clone(), timeout)
    }

    pub fn new(endpoint: String, timeout: Duration) -> Result<Self, String> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|err| format!("failed to build phoenix client: {}", err))?;
        Ok(Self { endpoint, client })
    }

    pub async fn score(&self, request: RankingRequest) -> Result<RankingResponse, String> {
        let url = format!("{}/rank", self.endpoint.trim_end_matches('/'));
        let response = self
            .client
            .post(url)
            .json(&request)
            .send()
            .await
            .map_err(|err| format!("phoenix request failed: {}", err))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("phoenix error {}: {}", status, body));
        }

        response
            .json::<RankingResponse>()
            .await
            .map_err(|err| format!("phoenix response parse failed: {}", err))
    }
}
