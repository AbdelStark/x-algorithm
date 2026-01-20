use serde::{Deserialize, Serialize};
use virality_sim::{
    ActionProbs, LlmScore, LlmTrace, MediaType, Signals, SimulationOutput, SimulatorInput,
};

#[derive(Debug, Deserialize)]
pub struct ApiSimulationRequest {
    pub text: Option<String>,
    pub request_id: Option<String>,
    pub media: Option<String>,
    pub has_link: Option<bool>,
    pub followers: Option<u64>,
    pub following: Option<u64>,
    pub account_age_days: Option<u32>,
    pub avg_engagement_rate: Option<f64>,
    pub posts_per_day: Option<f64>,
    pub verified: Option<bool>,
    pub hour_of_day: Option<u8>,
    pub novelty: Option<f64>,
    pub timeliness: Option<f64>,
    pub topic_saturation: Option<f64>,
    pub audience_fit: Option<f64>,
    pub controversy: Option<f64>,
    pub sentiment: Option<f64>,
    pub use_ai: Option<bool>,
}

impl ApiSimulationRequest {
    pub fn into_input(self) -> Result<SimulatorInput, String> {
        let mut input = SimulatorInput::default();
        let text = self
            .text
            .unwrap_or_default()
            .trim()
            .to_string();
        if text.is_empty() {
            return Err("text is required".to_string());
        }
        input.text = text;

        if let Some(media) = self.media.as_deref() {
            input.media = MediaType::from_str(media)
                .ok_or_else(|| format!("invalid media type: {}", media))?;
        }

        if let Some(has_link) = self.has_link {
            input.has_link_override = Some(has_link);
        }

        if let Some(value) = self.followers {
            input.followers = value;
        }
        if let Some(value) = self.following {
            input.following = value;
        }
        if let Some(value) = self.account_age_days {
            input.account_age_days = value;
        }
        if let Some(value) = self.avg_engagement_rate {
            input.avg_engagement_rate = value;
        }
        if let Some(value) = self.posts_per_day {
            input.posts_per_day = value;
        }
        if let Some(value) = self.verified {
            input.verified = value;
        }
        if let Some(value) = self.hour_of_day {
            input.hour_of_day = value.min(23);
        }
        if let Some(value) = self.novelty {
            input.novelty = value;
        }
        if let Some(value) = self.timeliness {
            input.timeliness = value;
        }
        if let Some(value) = self.topic_saturation {
            input.topic_saturation = value;
        }
        if let Some(value) = self.audience_fit {
            input.audience_fit = value;
        }
        if let Some(value) = self.controversy {
            input.controversy = value;
        }
        if let Some(value) = self.sentiment {
            input.sentiment = value;
        }

        Ok(input)
    }
}

#[derive(Debug, Serialize)]
pub struct ApiSimulationResponse {
    pub request_id: String,
    pub score: f64,
    pub tier: String,
    pub weighted_score: f64,
    pub impressions_in: f64,
    pub impressions_oon: f64,
    pub impressions_total: f64,
    pub expected_unique_engagements: f64,
    pub expected_action_volume: f64,
    pub unique_engagement_rate: f64,
    pub action_volume_rate: f64,
    pub actions: ActionProbs,
    pub signals: Signals,
    pub suggestions: Vec<String>,
    pub llm: Option<LlmScore>,
    pub llm_trace: Option<LlmTrace>,
    pub warnings: Vec<String>,
}

impl ApiSimulationResponse {
    pub fn from_output(output: SimulationOutput, warnings: Vec<String>, request_id: String) -> Self {
        Self {
            request_id,
            score: output.score,
            tier: output.tier.label().to_string(),
            weighted_score: output.weighted_score,
            impressions_in: output.impressions_in,
            impressions_oon: output.impressions_oon,
            impressions_total: output.impressions_total,
            expected_unique_engagements: output.expected_unique_engagements,
            expected_action_volume: output.expected_action_volume,
            unique_engagement_rate: output.unique_engagement_rate,
            action_volume_rate: output.action_volume_rate,
            actions: output.actions,
            signals: output.signals,
            suggestions: output.suggestions,
            llm: output.llm,
            llm_trace: output.llm_trace,
            warnings,
        }
    }
}
