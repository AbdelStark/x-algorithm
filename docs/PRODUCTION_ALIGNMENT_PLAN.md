# Production Alignment Plan: X For You Algorithm Simulator

This document outlines a comprehensive plan to evolve the simulator from its current heuristic-based implementation toward a more production-faithful scoring system. The goal is to leverage the existing Phoenix JAX models and align the simulator's behavior with the documented production pipeline.

---

## Executive Summary

**Current State:** The simulator uses hand-crafted heuristics with optional Grok LLM enhancement, and it can call the Phoenix ranking model via a local service. The 4-stage scoring pipeline (Phoenix → Weighted → Diversity → OON), configurable weights, user history, and calibration tooling are implemented.

**Target State:** A simulator that:
1. Runs the actual Phoenix JAX ranking model for engagement prediction
2. Implements the 4-stage production scoring pipeline (Phoenix → Weighted → AuthorDiversity → OON)
3. Uses configurable, production-aligned weights
4. Supports user engagement history for personalized predictions

**Estimated Effort:** Phases 1-4 implemented; remaining effort centers on data collection, retrieval integration, and validation.

---

## Gap Analysis

### What Exists Today

| Component | Location | Description |
|-----------|----------|-------------|
| Heuristic scorer | `simulator/src/lib.rs` | Text features → signals → action probabilities via sigmoid formulas |
| LLM enhancement | `simulator/src/llm.rs` | Grok API for hook/clarity/novelty scoring |
| Phoenix service + client | `phoenix/service/`, `simulator/src/phoenix_client.rs` | Optional Phoenix ranking model via local HTTP service |
| 19 action types | `simulator/src/lib.rs` | ActionProbs aligned to Phoenix action taxonomy |
| Scoring pipeline | `simulator/src/scoring/`, `config/scoring.toml` | Phoenix/heuristic → weighted → diversity → OON pipeline |
| User profiles/history | `simulator/src/user/`, `simulator/src/server.rs` | Stored profiles + synthetic history for Phoenix requests |
| Calibration CLI | `simulator/src/calibration/`, `docs/CALIBRATION.md` | Metrics, report output, and weight tuning |
| Impressions estimate | `simulator/src/lib.rs` | In-network + OON synthetic estimates |

### What Production Has (From Open Source)

| Component | Location | Description |
|-----------|----------|-------------|
| Phoenix Ranking Model | `phoenix/recsys_model.py` | JAX/Haiku transformer predicting 19 actions |
| Phoenix Retrieval Model | `phoenix/recsys_retrieval_model.py` | Two-tower model for candidate retrieval |
| 19 engagement types | `phoenix/runners.py:202-222` | ACTIONS list including dwell_time |
| Weighted Scorer | `home-mixer/scorers/weighted_scorer.rs` | Production weight application |
| Author Diversity Scorer | `home-mixer/scorers/author_diversity_scorer.rs` | Exponential decay for repeated authors |
| OON Scorer | `home-mixer/scorers/oon_scorer.rs` | Out-of-network score adjustment |
| 4-stage pipeline | `home-mixer/candidate_pipeline/` | Phoenix → Weighted → Diversity → OON |

### Critical Gaps

| Gap | Severity | Impact |
|-----|----------|--------|
| Phoenix retrieval not integrated | High | Candidate generation remains simulated |
| Production weights/embeddings missing | High | Scores cannot match production distributions |
| Filters/hydrators not executed | High | Visibility, safety, and metadata logic omitted |
| Calibration data volume missing | Medium | Accuracy vs production not validated |
| Latency/throughput benchmarking | Medium | Performance targets not enforced |
| Score normalization vs production | Medium | Final score scaling may differ |

---

## Implementation Plan

### Phase 1: Phoenix Model Integration (Core)

**Goal:** Wire the Phoenix JAX ranking model into the simulator as an optional scoring backend.

#### 1.1 Python Service for Phoenix Model

Create a lightweight HTTP/gRPC service that wraps the Phoenix model:

