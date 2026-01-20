use serde::{Deserialize, Serialize};

use crate::scoring::ScoredCandidate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OonScorerConfig {
    pub multiplier: f64,
}

impl Default for OonScorerConfig {
    fn default() -> Self {
        Self { multiplier: 0.8 }
    }
}

#[derive(Debug, Clone)]
pub struct OonScorer {
    config: OonScorerConfig,
}

impl OonScorer {
    pub fn new(config: OonScorerConfig) -> Self {
        Self { config }
    }

    pub fn score(&self, candidate: &mut ScoredCandidate, is_oon: bool) {
        if is_oon {
            candidate.oon_multiplier = self.config.multiplier;
            candidate.score *= self.config.multiplier;
        } else {
            candidate.oon_multiplier = 1.0;
        }
    }
}
