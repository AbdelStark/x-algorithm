use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};

use crate::scoring::{ActionWeights, AuthorDiversityConfig, OonScorerConfig};
use crate::ScoringMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringModeConfig {
    pub mode: String,
    pub phoenix_weight: f64,
}

impl Default for ScoringModeConfig {
    fn default() -> Self {
        Self {
            mode: "hybrid".to_string(),
            phoenix_weight: 0.7,
        }
    }
}

impl ScoringModeConfig {
    pub fn to_mode(&self) -> ScoringMode {
        match self.mode.to_lowercase().as_str() {
            "phoenix" => ScoringMode::Phoenix,
            "hybrid" => ScoringMode::Hybrid {
                phoenix_weight: self.phoenix_weight.clamp(0.0, 1.0),
            },
            _ => ScoringMode::Heuristic,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoenixConfig {
    pub endpoint: String,
    pub timeout_ms: u64,
    pub history_limit: usize,
}

impl Default for PhoenixConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:8000".to_string(),
            timeout_ms: 5000,
            history_limit: 50,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedConfig {
    pub vqv_duration_threshold: f64,
    pub score_offset: f64,
}

impl Default for WeightedConfig {
    fn default() -> Self {
        Self {
            vqv_duration_threshold: 6.0,
            score_offset: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfig {
    pub scoring: ScoringModeConfig,
    pub weights: ActionWeights,
    pub weighted: WeightedConfig,
    pub diversity: AuthorDiversityConfig,
    pub oon: OonScorerConfig,
    pub phoenix: PhoenixConfig,
}

impl Default for ScoringConfig {
    fn default() -> Self {
        Self {
            scoring: ScoringModeConfig::default(),
            weights: ActionWeights::default(),
            weighted: WeightedConfig::default(),
            diversity: AuthorDiversityConfig::default(),
            oon: OonScorerConfig::default(),
            phoenix: PhoenixConfig::default(),
        }
    }
}

impl ScoringConfig {
    pub fn load(path: Option<PathBuf>) -> Result<(Self, Option<PathBuf>), String> {
        let config_path = path.or_else(default_config_path);
        let mut config = if let Some(path) = config_path.as_ref() {
            if path.exists() {
                let contents = std::fs::read_to_string(path)
                    .map_err(|err| format!("failed to read config: {}", err))?;
                toml::from_str(&contents)
                    .map_err(|err| format!("failed to parse config: {}", err))?
            } else {
                ScoringConfig::default()
            }
        } else {
            ScoringConfig::default()
        };

        config.apply_env_overrides();
        Ok((config, config_path))
    }

    pub fn write(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create config dir: {}", err))?;
        }
        let payload = toml::to_string_pretty(self)
            .map_err(|err| format!("failed to serialize config: {}", err))?;
        std::fs::write(path, payload)
            .map_err(|err| format!("failed to write config: {}", err))?;
        Ok(())
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(mode) = env::var("SCORING_MODE") {
            if !mode.trim().is_empty() {
                self.scoring.mode = mode;
            }
        }
        if let Ok(weight) = env::var("PHOENIX_WEIGHT") {
            if let Ok(value) = weight.parse::<f64>() {
                self.scoring.phoenix_weight = value;
            }
        }
        if let Ok(endpoint) = env::var("PHOENIX_ENDPOINT") {
            if !endpoint.trim().is_empty() {
                self.phoenix.endpoint = endpoint;
            }
        }
        if let Ok(timeout) = env::var("PHOENIX_TIMEOUT_MS") {
            if let Ok(value) = timeout.parse::<u64>() {
                self.phoenix.timeout_ms = value;
            }
        }
        if let Ok(history_limit) = env::var("PHOENIX_HISTORY_LIMIT") {
            if let Ok(value) = history_limit.parse::<usize>() {
                self.phoenix.history_limit = value;
            }
        }
    }
}

fn default_config_path() -> Option<PathBuf> {
    env::var("SCORING_CONFIG_PATH")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .or_else(|| Some(PathBuf::from("config/scoring.toml")))
}