```
phoenix/
├── service/
│   ├── server.py         # FastAPI/gRPC server
│   ├── inference.py      # Model loading and inference
│   └── schema.py         # Request/response models
```

**Input Schema:**
```python
@dataclass
class RankingRequest:
    # User context (simplified for simulator)
    user_id: str
    user_embedding: Optional[List[float]]  # Pre-computed or generated

    # Engagement history (optional, enables personalization)
    history_posts: List[PostFeatures]
    history_actions: List[List[float]]  # [seq_len, 19] action indicators

    # Candidates to score (single post for simulator)
    candidates: List[PostFeatures]

@dataclass
class PostFeatures:
    post_id: str
    author_id: str
    text_hash: int           # Simulated hash for embedding lookup
    author_hash: int         # Simulated hash for author embedding
    product_surface: int     # 0=home, 1=search, etc.
```

**Output Schema:**
```python
@dataclass
class RankingResponse:
    scores: List[CandidateScore]

@dataclass
class CandidateScore:
    post_id: str
    phoenix_scores: PhoenixScores  # All 19 action probabilities
    weighted_score: float          # Combined score
    rank: int

@dataclass
class PhoenixScores:
    like: float
    reply: float
    repost: float
    photo_expand: float
    click: float
    profile_click: float
    video_view: float
    share: float
    share_dm: float
    share_link: float
    dwell: float
    quote: float
    quoted_click: float
    follow_author: float
    not_interested: float
    block: float
    mute: float
    report: float
    dwell_time: float  # Continuous value
```

#### 1.2 Rust Client Integration

Add Phoenix client to simulator:

```rust
// simulator/src/phoenix_client.rs

pub struct PhoenixClient {
    endpoint: String,
    timeout: Duration,
}

impl PhoenixClient {
    pub async fn score(&self, request: RankingRequest) -> Result<RankingResponse, PhoenixError>;
}
```

#### 1.3 Hybrid Scoring Mode

Update `lib.rs` to support three modes:

```rust
pub enum ScoringMode {
    Heuristic,          // Current behavior (fast, no ML)
    Phoenix,            // ML-only scoring
    Hybrid {            // ML with heuristic fallback
        phoenix_weight: f64,  // 0.7 = 70% Phoenix, 30% heuristic
    },
}
```

**Implementation in simulate_with_llm:**
```rust
pub async fn simulate_with_ml(
    input: &SimulatorInput,
    mode: ScoringMode,
    phoenix: Option<&PhoenixClient>,
    llm: Option<&LlmScore>,
) -> SimulationOutput {
    let heuristic_actions = compute_heuristic_actions(input);

    let actions = match (mode, phoenix) {
        (ScoringMode::Phoenix, Some(client)) => {
            client.score(input.to_ranking_request()).await?
        }
        (ScoringMode::Hybrid { weight }, Some(client)) => {
            let ml_actions = client.score(input.to_ranking_request()).await?;
            blend_actions(&heuristic_actions, &ml_actions, weight)
        }
        _ => heuristic_actions,
    };

    // Continue with weighted scoring, etc.
}
```

#### 1.4 Tasks

- [x] Create `phoenix/service/` directory structure
- [x] Implement FastAPI server with `/rank` endpoint
- [x] Add model loading using existing `RecsysInferenceRunner`
- [x] Create hash simulation for text → embedding lookup
- [x] Add Rust HTTP client in `simulator/src/phoenix_client.rs`
- [x] Update `server.rs` to optionally call Phoenix service
- [x] Add `PHOENIX_ENDPOINT` environment variable
- [x] Write integration tests

---

### Phase 2: Production Scoring Pipeline

**Goal:** Implement the 4-stage scoring pipeline matching production architecture.

#### 2.1 Align ActionProbs with Production

Expand from 15 to 19 action types:

