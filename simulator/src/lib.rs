pub mod config;
pub mod calibration;
pub mod phoenix_client;
pub mod scoring;
pub mod user;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::config::ScoringConfig;
use crate::scoring::{AuthorDiversityScorer, OonScorer, ScoredCandidate, ScoringPipeline, WeightedScorer};

#[derive(Debug, Clone, Copy)]
pub enum MediaType {
    None,
    Image,
    Video,
    Gif,
}

impl MediaType {
    pub fn from_str(value: &str) -> Option<Self> {
        match value.to_lowercase().as_str() {
            "none" | "text" => Some(MediaType::None),
            "image" | "photo" | "pic" => Some(MediaType::Image),
            "video" | "vid" => Some(MediaType::Video),
            "gif" => Some(MediaType::Gif),
            _ => None,
        }
    }

    pub fn media_score(self) -> f64 {
        match self {
            MediaType::None => 0.0,
            MediaType::Image => 0.4,
            MediaType::Gif => 0.6,
            MediaType::Video => 0.8,
        }
    }

    pub fn is_video(self) -> f64 {
        if matches!(self, MediaType::Video) {
            1.0
        } else {
            0.0
        }
    }

    pub fn is_image(self) -> f64 {
        if matches!(self, MediaType::Image | MediaType::Gif) {
            1.0
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone)]
pub struct SimulatorInput {
    pub text: String,
    pub media: MediaType,
    pub post_id: Option<String>,
    pub author_id: Option<String>,
    pub is_oon: bool,
    pub video_duration_seconds: Option<f64>,
    pub has_link_override: Option<bool>,
    pub followers: u64,
    pub following: u64,
    pub account_age_days: u32,
    pub avg_engagement_rate: f64,
    pub posts_per_day: f64,
    pub verified: bool,
    pub hour_of_day: u8,
    pub novelty: f64,
    pub timeliness: f64,
    pub topic_saturation: f64,
    pub audience_fit: f64,
    pub controversy: f64,
    pub sentiment: f64,
}

impl Default for SimulatorInput {
    fn default() -> Self {
        Self {
            text: String::new(),
            media: MediaType::None,
            post_id: None,
            author_id: None,
            is_oon: false,
            video_duration_seconds: None,
            has_link_override: None,
            followers: 1_000,
            following: 500,
            account_age_days: 365,
            avg_engagement_rate: 0.02,
            posts_per_day: 2.0,
            verified: false,
            hour_of_day: 12,
            novelty: 0.5,
            timeliness: 0.5,
            topic_saturation: 0.5,
            audience_fit: 0.6,
            controversy: 0.3,
            sentiment: 0.1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextFeatures {
    pub char_count: usize,
    pub word_count: usize,
    pub hashtags: usize,
    pub mentions: usize,
    pub urls: usize,
    pub questions: usize,
    pub exclamations: usize,
    pub emoji_count: usize,
    pub uppercase_ratio: f64,
    pub avg_word_len: f64,
    pub starts_with_number: bool,
    pub has_hook_word: bool,
    pub cta_share: bool,
    pub cta_reply: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmScore {
    pub hook: f64,
    pub clarity: f64,
    pub novelty: f64,
    pub shareability: f64,
    pub controversy: f64,
    pub sentiment: f64,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmTrace {
    pub model: String,
    pub latency_ms: u128,
    pub prompt_summary: String,
    pub prompt: String,
    pub raw_response: String,
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signals {
    pub length_score: f64,
    pub clarity: f64,
    pub hook: f64,
    pub novelty: f64,
    pub timeliness: f64,
    pub shareability: f64,
    pub content_quality: f64,
    pub author_quality: f64,
    pub audience_alignment: f64,
    pub negative_risk: f64,
    pub media_score: f64,
    pub time_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionProbs {
    pub like: f64,
    pub reply: f64,
    pub repost: f64,
    pub quote: f64,
    pub click: f64,
    pub profile_click: f64,
    pub video_view: f64,
    pub photo_expand: f64,
    pub share: f64,
    pub share_dm: f64,
    pub share_link: f64,
    pub dwell: f64,
    pub follow_author: f64,
    pub quoted_click: f64,
    pub not_interested: f64,
    pub block: f64,
    pub mute: f64,
    pub report: f64,
    pub dwell_time: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum ScoringMode {
    Heuristic,
    Phoenix,
    Hybrid { phoenix_weight: f64 },
}

impl ScoringMode {
    pub fn label(self) -> &'static str {
        match self {
            ScoringMode::Heuristic => "heuristic",
            ScoringMode::Phoenix => "phoenix",
            ScoringMode::Hybrid { .. } => "hybrid",
        }
    }

    pub fn phoenix_weight(self) -> f64 {
        match self {
            ScoringMode::Hybrid { phoenix_weight } => phoenix_weight,
            ScoringMode::Phoenix => 1.0,
            ScoringMode::Heuristic => 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ViralityTier {
    Low,
    Moderate,
    High,
    VeryHigh,
    Breakout,
}

impl ViralityTier {
    pub fn label(self) -> &'static str {
        match self {
            ViralityTier::Low => "Low",
            ViralityTier::Moderate => "Moderate",
            ViralityTier::High => "High",
            ViralityTier::VeryHigh => "Very High",
            ViralityTier::Breakout => "Breakout",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SimulationOutput {
    pub score: f64,
    pub tier: ViralityTier,
    pub scoring_mode: ScoringMode,
    pub weighted_score: f64,
    pub diversity_multiplier: f64,
    pub oon_multiplier: f64,
    pub final_score: f64,
    pub impressions_in: f64,
    pub impressions_oon: f64,
    pub impressions_total: f64,
    pub expected_unique_engagements: f64,
    pub expected_action_volume: f64,
    pub unique_engagement_rate: f64,
    pub action_volume_rate: f64,
    pub actions: ActionProbs,
    pub phoenix_actions: Option<ActionProbs>,
    pub signals: Signals,
    pub suggestions: Vec<String>,
    pub llm: Option<LlmScore>,
    pub llm_trace: Option<LlmTrace>,
}

pub fn extract_text_features(text: &str) -> TextFeatures {
    let mut hashtags = 0usize;
    let mut mentions = 0usize;
    let mut questions = 0usize;
    let mut exclamations = 0usize;
    let mut emoji_count = 0usize;
    let mut uppercase = 0usize;
    let mut letters = 0usize;
    let mut urls = 0usize;

    for ch in text.chars() {
        match ch {
            '#' => hashtags += 1,
            '@' => mentions += 1,
            '?' => questions += 1,
            '!' => exclamations += 1,
            _ => {
                if ch as u32 > 0x7f {
                    emoji_count += 1;
                }
            }
        }

        if ch.is_ascii_alphabetic() {
            letters += 1;
            if ch.is_ascii_uppercase() {
                uppercase += 1;
            }
        }
    }

    let lowercase = text.to_lowercase();
    let urls_list = ["http://", "https://", "www."];
    for needle in urls_list {
        urls += lowercase.matches(needle).count();
    }

    let mut word_total = 0usize;
    let mut word_count = 0usize;
    for word in text.split_whitespace() {
        let len = word.chars().filter(|c| c.is_ascii_alphabetic()).count();
        if len > 0 {
            word_total += len;
            word_count += 1;
        }
    }

    let avg_word_len = if word_count == 0 {
        0.0
    } else {
        word_total as f64 / word_count as f64
    };

    let uppercase_ratio = if letters == 0 {
        0.0
    } else {
        uppercase as f64 / letters as f64
    };

    let starts_with_number = text
        .chars()
        .find(|c| !c.is_whitespace())
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false);

    let hook_words = [
        "how", "why", "what", "stop", "new", "breaking", "secret", "tips", "guide", "learn",
        "thread", "facts", "proof", "mistakes", "warning",
    ];
    let has_hook_word = hook_words.iter().any(|word| lowercase.contains(word));

    let cta_share = ["retweet", "repost", "share", "rt ", "boost"].iter().any(|w| {
        lowercase.contains(w)
    });
    let cta_reply = ["thoughts", "what do you think", "agree", "disagree", "reply", "comment"]
        .iter()
        .any(|w| lowercase.contains(w));

    TextFeatures {
        char_count: text.chars().count(),
        word_count,
        hashtags,
        mentions,
        urls,
        questions,
        exclamations,
        emoji_count,
        uppercase_ratio,
        avg_word_len,
        starts_with_number,
        has_hook_word,
        cta_share,
        cta_reply,
    }
}

fn load_scoring_config() -> ScoringConfig {
    ScoringConfig::load(None)
        .map(|(config, _)| config)
        .unwrap_or_default()
}

pub fn simulate(input: &SimulatorInput) -> SimulationOutput {
    let config = load_scoring_config();
    simulate_with_mode(input, None, None, ScoringMode::Heuristic, None, &config)
}

pub fn simulate_with_llm(
    input: &SimulatorInput,
    llm: Option<&LlmScore>,
    llm_trace: Option<&LlmTrace>,
) -> SimulationOutput {
    let config = load_scoring_config();
    simulate_with_mode(
        input,
        llm,
        llm_trace,
        ScoringMode::Heuristic,
        None,
        &config,
    )
}

pub fn simulate_with_mode(
    input: &SimulatorInput,
    llm: Option<&LlmScore>,
    llm_trace: Option<&LlmTrace>,
    scoring_mode: ScoringMode,
    phoenix_actions: Option<&ActionProbs>,
    scoring_config: &ScoringConfig,
) -> SimulationOutput {
    let features = extract_text_features(&input.text);
    let media_score = input.media.media_score();
    let has_link = input
        .has_link_override
        .unwrap_or(features.urls > 0);
    let link_flag = if has_link { 1.0 } else { 0.0 };

    let length_score = gaussian(features.char_count as f64, 140.0, 70.0);
    let readability_score = gaussian(features.avg_word_len, 5.0, 2.0);

    let exclaim_factor = (features.exclamations as f64 / 3.0).min(1.0);
    let hashtag_factor = (features.hashtags as f64 / 5.0).min(1.0);
    let mention_factor = (features.mentions as f64 / 4.0).min(1.0);

    let spamminess = clamp01(
        0.2 * exclaim_factor
            + 0.25 * hashtag_factor
            + 0.2 * mention_factor
            + 0.3 * clamp01(features.uppercase_ratio / 0.4)
            + 0.2 * link_flag,
    );

    let base_hook = clamp01(
        0.35 * bool_to_f64(features.questions > 0)
            + 0.2 * bool_to_f64(features.exclamations > 0)
            + 0.25 * bool_to_f64(features.starts_with_number)
            + 0.2 * bool_to_f64(features.has_hook_word),
    );

    let base_clarity =
        clamp01(0.5 * length_score + 0.3 * readability_score + 0.2 * (1.0 - spamminess));
    let mut novelty = clamp01(input.novelty);
    let timeliness = clamp01(input.timeliness);
    let mut controversy = clamp01(input.controversy);
    let mut sentiment = input.sentiment.max(-1.0).min(1.0);

    let mut hook = base_hook;
    let mut clarity = base_clarity;
    if let Some(score) = llm {
        hook = blend_signal(hook, clamp01(score.hook), 0.6);
        clarity = blend_signal(clarity, clamp01(score.clarity), 0.6);
        novelty = blend_signal(novelty, clamp01(score.novelty), 0.6);
        controversy = blend_signal(controversy, clamp01(score.controversy), 0.5);
        sentiment = blend_sentiment(sentiment, score.sentiment, 0.5);
    }

    let mut shareability = clamp01(
        0.4 * hook
            + 0.3 * novelty
            + 0.2 * clarity
            + 0.1 * bool_to_f64(features.cta_share),
    );
    if let Some(score) = llm {
        shareability = blend_signal(shareability, clamp01(score.shareability), 0.6);
    }

    let content_quality = clamp01(0.45 * clarity + 0.25 * hook + 0.2 * novelty + 0.1 * timeliness);

    let followers_log = log10_safe((input.followers as f64) + 1.0);
    let author_strength = clamp01((followers_log - 2.0) / 3.0);

    let ratio = if input.following == 0 {
        1.0
    } else {
        (input.followers as f64) / (input.following as f64)
    };
    let ratio_score = clamp01(log10_safe(ratio + 1.0) / 2.0);

    let age_years = input.account_age_days as f64 / 365.0;
    let age_score = clamp01(age_years / 5.0);

    let eng_score = clamp01(input.avg_engagement_rate / 0.06);

    let cadence_score = gaussian(input.posts_per_day, 2.0, 2.5);
    let verified_bonus = if input.verified { 0.1 } else { 0.0 };

    let author_quality = clamp01(
        0.35 * author_strength
            + 0.2 * age_score
            + 0.2 * eng_score
            + 0.15 * ratio_score
            + 0.1 * cadence_score
            + verified_bonus,
    );

    let topic_saturation = clamp01(input.topic_saturation);
    let audience_fit = clamp01(input.audience_fit);

    let audience_alignment =
        clamp01(0.6 * audience_fit + 0.2 * (1.0 - topic_saturation) + 0.2 * ratio_score);

    let negative_sentiment = (-sentiment).max(0.0);
    let caps_risk = clamp01(features.uppercase_ratio / 0.35) * 0.2;
    let negative_risk = clamp01(
        0.4 * controversy
            + 0.25 * spamminess
            + 0.15 * negative_sentiment
            + caps_risk
            + 0.1 * topic_saturation,
    );

    let positive_signal = clamp01(0.4 * content_quality + 0.35 * author_quality + 0.25 * audience_alignment);
    let viral_lift = clamp01(0.5 * hook + 0.3 * novelty + 0.2 * media_score);

    let base = -2.0 + 3.2 * positive_signal + 1.4 * viral_lift - 2.2 * negative_risk;

    let has_question = bool_to_f64(features.questions > 0);
    let cta_reply = bool_to_f64(features.cta_reply);
    let cta_share = bool_to_f64(features.cta_share);
    let is_video = input.media.is_video();
    let is_image = input.media.is_image();

    let like = sigmoid(base + 0.6 * media_score + 0.2 * sentiment.max(0.0));
    let reply = sigmoid(
        base - 0.2 * media_score + 0.6 * has_question + 0.3 * controversy + 0.2 * cta_reply,
    );
    let repost =
        sigmoid(base + 0.6 * shareability + 0.3 * novelty - 0.3 * link_flag + 0.1 * cta_share);
    let quote = sigmoid(base + 0.4 * controversy + 0.2 * novelty);
    let click = sigmoid(base + 0.9 * link_flag + 0.2 * hook);
    let profile_click = sigmoid(base + 0.5 * author_quality + 0.2 * novelty);
    let video_view = sigmoid(base + 1.2 * is_video + 0.2 * hook);
    let photo_expand = sigmoid(base + 1.0 * is_image + 0.1 * hook);
    let share = sigmoid(base + 0.5 * shareability + 0.2 * novelty);
    let share_dm = sigmoid(base + 0.35 * shareability + 0.1 * novelty - 0.1 * link_flag);
    let share_link = sigmoid(base + 0.25 * shareability + 0.2 * link_flag);
    let dwell = sigmoid(base + 0.2 * length_score + 0.4 * media_score - 0.2 * link_flag);
    let follow_author = sigmoid(base + 0.6 * author_quality + 0.2 * hook);
    let quoted_click = sigmoid(base + 0.4 * controversy + 0.2 * hook + 0.1 * novelty);
    let not_interested = sigmoid(
        -1.0 + 2.2 * negative_risk + 0.6 * topic_saturation - 0.8 * audience_alignment,
    );
    let block = sigmoid(-2.0 + 2.6 * negative_risk + 0.6 * controversy);
    let mute = sigmoid(-1.8 + 2.3 * negative_risk + 0.8 * topic_saturation);
    let report = sigmoid(-2.4 + 2.8 * negative_risk + 0.6 * controversy);
    let dwell_time = estimate_dwell_time(features.char_count, media_score, dwell);

    let heuristic_actions = ActionProbs {
        like,
        reply,
        repost,
        quote,
        click,
        profile_click,
        video_view,
        photo_expand,
        share,
        share_dm,
        share_link,
        dwell,
        follow_author,
        quoted_click,
        not_interested,
        block,
        mute,
        report,
        dwell_time,
    };

    let actions = match scoring_mode {
        ScoringMode::Heuristic => heuristic_actions.clone(),
        ScoringMode::Phoenix => phoenix_actions
            .cloned()
            .unwrap_or_else(|| heuristic_actions.clone()),
        ScoringMode::Hybrid { phoenix_weight } => {
            if let Some(phoenix_actions) = phoenix_actions {
                blend_actions(&heuristic_actions, phoenix_actions, phoenix_weight)
            } else {
                heuristic_actions.clone()
            }
        }
    };

    let pipeline = build_pipeline(scoring_config);
    let mut candidates = vec![ScoredCandidate::new(
        derive_post_id(input),
        derive_author_id(input),
        input.is_oon,
        derive_video_duration(input),
        actions.clone(),
    )];
    pipeline.score(&mut candidates);
    let candidate = candidates.pop().unwrap_or_else(|| {
        ScoredCandidate::new(
            "post".to_string(),
            "author".to_string(),
            false,
            None,
            actions.clone(),
        )
    });

    let weighted_score = candidate.weighted_score;
    let diversity_multiplier = candidate.diversity_multiplier;
    let oon_multiplier = candidate.oon_multiplier;
    let final_score = candidate.score;

    let time_score = time_of_day_score(input.hour_of_day);
    let active_fraction = 0.015 + 0.08 * time_score;
    let impressions_in = (input.followers as f64)
        * active_fraction
        * (0.6 + 0.4 * audience_alignment)
        .max(0.0);

    let oon_seed = 300.0 + 1400.0 * positive_signal;
    let oon_reach_multiplier = 1.0 + clamp01((weighted_score - 1.0) / 3.0) * 4.0;
    let mut impressions_oon = oon_seed
        * oon_reach_multiplier
        * (0.5 + 0.5 * content_quality)
        * (1.0 - 0.7 * topic_saturation)
        * (1.0 - 0.5 * negative_risk);

    if impressions_oon.is_nan() || impressions_oon.is_sign_negative() {
        impressions_oon = 0.0;
    }

    let impressions_total = impressions_in + impressions_oon;

    let action_volume_rate = action_volume_rate(&actions);
    let unique_engagement_rate = unique_engagement_rate(&actions);
    let expected_action_volume = impressions_total * action_volume_rate;
    let expected_unique_engagements = impressions_total * unique_engagement_rate;

    let raw = (final_score - 1.0) * 0.8 + (log10_safe(impressions_total + 1.0) - 3.0) * 0.4;
    let score = 100.0 * sigmoid(raw);
    let tier = tier_from_score(score);

    let signals = Signals {
        length_score,
        clarity,
        hook,
        novelty,
        timeliness,
        shareability,
        content_quality,
        author_quality,
        audience_alignment,
        negative_risk,
        media_score,
        time_score,
    };

    let mut suggestions = build_suggestions(input, &features, &signals, &actions, weighted_score);
    if let Some(score) = llm {
        merge_suggestions(&mut suggestions, &score.suggestions);
    }

    SimulationOutput {
        score,
        tier,
        scoring_mode,
        weighted_score,
        diversity_multiplier,
        oon_multiplier,
        final_score,
        impressions_in,
        impressions_oon,
        impressions_total,
        expected_unique_engagements,
        expected_action_volume,
        unique_engagement_rate,
        action_volume_rate,
        actions,
        phoenix_actions: phoenix_actions.cloned(),
        signals,
        suggestions,
        llm: llm.cloned(),
        llm_trace: llm_trace.cloned(),
    }
}

fn build_pipeline(config: &ScoringConfig) -> ScoringPipeline {
    let weighted = WeightedScorer::new(
        config.weights.clone(),
        config.weighted.vqv_duration_threshold,
        config.weighted.score_offset,
    );
    let diversity = AuthorDiversityScorer::new(config.diversity.clone());
    let oon = OonScorer::new(config.oon.clone());
    ScoringPipeline::new(weighted, diversity, oon)
}

fn derive_post_id(input: &SimulatorInput) -> String {
    input.post_id.clone().unwrap_or_else(|| {
        let hash = stable_hash64(&input.text);
        format!("post_{:x}", hash)
    })
}

fn derive_author_id(input: &SimulatorInput) -> String {
    input.author_id.clone().unwrap_or_else(|| {
        let payload = format!(
            "{}:{}:{}:{}",
            input.followers, input.following, input.account_age_days, input.verified
        );
        let hash = stable_hash64(&payload);
        format!("author_{:x}", hash)
    })
}

fn derive_video_duration(input: &SimulatorInput) -> Option<f64> {
    if let Some(duration) = input.video_duration_seconds {
        return Some(duration.max(0.0));
    }
    if matches!(input.media, MediaType::Video) {
        return Some(15.0);
    }
    None
}

fn estimate_dwell_time(char_count: usize, media_score: f64, dwell_prob: f64) -> f64 {
    let base = 1.5 + (char_count as f64 / 80.0);
    let media_lift = 6.0 * media_score;
    let dwell_lift = 10.0 * dwell_prob;
    let estimate = base + media_lift + dwell_lift;
    estimate.max(0.0).min(60.0)
}

fn blend_actions(base: &ActionProbs, overlay: &ActionProbs, weight: f64) -> ActionProbs {
    let blend_prob = |a: f64, b: f64| clamp01(a * (1.0 - weight) + b * weight);
    let blend_value = |a: f64, b: f64| a * (1.0 - weight) + b * weight;

    ActionProbs {
        like: blend_prob(base.like, overlay.like),
        reply: blend_prob(base.reply, overlay.reply),
        repost: blend_prob(base.repost, overlay.repost),
        quote: blend_prob(base.quote, overlay.quote),
        click: blend_prob(base.click, overlay.click),
        profile_click: blend_prob(base.profile_click, overlay.profile_click),
        video_view: blend_prob(base.video_view, overlay.video_view),
        photo_expand: blend_prob(base.photo_expand, overlay.photo_expand),
        share: blend_prob(base.share, overlay.share),
        share_dm: blend_prob(base.share_dm, overlay.share_dm),
        share_link: blend_prob(base.share_link, overlay.share_link),
        dwell: blend_prob(base.dwell, overlay.dwell),
        follow_author: blend_prob(base.follow_author, overlay.follow_author),
        quoted_click: blend_prob(base.quoted_click, overlay.quoted_click),
        not_interested: blend_prob(base.not_interested, overlay.not_interested),
        block: blend_prob(base.block, overlay.block),
        mute: blend_prob(base.mute, overlay.mute),
        report: blend_prob(base.report, overlay.report),
        dwell_time: blend_value(base.dwell_time, overlay.dwell_time),
    }
}

fn stable_hash64(value: &str) -> u64 {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&digest[..8]);
    u64::from_be_bytes(bytes)
}

fn build_suggestions(
    input: &SimulatorInput,
    features: &TextFeatures,
    signals: &Signals,
    actions: &ActionProbs,
    weighted_score: f64,
) -> Vec<String> {
    let mut suggestions = Vec::new();
    if features.char_count < 50 {
        suggestions.push("Add a clearer hook and more context; aim for ~80-200 characters.".to_string());
    }
    if features.char_count > 260 {
        suggestions.push("Trim to under ~220 characters to improve early engagement velocity.".to_string());
    }
    if features.hashtags > 3 {
        suggestions.push("Reduce hashtags to 1-2; too many can look spammy.".to_string());
    }
    if features.mentions > 2 {
        suggestions.push("Limit mentions; too many can reduce reach and clarity.".to_string());
    }
    if input.has_link_override.unwrap_or(features.urls > 0) {
        suggestions.push("Links often reduce in-feed engagement; consider moving the link to a reply.".to_string());
    }
    if matches!(input.media, MediaType::None) && features.char_count < 160 {
        suggestions.push("Consider adding an image or video to boost dwell and shares.".to_string());
    }
    if signals.hook < 0.35 {
        suggestions.push("Strengthen the first line with a question, surprising stat, or bold claim.".to_string());
    }
    if signals.shareability < 0.4 {
        suggestions.push("Make it more shareable: concise takeaway, list, or strong opinion.".to_string());
    }
    if signals.clarity < 0.5 {
        suggestions.push("Simplify wording; shorter words and fewer clauses improve scanability.".to_string());
    }
    if features.uppercase_ratio > 0.3 {
        suggestions.push("Reduce ALL CAPS; it increases negative feedback signals.".to_string());
    }
    if signals.negative_risk > 0.55 {
        suggestions.push("Tone down contentious framing to reduce not-interested/report signals.".to_string());
    }
    if input.topic_saturation > 0.6 {
        suggestions.push("High topic saturation; use a unique angle or niche framing.".to_string());
    }
    if input.audience_fit < 0.5 {
        suggestions.push("Align the topic with follower interests to boost initial velocity.".to_string());
    }
    if signals.time_score < 0.4 {
        suggestions.push("Post during peak hours (around 9-11am or 7-9pm local).".to_string());
    }
    if weighted_score < 0.8 {
        suggestions.push("Focus on increasing repost/share intent; that is a main driver of out-of-network reach.".to_string());
    }
    if actions.reply < 0.02 && features.questions == 0 {
        suggestions.push("Invite replies with a direct question or a clear prompt.".to_string());
    }

    suggestions
}

fn merge_suggestions(base: &mut Vec<String>, extras: &[String]) {
    let mut seen: HashSet<String> = base.iter().map(|s| normalize_text(s)).collect();
    for suggestion in extras {
        let normalized = normalize_text(suggestion);
        if normalized.is_empty() || seen.contains(&normalized) {
            continue;
        }
        base.push(suggestion.clone());
        seen.insert(normalized);
    }
    if base.len() > 10 {
        base.truncate(10);
    }
}

fn action_volume_rate(actions: &ActionProbs) -> f64 {
    positive_action_probs(actions).iter().sum()
}

fn unique_engagement_rate(actions: &ActionProbs) -> f64 {
    let mut none_probability = 1.0;
    for probability in positive_action_probs(actions) {
        none_probability *= 1.0 - clamp01(probability);
    }
    clamp01(1.0 - none_probability)
}

fn positive_action_probs(actions: &ActionProbs) -> Vec<f64> {
    vec![
        actions.like,
        actions.reply,
        actions.repost,
        actions.quote,
        actions.share,
        actions.share_dm,
        actions.share_link,
        actions.click,
        actions.profile_click,
        actions.follow_author,
        actions.video_view,
        actions.photo_expand,
        actions.quoted_click,
    ]
}

fn tier_from_score(score: f64) -> ViralityTier {
    if score < 35.0 {
        ViralityTier::Low
    } else if score < 55.0 {
        ViralityTier::Moderate
    } else if score < 75.0 {
        ViralityTier::High
    } else if score < 90.0 {
        ViralityTier::VeryHigh
    } else {
        ViralityTier::Breakout
    }
}

fn time_of_day_score(hour: u8) -> f64 {
    let h = hour.min(23) as f64;
    let morning = gaussian(h, 9.0, 3.5);
    let evening = gaussian(h, 20.0, 3.5);
    morning.max(evening)
}

fn gaussian(x: f64, center: f64, width: f64) -> f64 {
    if width <= 0.0 {
        return 0.0;
    }
    let z = (x - center) / width;
    (-z * z).exp()
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

fn clamp01(value: f64) -> f64 {
    if value.is_nan() {
        return 0.0;
    }
    value.max(0.0).min(1.0)
}

fn bool_to_f64(value: bool) -> f64 {
    if value {
        1.0
    } else {
        0.0
    }
}

fn log10_safe(value: f64) -> f64 {
    if value <= 0.0 {
        0.0
    } else {
        value.log10()
    }
}

fn blend_signal(base: f64, overlay: f64, weight: f64) -> f64 {
    clamp01(base * (1.0 - weight) + overlay * weight)
}

fn blend_sentiment(base: f64, overlay: f64, weight: f64) -> f64 {
    let blended = base * (1.0 - weight) + overlay * weight;
    blended.max(-1.0).min(1.0)
}

fn normalize_text(value: &str) -> String {
    value
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn format_number(value: f64) -> String {
    let rounded = value.round().max(0.0) as i64;
    let mut chars: Vec<char> = rounded.to_string().chars().collect();
    let mut result = String::new();
    let mut count = 0usize;

    while let Some(ch) = chars.pop() {
        if count == 3 {
            result.push(',');
            count = 0;
        }
        result.push(ch);
        count += 1;
    }

    result.chars().rev().collect()
}

pub fn format_percent(value: f64) -> String {
    format!("{:.1}%", value * 100.0)
}

pub fn format_float(value: f64, digits: usize) -> String {
    format!("{:.1$}", value, digits)
}
