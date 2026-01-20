use std::cmp::Ordering;

use crate::scoring::{AuthorDiversityScorer, OonScorer, WeightedScorer};
use crate::ActionProbs;

#[derive(Debug, Clone)]
pub struct ScoredCandidate {
    pub post_id: String,
    pub author_id: String,
    pub is_oon: bool,
    pub video_duration: Option<f64>,
    pub phoenix_scores: ActionProbs,
    pub weighted_score: f64,
    pub diversity_multiplier: f64,
    pub oon_multiplier: f64,
    pub score: f64,
}

impl ScoredCandidate {
    pub fn new(
        post_id: String,
        author_id: String,
        is_oon: bool,
        video_duration: Option<f64>,
        phoenix_scores: ActionProbs,
    ) -> Self {
        Self {
            post_id,
            author_id,
            is_oon,
            video_duration,
            phoenix_scores,
            weighted_score: 0.0,
            diversity_multiplier: 1.0,
            oon_multiplier: 1.0,
            score: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScoringPipeline {
    weighted_scorer: WeightedScorer,
    diversity_scorer: AuthorDiversityScorer,
    oon_scorer: OonScorer,
}

impl ScoringPipeline {
    pub fn new(
        weighted_scorer: WeightedScorer,
        diversity_scorer: AuthorDiversityScorer,
        oon_scorer: OonScorer,
    ) -> Self {
        Self {
            weighted_scorer,
            diversity_scorer,
            oon_scorer,
        }
    }

    pub fn score(&self, candidates: &mut [ScoredCandidate]) {
        for candidate in candidates.iter_mut() {
            candidate.weighted_score = self.weighted_scorer.score(
                &candidate.phoenix_scores,
                candidate.video_duration,
            );
            candidate.score = candidate.weighted_score;
            candidate.diversity_multiplier = 1.0;
            candidate.oon_multiplier = 1.0;
        }

        candidates.sort_by(|a, b| {
            b.weighted_score
                .partial_cmp(&a.weighted_score)
                .unwrap_or(Ordering::Equal)
        });

        self.diversity_scorer.score(candidates);

        for candidate in candidates.iter_mut() {
            self.oon_scorer.score(candidate, candidate.is_oon);
        }

        candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
    }
}