```rust
// simulator/src/lib.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionProbs {
    // Existing
    pub like: f64,           // favorite_score
    pub reply: f64,          // reply_score
    pub repost: f64,         // repost_score
    pub quote: f64,          // quote_score
    pub click: f64,          // click_score
    pub profile_click: f64,  // profile_click_score
    pub video_view: f64,     // vqv_score
    pub photo_expand: f64,   // photo_expand_score
    pub share: f64,          // share_score
    pub dwell: f64,          // dwell_score
    pub follow_author: f64,  // follow_author_score
    pub not_interested: f64, // not_interested_score
    pub block: f64,          // block_author_score
    pub mute: f64,           // mute_author_score
    pub report: f64,         // report_score

    // New (matching production)
    pub share_dm: f64,       // share_via_dm_score
    pub share_link: f64,     // share_via_copy_link_score
    pub quoted_click: f64,   // quoted_click_score
    pub dwell_time: f64,     // dwell_time (continuous, in seconds)
}
```

#### 2.2 Weighted Scorer Implementation

Create a configurable weighted scorer matching production:

```rust
// simulator/src/scoring/weighted_scorer.rs

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
        // Best-effort approximation based on open-source structure
        // Real weights are in excluded params module
        Self {
            favorite: 1.0,
            reply: 1.6,
            repost: 2.0,
            photo_expand: 0.3,
            click: 0.4,
            profile_click: 0.3,
            vqv: 0.5,       // Only applied if video duration > threshold
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
            dwell_time: 0.1,  // Per-second weight
        }
    }
}

pub struct WeightedScorer {
    weights: ActionWeights,
    vqv_duration_threshold: f64,  // Seconds
    score_offset: f64,
}

impl WeightedScorer {
    pub fn score(&self, actions: &ActionProbs, video_duration: Option<f64>) -> f64 {
        let mut score = 0.0;

        // Positive actions
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

        // VQV only if video is long enough
        if let Some(duration) = video_duration {
            if duration >= self.vqv_duration_threshold {
                score += actions.video_view * self.weights.vqv;
            }
        }

        // Negative actions
        score += actions.not_interested * self.weights.not_interested;
        score += actions.block * self.weights.block;
        score += actions.mute * self.weights.mute;
        score += actions.report * self.weights.report;

        // Continuous dwell time
        score += actions.dwell_time * self.weights.dwell_time;

        // Apply offset for negative scores (production behavior)
        if score < 0.0 {
            score = self.offset_score(score);
        }

        score
    }

    fn offset_score(&self, score: f64) -> f64 {
        // Shift negative scores to positive range
        // Exact formula from production is unknown
        score + self.score_offset
    }
}
```

#### 2.3 Author Diversity Scorer

Implement the exponential decay diversity scorer:

```rust
// simulator/src/scoring/author_diversity_scorer.rs

pub struct AuthorDiversityConfig {
    pub decay: f64,     // e.g., 0.7 - multiplier for each repeated occurrence
    pub floor: f64,     // e.g., 0.1 - minimum multiplier
}

impl Default for AuthorDiversityConfig {
    fn default() -> Self {
        Self {
            decay: 0.7,
            floor: 0.1,
        }
    }
}

pub struct AuthorDiversityScorer {
    config: AuthorDiversityConfig,
}

impl AuthorDiversityScorer {
    /// Calculate multiplier for nth occurrence of author (0-indexed)
    pub fn multiplier(&self, occurrence: usize) -> f64 {
        // multiplier = (1 - floor) * decay^occurrence + floor
        let decay_factor = self.config.decay.powi(occurrence as i32);
        (1.0 - self.config.floor) * decay_factor + self.config.floor
    }

    /// Score a list of candidates, applying diversity penalties
    /// Assumes candidates are pre-sorted by weighted_score descending
    pub fn score(&self, candidates: &mut [ScoredCandidate]) {
        let mut author_counts: HashMap<String, usize> = HashMap::new();

        for candidate in candidates.iter_mut() {
            let count = author_counts.entry(candidate.author_id.clone()).or_insert(0);
            let multiplier = self.multiplier(*count);
            candidate.score = candidate.weighted_score * multiplier;
            *count += 1;
        }
    }
}
```

