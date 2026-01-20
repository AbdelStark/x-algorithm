use serde::{Deserialize, Serialize};

use crate::config::ScoringConfig;
use crate::{simulate_with_mode, MediaType, ScoringMode, SimulatorInput};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationSample {
    pub post_id: String,
    pub post_text: String,
    pub author_followers: u64,
    pub author_following: Option<u64>,
    pub account_age_days: Option<u32>,
    pub avg_engagement_rate: Option<f64>,
    pub posts_per_day: Option<f64>,
    pub verified: Option<bool>,
    pub media_type: String,
    pub actual_impressions: u64,
    pub actual_likes: u64,
    pub actual_replies: u64,
    pub actual_reposts: u64,
    pub actual_quotes: Option<u64>,
    pub actual_shares: Option<u64>,
}

impl CalibrationSample {
    pub fn to_input(&self) -> SimulatorInput {
        let mut input = SimulatorInput::default();
        input.text = self.post_text.clone();
        input.followers = self.author_followers;
        if let Some(following) = self.author_following {
            input.following = following;
        }
        if let Some(account_age_days) = self.account_age_days {
            input.account_age_days = account_age_days;
        }
        if let Some(avg_engagement_rate) = self.avg_engagement_rate {
            input.avg_engagement_rate = avg_engagement_rate;
        }
        if let Some(posts_per_day) = self.posts_per_day {
            input.posts_per_day = posts_per_day;
        }
        if let Some(verified) = self.verified {
            input.verified = verified;
        }
        if let Some(media) = MediaType::from_str(&self.media_type) {
            input.media = media;
        }
        input
    }

    pub fn engagement_rate(&self) -> f64 {
        let impressions = self.actual_impressions as f64;
        if impressions <= 0.0 {
            return 0.0;
        }
        let mut total = self.actual_likes + self.actual_replies + self.actual_reposts;
        if let Some(quotes) = self.actual_quotes {
            total += quotes;
        }
        if let Some(shares) = self.actual_shares {
            total += shares;
        }
        total as f64 / impressions
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CalibrationMetrics {
    pub impression_correlation: f64,
    pub engagement_rate_correlation: f64,
    pub like_rate_mae: f64,
    pub reply_rate_mae: f64,
    pub repost_rate_mae: f64,
    pub pairwise_ranking_accuracy: f64,
    pub sample_count: usize,
}

pub struct CalibrationRunner {
    pub samples: Vec<CalibrationSample>,
}

impl CalibrationRunner {
    pub fn new(samples: Vec<CalibrationSample>) -> Self {
        Self { samples }
    }

    pub fn compute_metrics(&self, config: &ScoringConfig) -> CalibrationMetrics {
        if self.samples.is_empty() {
            return CalibrationMetrics::default();
        }

        let mut impression_pairs = Vec::new();
        let mut engagement_pairs = Vec::new();
        let mut like_errors = Vec::new();
        let mut reply_errors = Vec::new();
        let mut repost_errors = Vec::new();
        let mut predicted_scores = Vec::new();
        let mut actual_rates = Vec::new();

        for sample in &self.samples {
            let input = sample.to_input();
            let output = simulate_with_mode(
                &input,
                None,
                None,
                ScoringMode::Heuristic,
                None,
                config,
            );

            impression_pairs.push((output.impressions_total, sample.actual_impressions as f64));

            let actual_engagement = sample.engagement_rate();
            engagement_pairs.push((output.unique_engagement_rate, actual_engagement));

            let impressions = sample.actual_impressions as f64;
            if impressions > 0.0 {
                let actual_like_rate = sample.actual_likes as f64 / impressions;
                let actual_reply_rate = sample.actual_replies as f64 / impressions;
                let actual_repost_rate = sample.actual_reposts as f64 / impressions;

                like_errors.push((output.actions.like - actual_like_rate).abs());
                reply_errors.push((output.actions.reply - actual_reply_rate).abs());
                repost_errors.push((output.actions.repost - actual_repost_rate).abs());
            }

            predicted_scores.push(output.final_score);
            actual_rates.push(actual_engagement);
        }

        CalibrationMetrics {
            impression_correlation: correlation(&impression_pairs),
            engagement_rate_correlation: correlation(&engagement_pairs),
            like_rate_mae: mean(&like_errors),
            reply_rate_mae: mean(&reply_errors),
            repost_rate_mae: mean(&repost_errors),
            pairwise_ranking_accuracy: pairwise_accuracy(&predicted_scores, &actual_rates),
            sample_count: self.samples.len(),
        }
    }
}

fn correlation(pairs: &[(f64, f64)]) -> f64 {
    if pairs.len() < 2 {
        return 0.0;
    }

    let (xs, ys): (Vec<f64>, Vec<f64>) = pairs.iter().cloned().unzip();
    let mean_x = mean(&xs);
    let mean_y = mean(&ys);

    let mut numerator = 0.0;
    let mut denom_x = 0.0;
    let mut denom_y = 0.0;

    for (x, y) in xs.iter().zip(ys.iter()) {
        let dx = x - mean_x;
        let dy = y - mean_y;
        numerator += dx * dy;
        denom_x += dx * dx;
        denom_y += dy * dy;
    }

    if denom_x <= 0.0 || denom_y <= 0.0 {
        return 0.0;
    }

    numerator / (denom_x.sqrt() * denom_y.sqrt())
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

fn pairwise_accuracy(predicted: &[f64], actual: &[f64]) -> f64 {
    let mut correct = 0usize;
    let mut total = 0usize;

    for i in 0..predicted.len() {
        for j in (i + 1)..predicted.len() {
            let pred_order = predicted[i].partial_cmp(&predicted[j]);
            let actual_order = actual[i].partial_cmp(&actual[j]);
            if let (Some(pred), Some(act)) = (pred_order, actual_order) {
                if pred == act {
                    correct += 1;
                }
                total += 1;
            }
        }
    }

    if total == 0 {
        0.0
    } else {
        correct as f64 / total as f64
    }
}
