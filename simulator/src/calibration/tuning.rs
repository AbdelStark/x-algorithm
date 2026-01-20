use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::calibration::runner::CalibrationSample;
use crate::config::ScoringConfig;
use crate::scoring::ActionWeights;
use crate::{simulate_with_mode, ScoringMode};

pub struct WeightTuner {
    pub calibration_data: Vec<CalibrationSample>,
}

impl WeightTuner {
    pub fn new(calibration_data: Vec<CalibrationSample>) -> Self {
        Self { calibration_data }
    }

    pub fn tune(&self, initial_weights: ActionWeights, config: &ScoringConfig) -> ActionWeights {
        let mut rng = StdRng::seed_from_u64(42);
        let mut best = initial_weights.clone();
        let mut best_score = objective(&best, &self.calibration_data, config);

        let iterations = 200;
        let step = 0.2;

        for _ in 0..iterations {
            let candidate = perturb_weights(&best, &mut rng, step);
            let score = objective(&candidate, &self.calibration_data, config);
            if score < best_score {
                best = candidate;
                best_score = score;
            }
        }

        best
    }
}

fn objective(weights: &ActionWeights, data: &[CalibrationSample], config: &ScoringConfig) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mut config = config.clone();
    config.weights = weights.clone();

    let mut total_error = 0.0;
    for sample in data {
        let input = sample.to_input();
        let output = simulate_with_mode(
            &input,
            None,
            None,
            ScoringMode::Heuristic,
            None,
            &config,
        );
        let predicted = output.unique_engagement_rate;
        let actual = sample.engagement_rate();
        total_error += (predicted - actual).powi(2);
    }

    (total_error / data.len() as f64).sqrt()
}

fn perturb_weights(weights: &ActionWeights, rng: &mut StdRng, scale: f64) -> ActionWeights {
    let mut adjust = |value: f64| -> f64 { value * (1.0 + rng.gen_range(-scale..scale)) };

    ActionWeights {
        favorite: adjust(weights.favorite),
        reply: adjust(weights.reply),
        repost: adjust(weights.repost),
        photo_expand: adjust(weights.photo_expand),
        click: adjust(weights.click),
        profile_click: adjust(weights.profile_click),
        vqv: adjust(weights.vqv),
        share: adjust(weights.share),
        share_dm: adjust(weights.share_dm),
        share_link: adjust(weights.share_link),
        dwell: adjust(weights.dwell),
        quote: adjust(weights.quote),
        quoted_click: adjust(weights.quoted_click),
        follow_author: adjust(weights.follow_author),
        not_interested: adjust(weights.not_interested),
        block: adjust(weights.block),
        mute: adjust(weights.mute),
        report: adjust(weights.report),
        dwell_time: adjust(weights.dwell_time),
    }
}