#### 2.4 OON Scorer

Implement out-of-network score adjustment:

```rust
// simulator/src/scoring/oon_scorer.rs

pub struct OONScorerConfig {
    pub oon_multiplier: f64,  // e.g., 0.8 - penalty for OON content
}

impl Default for OONScorerConfig {
    fn default() -> Self {
        Self {
            oon_multiplier: 0.8,
        }
    }
}

pub struct OONScorer {
    config: OONScorerConfig,
}

impl OONScorer {
    pub fn score(&self, candidate: &mut ScoredCandidate, is_oon: bool) {
        if is_oon {
            candidate.score *= self.config.oon_multiplier;
        }
    }
}
```

#### 2.5 Unified Pipeline

Combine all scorers into a single pipeline:

```rust
// simulator/src/scoring/pipeline.rs

pub struct ScoringPipeline {
    weighted_scorer: WeightedScorer,
    diversity_scorer: AuthorDiversityScorer,
    oon_scorer: OONScorer,
}

impl ScoringPipeline {
    /// Run the full 4-stage scoring pipeline
    pub fn score(&self, candidates: &mut [PostCandidate]) {
        // Stage 1: Phoenix scores already computed (or heuristic)
        // Stage 2: Compute weighted scores
        for candidate in candidates.iter_mut() {
            candidate.weighted_score = self.weighted_scorer.score(
                &candidate.phoenix_scores,
                candidate.video_duration,
            );
        }

        // Sort by weighted score for diversity calculation
        candidates.sort_by(|a, b| b.weighted_score.partial_cmp(&a.weighted_score).unwrap());

        // Stage 3: Apply author diversity
        self.diversity_scorer.score(candidates);

        // Stage 4: Apply OON penalties
        for candidate in candidates.iter_mut() {
            self.oon_scorer.score(candidate, candidate.is_oon);
        }

        // Final sort by score
        candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    }
}
```

#### 2.6 Tasks

- [x] Add 4 new action types to ActionProbs
- [x] Create `simulator/src/scoring/` module structure
- [x] Implement WeightedScorer with configurable weights
- [x] Implement AuthorDiversityScorer
- [x] Implement OONScorer
- [x] Create ScoringPipeline combining all scorers
- [x] Add configuration file support (`config/scoring.toml`)
- [x] Update API to expose individual stage scores
- [x] Update webapp to show stage-by-stage breakdown

---

### Phase 3: User Context and History

**Goal:** Enable personalized scoring by supporting user engagement history.

#### 3.1 User Profile Store

Create a simple user profile storage:

```rust
// simulator/src/user/mod.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: String,
    pub followers: u64,
    pub following: u64,
    pub account_age_days: u32,
    pub verified: bool,
    pub engagement_history: Vec<EngagementEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementEvent {
    pub post_id: String,
    pub author_id: String,
    pub timestamp: i64,
    pub actions: ActionFlags,  // Which actions the user took
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionFlags {
    pub liked: bool,
    pub replied: bool,
    pub reposted: bool,
    pub quoted: bool,
    pub clicked: bool,
    pub shared: bool,
    pub followed_author: bool,
    pub blocked: bool,
    pub muted: bool,
    pub reported: bool,
}
```

#### 3.2 History-Aware Phoenix Requests

When user history is available, include it in Phoenix requests:

```python
# phoenix/service/inference.py

def build_ranking_request(
    user_profile: UserProfile,
    candidates: List[PostFeatures],
    history_limit: int = 50,
) -> RecsysBatch:
    """Build a batch for Phoenix model with user history."""

    # Convert engagement history to model format
    history = user_profile.engagement_history[-history_limit:]

    history_post_hashes = [hash_post(e.post_id) for e in history]
    history_author_hashes = [hash_author(e.author_id) for e in history]
    history_actions = [actions_to_vector(e.actions) for e in history]

    return RecsysBatch(
        user_hashes=hash_user(user_profile.user_id),
        history_post_hashes=pad_sequence(history_post_hashes, history_limit),
        history_author_hashes=pad_sequence(history_author_hashes, history_limit),
        history_actions=pad_sequence(history_actions, history_limit),
        history_product_surface=np.zeros(history_limit),
        candidate_post_hashes=[hash_post(c.post_id) for c in candidates],
        candidate_author_hashes=[hash_author(c.author_id) for c in candidates],
        candidate_product_surface=np.zeros(len(candidates)),
    )
```

