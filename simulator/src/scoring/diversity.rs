use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::scoring::ScoredCandidate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorDiversityConfig {
    pub decay: f64,
    pub floor: f64,
}

impl Default for AuthorDiversityConfig {
    fn default() -> Self {
        Self {
            decay: 0.7,
            floor: 0.1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthorDiversityScorer {
    config: AuthorDiversityConfig,
}

impl AuthorDiversityScorer {
    pub fn new(config: AuthorDiversityConfig) -> Self {
        Self { config }
    }

    pub fn multiplier(&self, occurrence: usize) -> f64 {
        let decay_factor = self.config.decay.powi(occurrence as i32);
        (1.0 - self.config.floor) * decay_factor + self.config.floor
    }

    pub fn score(&self, candidates: &mut [ScoredCandidate]) {
        let mut author_counts: HashMap<String, usize> = HashMap::new();

        for candidate in candidates.iter_mut() {
            let count = author_counts.entry(candidate.author_id.clone()).or_insert(0);
            let multiplier = self.multiplier(*count);
            candidate.diversity_multiplier = multiplier;
            candidate.score = candidate.weighted_score * multiplier;
            *count += 1;
        }
    }
}
