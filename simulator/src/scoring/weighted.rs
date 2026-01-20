use serde::{Deserialize, Serialize};

use crate::ActionProbs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionWeights {
    pub favorite: f64,
    pub reply: f64,
    pub repost: f64,
    pub photo_expand: f64,
    pub click: f64,
    pub profile_click: f64,
    pub vqv: f64,
    pub share: f64,
    pub share_dm: f64,
    pub share_link: f64,
    pub dwell: f64,
    pub quote: f64,
    pub quoted_click: f64,
    pub follow_author: f64,
    pub not_interested: f64,
    pub block: f64,
    pub mute: f64,
    pub report: f64,
    pub dwell_time: f64,
}

impl Default for ActionWeights {
    fn default() -> Self {
        Self {
            favorite: 1.0,
            reply: 1.6,
            repost: 2.0,
            photo_expand: 0.3,
            click: 0.4,
            profile_click: 0.3,
            vqv: 0.5,
            share: 1.4,
            share_dm: 0.8,
            share_link: 0.6,
            dwell: 0.2,
            quote: 1.7,
            quoted_click: 0.5,
            follow_author: 1.2,
            not_interested: -2.5,
            block: -5.0,
            mute: -3.0,
            report: -6.0,
            dwell_time: 0.1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WeightedScorer {
    weights: ActionWeights,
    vqv_duration_threshold: f64,
    score_offset: f64,
}

impl WeightedScorer {
    pub fn new(weights: ActionWeights, vqv_duration_threshold: f64, score_offset: f64) -> Self {
        Self {
            weights,
            vqv_duration_threshold,
            score_offset,
        }
    }

    pub fn score(&self, actions: &ActionProbs, video_duration: Option<f64>) -> f64 {
        let mut score = 0.0;

        score += actions.like * self.weights.favorite;
        score += actions.reply * self.weights.reply;
        score += actions.repost * self.weights.repost;
        score += actions.photo_expand * self.weights.photo_expand;
        score += actions.click * self.weights.click;
        score += actions.profile_click * self.weights.profile_click;
        score += actions.share * self.weights.share;
        score += actions.share_dm * self.weights.share_dm;
        score += actions.share_link * self.weights.share_link;
        score += actions.dwell * self.weights.dwell;
        score += actions.quote * self.weights.quote;
        score += actions.quoted_click * self.weights.quoted_click;
        score += actions.follow_author * self.weights.follow_author;

        if let Some(duration) = video_duration {
            if duration >= self.vqv_duration_threshold {
                score += actions.video_view * self.weights.vqv;
            }
        }

        score += actions.not_interested * self.weights.not_interested;
        score += actions.block * self.weights.block;
        score += actions.mute * self.weights.mute;
        score += actions.report * self.weights.report;

        score += actions.dwell_time * self.weights.dwell_time;

        if score < 0.0 {
            score = self.offset_score(score);
        }

        score
    }

    fn offset_score(&self, score: f64) -> f64 {
        score + self.score_offset
    }
}