#### 3.3 Synthetic History Generation

For users without real history, generate synthetic history based on their profile:

```rust
// simulator/src/user/synthetic.rs

pub fn generate_synthetic_history(
    profile: &UserProfile,
    seed: u64,
) -> Vec<EngagementEvent> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut history = Vec::new();

    // Generate based on profile characteristics
    let engagement_rate = estimate_engagement_rate(profile);
    let action_distribution = estimate_action_distribution(profile);

    // Create synthetic engagement sequence
    for i in 0..50 {
        if rng.gen::<f64>() < engagement_rate {
            history.push(EngagementEvent {
                post_id: format!("synthetic_{}", i),
                author_id: format!("author_{}", rng.gen_range(0..100)),
                timestamp: Utc::now().timestamp() - (i as i64 * 3600),
                actions: sample_actions(&action_distribution, &mut rng),
            });
        }
    }

    history
}
```

#### 3.4 Tasks

- [x] Create `simulator/src/user/` module
- [x] Implement UserProfile and EngagementEvent structs
- [x] Add user profile storage (JSON file initially)
- [x] Update Phoenix service to accept user history
- [x] Implement synthetic history generation
- [x] Add API endpoint for user profile management
- [x] Update webapp with "Simulate As User" feature

---

### Phase 4: Calibration and Validation

**Goal:** Calibrate the simulator against real-world data and validate accuracy.

#### 4.1 Calibration Dataset

Create a framework for collecting calibration data:

```rust
// simulator/src/calibration/mod.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationSample {
    pub post_id: String,
    pub post_text: String,
    pub author_followers: u64,
    pub media_type: MediaType,
    pub actual_impressions: u64,
    pub actual_likes: u64,
    pub actual_replies: u64,
    pub actual_reposts: u64,
    // ... other metrics
}

pub struct CalibrationRunner {
    samples: Vec<CalibrationSample>,
}

impl CalibrationRunner {
    pub fn compute_metrics(&self, simulator: &Simulator) -> CalibrationMetrics {
        let mut metrics = CalibrationMetrics::default();

        for sample in &self.samples {
            let predicted = simulator.simulate(&sample.to_input());

            // Compare predicted vs actual
            metrics.update(sample, &predicted);
        }

        metrics.finalize()
    }
}
```

#### 4.2 Weight Tuning

Implement weight optimization based on calibration data:

```rust
// simulator/src/calibration/tuning.rs

pub struct WeightTuner {
    calibration_data: Vec<CalibrationSample>,
}

impl WeightTuner {
    /// Find weights that minimize prediction error
    pub fn tune(&self, initial_weights: ActionWeights) -> ActionWeights {
        // Use gradient-free optimization (e.g., Nelder-Mead)
        // Objective: minimize RMSE between predicted and actual engagement rates

        let objective = |weights: &ActionWeights| -> f64 {
            let mut total_error = 0.0;
            for sample in &self.calibration_data {
                let predicted = simulate_with_weights(sample, weights);
                let actual = sample.engagement_rate();
                total_error += (predicted - actual).powi(2);
            }
            (total_error / self.calibration_data.len() as f64).sqrt()
        };

        optimize(objective, initial_weights)
    }
}
```

#### 4.3 Validation Metrics

Define metrics to track simulator accuracy:

```rust
pub struct CalibrationMetrics {
    // Correlation between predicted and actual
    pub impression_correlation: f64,
    pub engagement_rate_correlation: f64,

    // Mean absolute error
    pub like_rate_mae: f64,
    pub reply_rate_mae: f64,
    pub repost_rate_mae: f64,

    // Ranking accuracy (for comparing drafts)
    pub pairwise_ranking_accuracy: f64,  // % of pairs ranked correctly
}
```

#### 4.4 Tasks

- [x] Define CalibrationSample schema
- [x] Create calibration data collection guide
- [x] Implement CalibrationRunner
- [x] Implement WeightTuner with optimization
- [x] Add CLI command for calibration: `cargo run -- calibrate --data calibration.json`
- [x] Create validation report generator
- [x] Document calibration methodology

---

## Configuration

### Environment Variables

```bash
# Phoenix model service
PHOENIX_ENDPOINT=http://localhost:8000
PHOENIX_TIMEOUT_MS=5000

# Scoring mode
SCORING_MODE=hybrid  # heuristic, phoenix, hybrid
PHOENIX_WEIGHT=0.7   # For hybrid mode

# xAI/Grok LLM (existing)
XAI_API_KEY=...
XAI_MODEL=grok-2-latest

# User data
USER_PROFILES_PATH=data/user_profiles.json

# Calibration
CALIBRATION_DATA_PATH=data/calibration.json
```

### Configuration File

```toml
# config/scoring.toml

[scoring]
mode = "hybrid"
phoenix_weight = 0.7

[weights]
favorite = 1.0
reply = 1.6
repost = 2.0
# ... etc

[diversity]
decay = 0.7
floor = 0.1

[oon]
multiplier = 0.8

[phoenix]
endpoint = "http://localhost:8000"
timeout_ms = 5000
history_limit = 50
```

---

## API Changes

### New Endpoints

```
POST /api/simulate/phoenix
  - Uses Phoenix model for scoring
  - Requires PHOENIX_ENDPOINT

POST /api/simulate/compare
  - Compare multiple drafts
  - Returns ranked list with score breakdowns

GET /api/config
  - Returns current scoring configuration

PUT /api/config/weights
  - Update scoring weights

POST /api/users
  - Create/update user profile

GET /api/users/{user_id}/history
  - Get user's engagement history
```

### Enhanced Response

```json
{
  "score": 72.5,
  "tier": "High",
  "scoring_mode": "hybrid",
  "phoenix_actions": {
    "like": 0.15,
    "reply": 0.08,
    "repost": 0.12,
    "...": "..."
  },
  "weighted_score": 1.45,
  "diversity_multiplier": 1.0,
  "oon_multiplier": 1.0,
  "final_score": 1.45,
  "impressions_in": 450,
  "impressions_oon": 1200,
  "impressions_total": 1650,
  "expected_unique_engagements": 42,
  "expected_action_volume": 65,
  "signals": { "..." },
  "suggestions": ["..."]
}
```

---

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Phoenix model too slow for interactive use | Medium | High | Add caching, async scoring, or use smaller model config |
| JAX/CUDA compatibility issues | Medium | Medium | Document requirements, add Docker container |
| Calibration data hard to collect | High | Medium | Start with synthetic data, add X API integration later |
| Production weights significantly different | High | Medium | Expose weights as configurable, support A/B testing |
| Memory usage too high for local dev | Low | Medium | Add model size configuration, support CPU-only mode |

---

## Success Criteria

### Phase 1 Complete When:
- [x] Phoenix service runs and responds to requests
- [x] Simulator can optionally use Phoenix for scoring
- [ ] Latency < 500ms for single-post scoring

### Phase 2 Complete When:
- [x] All 19 action types supported
- [x] 4-stage pipeline implemented
- [x] Webapp shows stage-by-stage breakdown

### Phase 3 Complete When:
- [x] User profiles can be created and stored
- [x] Phoenix requests include user history
- [x] Scoring differs meaningfully for different user profiles

### Phase 4 Complete When:
- [x] Calibration framework functional
- [ ] At least 100 calibration samples collected
- [ ] Pairwise ranking accuracy > 70%

---

## Appendix A: Phoenix Model Input Format

From `phoenix/runners.py` and `phoenix/recsys_model.py`:

```python
@dataclass
class RecsysBatch:
    user_hashes: Array           # [B, num_user_hashes]
    history_post_hashes: Array   # [B, history_len, num_item_hashes]
    history_author_hashes: Array # [B, history_len, num_author_hashes]
    history_actions: Array       # [B, history_len, num_actions]
    history_product_surface: Array # [B, history_len]
    candidate_post_hashes: Array # [B, num_candidates, num_item_hashes]
    candidate_author_hashes: Array # [B, num_candidates, num_author_hashes]
    candidate_product_surface: Array # [B, num_candidates]

# Default config (from run_ranker.py)
HashConfig(
    num_user_hashes=2,
    num_item_hashes=2,
    num_author_hashes=2,
)

PhoenixModelConfig(
    emb_size=128,
    vocab_size=100_000,
    history_seq_len=50,
    candidate_seq_len=10,
    num_actions=19,
)
```

## Appendix B: Action Type Mapping

| Simulator (current) | Production (target) | Notes |
|--------------------|---------------------|-------|
| like | favorite_score | Direct map |
| reply | reply_score | Direct map |
| repost | repost_score | Direct map |
| quote | quote_score | Direct map |
| click | click_score | Direct map |
| profile_click | profile_click_score | Direct map |
| video_view | vqv_score | Video Quality View |
| photo_expand | photo_expand_score | Direct map |
| share | share_score | Direct map |
| share_dm | share_via_dm_score | Direct map |
| share_link | share_via_copy_link_score | Direct map |
| dwell | dwell_score | Direct map |
| quoted_click | quoted_click_score | Direct map |
| follow_author | follow_author_score | Direct map |
| not_interested | not_interested_score | Direct map |
| block | block_author_score | Direct map |
| mute | mute_author_score | Direct map |
| report | report_score | Direct map |
| dwell_time | dwell_time | Continuous (seconds) |

## Appendix C: File Structure After Implementation

```
simulator/
├── src/
│   ├── lib.rs              # Core simulation (updated)
│   ├── llm.rs              # Grok LLM scoring
│   ├── server.rs           # API server (updated)
│   ├── phoenix_client.rs   # NEW: Phoenix HTTP client
│   ├── scoring/
│   │   ├── mod.rs          # NEW: Scoring module
│   │   ├── weighted.rs     # NEW: Weighted scorer
│   │   ├── diversity.rs    # NEW: Author diversity
│   │   ├── oon.rs          # NEW: OON scorer
│   │   └── pipeline.rs     # NEW: Unified pipeline
│   ├── user/
│   │   ├── mod.rs          # NEW: User module
│   │   ├── profile.rs      # NEW: User profiles
│   │   └── synthetic.rs    # NEW: Synthetic history
│   └── calibration/
│       ├── mod.rs          # NEW: Calibration module
│       ├── runner.rs       # NEW: Calibration runner
│       └── tuning.rs       # NEW: Weight tuning
├── config/
│   └── scoring.toml        # NEW: Scoring configuration

phoenix/
├── service/
│   ├── __init__.py         # NEW
│   ├── server.py           # NEW: FastAPI server
│   ├── inference.py        # NEW: Model inference
│   └── schema.py           # NEW: Request/response types
├── recsys_model.py         # Existing
├── runners.py              # Existing
└── ...
```

---

## Next Steps

1. **Immediate**: Collect calibration samples and validate metrics at scale.
2. **Short-term**: Integrate candidate retrieval (Thunder + Phoenix retrieval) with real candidates.
3. **Short-term**: Implement core hydrators/filters to mirror production eligibility rules.
4. **Medium-term**: Validate latency/throughput targets for Phoenix service + simulator.
5. **Ongoing**: Tune weights against calibration data and revisit score normalization.

---

*Last updated: 2026-01-21*
*Author: Claude (automated analysis)*
