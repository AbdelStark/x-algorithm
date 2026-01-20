use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tower_http::services::{ServeDir, ServeFile};

use crate::api::{ApiSimulationRequest, ApiSimulationResponse};
use crate::llm::{prompt_for_text, LlmClient};
use crate::snapshots::{Snapshot, SnapshotStore};
use crate::x_api::{XApiClient, XUserProfile};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::RngCore;
use sha2::{Digest, Sha256};
use virality_sim::config::ScoringConfig;
use virality_sim::phoenix_client::{PhoenixClient, PostFeatures, RankingRequest};
use virality_sim::scoring::ActionWeights;
use virality_sim::user::{
    action_flags_to_vector, generate_synthetic_history, profile_from_input, EngagementEvent,
    UserProfile, UserProfileStore,
};
use virality_sim::{simulate_with_mode, ActionProbs, MediaType, ScoringMode, SimulatorInput};

#[derive(Clone)]
struct AppState {
    llm_client: Option<LlmClient>,
    x_client: Option<XApiClient>,
    x_user_token: Arc<Mutex<Option<XUserToken>>>,
    oauth_state: Arc<Mutex<HashMap<String, String>>>,
    channels: Arc<Mutex<HashMap<String, broadcast::Sender<StreamEvent>>>>,
    snapshots: Arc<SnapshotStore>,
    user_profiles: Arc<UserProfileStore>,
    scoring_config: Arc<RwLock<ScoringConfig>>,
    scoring_config_path: Option<PathBuf>,
}

#[derive(Clone, Serialize)]
struct StreamEvent {
    event: String,
    message: String,
    timestamp_ms: u128,
}

#[derive(serde::Deserialize)]
struct StreamQuery {
    request_id: String,
}

#[derive(Deserialize)]
struct XProfileQuery {
    username: String,
}

#[derive(Serialize)]
struct XOAuthStartResponse {
    auth_url: String,
}

#[derive(Deserialize)]
struct XOAuthCallbackQuery {
    state: Option<String>,
    code: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Serialize)]
struct XOAuthStatus {
    enabled: bool,
    connected: bool,
}

#[derive(Clone)]
struct XUserToken {
    access_token: String,
    expires_at: Instant,
}

#[derive(Clone, Copy)]
enum OAuthAuthMode {
    Basic,
    Body,
}

static REQUEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub async fn serve(args: crate::ServeArgs) -> Result<(), String> {
    let snapshot_path = snapshot_path();
    let snapshot_store = SnapshotStore::load(snapshot_path).await?;
    let profiles_path = user_profiles_path();
    let user_profiles = UserProfileStore::load(profiles_path).await?;
    let (scoring_config, scoring_config_path) = ScoringConfig::load(None)?;
    let state = AppState {
        llm_client: LlmClient::from_env(None),
        x_client: XApiClient::from_env(),
        x_user_token: Arc::new(Mutex::new(None)),
        oauth_state: Arc::new(Mutex::new(HashMap::new())),
        channels: Arc::new(Mutex::new(HashMap::new())),
        snapshots: Arc::new(snapshot_store),
        user_profiles: Arc::new(user_profiles),
        scoring_config: Arc::new(RwLock::new(scoring_config)),
        scoring_config_path,
    };

    let web_root = args.web_root;
    let index_path = format!("{}/index.html", web_root.trim_end_matches('/'));
    let static_service = ServeDir::new(web_root).not_found_service(ServeFile::new(index_path));

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/simulate", post(simulate_handler))
        .route("/api/simulate/phoenix", post(simulate_phoenix_handler))
        .route("/api/simulate/compare", post(compare_handler))
        .route("/api/simulate/stream", get(stream_handler))
        .route("/api/config", get(get_config))
        .route("/api/config/weights", put(update_weights))
        .route("/api/users", post(upsert_user))
        .route("/api/users/:user_id/history", get(get_user_history))
        .route("/api/x/profile", get(x_profile_handler))
        .route("/api/x/me", get(x_me_handler))
        .route("/api/x/oauth/start", get(x_oauth_start))
        .route("/api/x/oauth/callback", get(x_oauth_callback))
        .route("/api/x/oauth/status", get(x_oauth_status))
        .route("/api/snapshots", get(list_snapshots).post(create_snapshot))
        .route("/api/snapshots/:id", get(get_snapshot).delete(delete_snapshot))
        .nest_service("/", static_service)
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", args.host, args.port)
        .parse()
        .map_err(|err| format!("invalid bind address: {}", err))?;

    axum::serve(tokio::net::TcpListener::bind(addr).await.map_err(|err| {
        format!("failed to bind server: {}", err)
    })?, app)
    .await
    .map_err(|err| format!("server error: {}", err))?;

    Ok(())
}

async fn health() -> impl IntoResponse {
    StatusCode::OK
}

async fn simulate_handler(
    State(state): State<AppState>,
    Json(request): Json<ApiSimulationRequest>,
) -> Result<Json<ApiSimulationResponse>, (StatusCode, String)> {
    let use_ai = request.use_ai.unwrap_or(false);
    let request_id = request
        .request_id
        .clone()
        .unwrap_or_else(generate_request_id);
    let mut input = request.into_input().map_err(|err| (StatusCode::BAD_REQUEST, err))?;
    let channel = if use_ai {
        Some(get_or_create_channel(&state, &request_id).await)
    } else {
        None
    };
    let mut progress_done: Option<Arc<AtomicBool>> = None;

    let mut warnings = Vec::new();
    let llm_result = if use_ai {
        if let Some(sender) = channel.as_ref() {
            send_event(sender, "start", "Preparing Grok prompt");
        }
        match &state.llm_client {
            Some(client) => {
                if let Some(sender) = channel.as_ref() {
                    let prompt = prompt_for_text(&input.text);
                    send_event(sender, "prompt", &prompt);
                    send_event(sender, "calling", "Calling Grok API");
                    progress_done = Some(start_progress(sender.clone()));
                }
                let result = if let Some(sender) = channel.as_ref() {
                    let token_sender = sender.clone();
                    client
                        .score_text_stream(&input.text, |chunk| {
                            send_event(&token_sender, "token", chunk);
                        })
                        .await
                } else {
                    client.score_text(&input.text).await
                };
                match result {
                    Ok(result) => {
                        if let Some(sender) = channel.as_ref() {
                            send_event(sender, "received", "Received Grok response");
                        }
                        Some(result)
                    }
                    Err(err) => {
                        warnings.push(format!("AI scoring failed: {}", err));
                        if let Some(sender) = channel.as_ref() {
                            send_event(sender, "error", "Grok call failed");
                        }
                        None
                    }
                }
            }
            None => {
                warnings.push("AI scoring not configured: set XAI_API_KEY".to_string());
                if let Some(sender) = channel.as_ref() {
                    send_event(sender, "error", "AI scoring not configured");
                }
                None
            }
        }
    } else {
        None
    };

    if let Some(done_flag) = progress_done {
        done_flag.store(true, Ordering::Relaxed);
    }

    if let Some(sender) = channel.as_ref() {
        send_event(sender, "merge", "Merging Grok signals into model");
    }

    let scoring_config = state.scoring_config.read().await.clone();
    let mut scoring_mode = resolve_scoring_mode(&request, &scoring_config, &mut warnings);

    let mut phoenix_actions = None;
    if matches!(scoring_mode, ScoringMode::Phoenix | ScoringMode::Hybrid { .. }) {
        if let Some(sender) = channel.as_ref() {
            send_event(sender, "phoenix", "Calling Phoenix service");
        }
        match fetch_phoenix_actions(&state, &scoring_config, &mut input, &request).await {
            Ok(actions) => phoenix_actions = Some(actions),
            Err(err) => warnings.push(format!("Phoenix scoring failed: {}", err)),
        }
    }

    if phoenix_actions.is_none()
        && matches!(scoring_mode, ScoringMode::Phoenix | ScoringMode::Hybrid { .. })
    {
        warnings.push("Phoenix unavailable; falling back to heuristic scoring.".to_string());
        scoring_mode = ScoringMode::Heuristic;
    }

    let output = simulate_with_mode(
        &input,
        llm_result.as_ref().map(|result| &result.score),
        llm_result.as_ref().map(|result| &result.trace),
        scoring_mode,
        phoenix_actions.as_ref(),
        &scoring_config,
    );
    if let Some(sender) = channel.as_ref() {
        send_event(sender, "done", "Simulation complete");
        schedule_cleanup(state.channels.clone(), request_id.clone());
    }

    let response = ApiSimulationResponse::from_output(output, warnings, request_id);
    Ok(Json(response))
}

async fn simulate_phoenix_handler(
    State(state): State<AppState>,
    Json(mut request): Json<ApiSimulationRequest>,
) -> Result<Json<ApiSimulationResponse>, (StatusCode, String)> {
    request.scoring_mode = Some("phoenix".to_string());
    simulate_handler(State(state), Json(request)).await
}

#[derive(Deserialize)]
struct CompareRequest {
    candidates: Vec<ApiSimulationRequest>,
    scoring_mode: Option<String>,
    phoenix_weight: Option<f64>,
    user_id: Option<String>,
}

#[derive(Serialize)]
struct CompareResult {
    rank: usize,
    response: ApiSimulationResponse,
}

#[derive(Serialize)]
struct CompareResponse {
    results: Vec<CompareResult>,
}

async fn compare_handler(
    State(state): State<AppState>,
    Json(request): Json<CompareRequest>,
) -> Result<Json<CompareResponse>, (StatusCode, String)> {
    if request.candidates.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "candidates are required".to_string()));
    }

    let scoring_config = state.scoring_config.read().await.clone();
    let mut responses = Vec::new();

    for mut candidate in request.candidates {
        if candidate.scoring_mode.is_none() {
            candidate.scoring_mode = request.scoring_mode.clone();
        }
        if candidate.phoenix_weight.is_none() {
            candidate.phoenix_weight = request.phoenix_weight;
        }
        if candidate.user_id.is_none() {
            candidate.user_id = request.user_id.clone();
        }

        let mut warnings = Vec::new();
        let mut input =
            candidate.into_input().map_err(|err| (StatusCode::BAD_REQUEST, err))?;

        let llm_result = if candidate.use_ai.unwrap_or(false) {
            match &state.llm_client {
                Some(client) => match client.score_text(&input.text).await {
                    Ok(result) => Some(result),
                    Err(err) => {
                        warnings.push(format!("AI scoring failed: {}", err));
                        None
                    }
                },
                None => {
                    warnings.push("AI scoring not configured: set XAI_API_KEY".to_string());
                    None
                }
            }
        } else {
            None
        };

        let mut scoring_mode = resolve_scoring_mode(&candidate, &scoring_config, &mut warnings);
        let mut phoenix_actions = None;
        if matches!(scoring_mode, ScoringMode::Phoenix | ScoringMode::Hybrid { .. }) {
            match fetch_phoenix_actions(&state, &scoring_config, &mut input, &candidate).await {
                Ok(actions) => phoenix_actions = Some(actions),
                Err(err) => warnings.push(format!("Phoenix scoring failed: {}", err)),
            }
        }

        if phoenix_actions.is_none()
            && matches!(scoring_mode, ScoringMode::Phoenix | ScoringMode::Hybrid { .. })
        {
            warnings.push("Phoenix unavailable; falling back to heuristic scoring.".to_string());
            scoring_mode = ScoringMode::Heuristic;
        }

        let output = simulate_with_mode(
            &input,
            llm_result.as_ref().map(|result| &result.score),
            llm_result.as_ref().map(|result| &result.trace),
            scoring_mode,
            phoenix_actions.as_ref(),
            &scoring_config,
        );

        let request_id = candidate
            .request_id
            .clone()
            .unwrap_or_else(generate_request_id);
        let response = ApiSimulationResponse::from_output(output, warnings, request_id);
        responses.push(response);
    }

    responses.sort_by(|a, b| {
        b.final_score
            .partial_cmp(&a.final_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let results = responses
        .into_iter()
        .enumerate()
        .map(|(idx, response)| CompareResult {
            rank: idx + 1,
            response,
        })
        .collect();

    Ok(Json(CompareResponse { results }))
}

async fn get_config(State(state): State<AppState>) -> Json<ScoringConfig> {
    let config = state.scoring_config.read().await.clone();
    Json(config)
}

#[derive(Deserialize)]
struct UpdateWeightsRequest {
    weights: ActionWeights,
}

async fn update_weights(
    State(state): State<AppState>,
    Json(request): Json<UpdateWeightsRequest>,
) -> Result<Json<ScoringConfig>, (StatusCode, String)> {
    let mut config = state.scoring_config.write().await;
    config.weights = request.weights.clone();

    if let Some(path) = state.scoring_config_path.as_ref() {
        config
            .write(path)
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))?;
    }

    Ok(Json(config.clone()))
}

#[derive(Deserialize)]
struct UserProfileRequest {
    user_id: String,
    followers: Option<u64>,
    following: Option<u64>,
    account_age_days: Option<u32>,
    verified: Option<bool>,
    engagement_history: Option<Vec<EngagementEvent>>,
    generate_synthetic_history: Option<bool>,
    synthetic_seed: Option<u64>,
}

async fn upsert_user(
    State(state): State<AppState>,
    Json(request): Json<UserProfileRequest>,
) -> Result<Json<UserProfile>, (StatusCode, String)> {
    let user_id = request.user_id.trim();
    if user_id.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "user_id is required".to_string()));
    }

    let existing = state.user_profiles.get(user_id).await;
    let mut profile = existing.unwrap_or(UserProfile {
        user_id: user_id.to_string(),
        followers: request.followers.unwrap_or(0),
        following: request.following.unwrap_or(0),
        account_age_days: request.account_age_days.unwrap_or(0),
        verified: request.verified.unwrap_or(false),
        engagement_history: Vec::new(),
    });

    if let Some(followers) = request.followers {
        profile.followers = followers;
    }
    if let Some(following) = request.following {
        profile.following = following;
    }
    if let Some(account_age_days) = request.account_age_days {
        profile.account_age_days = account_age_days;
    }
    if let Some(verified) = request.verified {
        profile.verified = verified;
    }
    if let Some(history) = request.engagement_history {
        profile.engagement_history = history;
    }
    if request.generate_synthetic_history.unwrap_or(false) {
        let seed = request
            .synthetic_seed
            .unwrap_or_else(|| stable_hash64(user_id));
        profile.engagement_history = generate_synthetic_history(&profile, seed);
    }

    let saved = state
        .user_profiles
        .upsert(profile)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))?;

    Ok(Json(saved))
}

async fn get_user_history(
    State(state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> Result<Json<Vec<EngagementEvent>>, (StatusCode, String)> {
    match state.user_profiles.get(&user_id).await {
        Some(profile) => Ok(Json(profile.engagement_history)),
        None => Err((StatusCode::NOT_FOUND, "user not found".to_string())),
    }
}

fn resolve_scoring_mode(
    request: &ApiSimulationRequest,
    config: &ScoringConfig,
    warnings: &mut Vec<String>,
) -> ScoringMode {
    let mut mode = config.scoring.to_mode();

    if let Some(raw_mode) = request.scoring_mode.as_deref() {
        match raw_mode.to_lowercase().as_str() {
            "heuristic" => mode = ScoringMode::Heuristic,
            "phoenix" => mode = ScoringMode::Phoenix,
            "hybrid" => {
                let weight = request
                    .phoenix_weight
                    .unwrap_or(config.scoring.phoenix_weight)
                    .clamp(0.0, 1.0);
                mode = ScoringMode::Hybrid {
                    phoenix_weight: weight,
                };
            }
            _ => warnings.push(format!(
                "Unknown scoring_mode '{}'; using config default.",
                raw_mode
            )),
        }
    }

    if let Some(weight) = request.phoenix_weight {
        if matches!(mode, ScoringMode::Hybrid { .. }) {
            mode = ScoringMode::Hybrid {
                phoenix_weight: weight.clamp(0.0, 1.0),
            };
        }
    }

    mode
}

async fn fetch_phoenix_actions(
    state: &AppState,
    scoring_config: &ScoringConfig,
    input: &mut SimulatorInput,
    request: &ApiSimulationRequest,
) -> Result<ActionProbs, String> {
    let client = PhoenixClient::from_config(scoring_config)?;
    let (post_id, author_id) = ensure_candidate_ids(input);
    let candidate = build_post_features(input, &post_id, &author_id);

    let user_id = derive_user_id(input, request);
    let (history_posts, history_actions) =
        build_history(state, &user_id, input, scoring_config.phoenix.history_limit).await?;
    let ranking_request = RankingRequest {
        user_id,
        user_embedding: None,
        history_posts,
        history_actions,
        candidates: vec![candidate],
    };

    let response = client.score(ranking_request).await?;
    let score = response
        .scores
        .iter()
        .find(|candidate| candidate.post_id == post_id)
        .or_else(|| response.scores.first())
        .ok_or_else(|| "Phoenix response missing scores".to_string())?;
    Ok(score.phoenix_scores.clone())
}

async fn build_history(
    state: &AppState,
    user_id: &str,
    input: &SimulatorInput,
    history_limit: usize,
) -> Result<(Vec<PostFeatures>, Vec<Vec<f32>>), String> {
    let mut profile = state.user_profiles.get(user_id).await;
    let seed = stable_hash64(user_id);

    if profile.is_none() {
        let mut new_profile = profile_from_input(user_id.to_string(), input);
        new_profile.engagement_history = generate_synthetic_history(&new_profile, seed);
        state.user_profiles.upsert(new_profile.clone()).await?;
        profile = Some(new_profile);
    }

    let mut profile = profile.expect("profile must be initialized");
    if profile.engagement_history.is_empty() {
        profile.engagement_history = generate_synthetic_history(&profile, seed);
        state.user_profiles.upsert(profile.clone()).await?;
    }

    let history = if profile.engagement_history.len() > history_limit {
        let start = profile.engagement_history.len() - history_limit;
        &profile.engagement_history[start..]
    } else {
        &profile.engagement_history[..]
    };

    let history_posts = history
        .iter()
        .map(|event| PostFeatures {
            post_id: event.post_id.clone(),
            author_id: event.author_id.clone(),
            text_hash: stable_hash64(&event.post_id),
            author_hash: stable_hash64(&event.author_id),
            product_surface: 0,
            video_duration_seconds: None,
        })
        .collect();

    let history_actions = history
        .iter()
        .map(|event| action_flags_to_vector(&event.actions))
        .collect();

    Ok((history_posts, history_actions))
}

fn ensure_candidate_ids(input: &mut SimulatorInput) -> (String, String) {
    let post_id = input.post_id.clone().unwrap_or_else(|| {
        let hash = stable_hash64(&input.text);
        format!("post_{:x}", hash)
    });
    if input.post_id.is_none() {
        input.post_id = Some(post_id.clone());
    }

    let author_id = input.author_id.clone().unwrap_or_else(|| {
        let payload = format!(
            "{}:{}:{}:{}",
            input.followers, input.following, input.account_age_days, input.verified
        );
        let hash = stable_hash64(&payload);
        format!("author_{:x}", hash)
    });
    if input.author_id.is_none() {
        input.author_id = Some(author_id.clone());
    }

    (post_id, author_id)
}

fn build_post_features(
    input: &SimulatorInput,
    post_id: &str,
    author_id: &str,
) -> PostFeatures {
    let video_duration_seconds = input.video_duration_seconds.or_else(|| {
        if matches!(input.media, MediaType::Video) {
            Some(15.0)
        } else {
            None
        }
    });

    PostFeatures {
        post_id: post_id.to_string(),
        author_id: author_id.to_string(),
        text_hash: stable_hash64(&input.text),
        author_hash: stable_hash64(author_id),
        product_surface: 0,
        video_duration_seconds,
    }
}

fn derive_user_id(input: &SimulatorInput, request: &ApiSimulationRequest) -> String {
    request
        .user_id
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            let payload = format!(
                "{}:{}:{}",
                input.followers, input.following, input.account_age_days
            );
            format!("user_{:x}", stable_hash64(&payload))
        })
}

fn stable_hash64(value: &str) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&digest[..8]);
    u64::from_be_bytes(bytes)
}

#[derive(Deserialize)]
struct SnapshotRequest {
    id: Option<String>,
    created_at: Option<String>,
    input: serde_json::Value,
    output: serde_json::Value,
}

async fn list_snapshots(State(state): State<AppState>) -> Result<Json<Vec<Snapshot>>, StatusCode> {
    let snapshots = state.snapshots.list().await;
    Ok(Json(snapshots))
}

async fn get_snapshot(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<Snapshot>, (StatusCode, String)> {
    match state.snapshots.get(&id).await {
        Some(snapshot) => Ok(Json(snapshot)),
        None => Err((StatusCode::NOT_FOUND, "Snapshot not found".to_string())),
    }
}

async fn create_snapshot(
    State(state): State<AppState>,
    Json(payload): Json<SnapshotRequest>,
) -> Result<Json<Snapshot>, (StatusCode, String)> {
    let snapshot = Snapshot {
        id: payload.id.unwrap_or_else(generate_snapshot_id),
        created_at: payload
            .created_at
            .unwrap_or_else(|| chrono_like_timestamp()),
        input: payload.input,
        output: payload.output,
    };
    let saved = state
        .snapshots
        .add(snapshot)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))?;
    Ok(Json(saved))
}

async fn delete_snapshot(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let removed = state
        .snapshots
        .delete(&id)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))?;
    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "Snapshot not found".to_string()))
    }
}

async fn stream_handler(
    State(state): State<AppState>,
    Query(query): Query<StreamQuery>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>>, StatusCode>
{
    let sender = get_or_create_channel(&state, &query.request_id).await;
    let receiver = sender.subscribe();
    let stream = BroadcastStream::new(receiver).filter_map(|event| {
        match event {
            Ok(event) => {
                let data = serde_json::to_string(&event).unwrap_or_default();
                Some(Ok(Event::default().data(data)))
            }
            Err(_) => None,
        }
    });

    send_event(&sender, "connected", "Streaming Grok status");
    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(8))))
}

async fn x_profile_handler(
    State(state): State<AppState>,
    Query(query): Query<XProfileQuery>,
) -> Result<Json<XUserProfile>, (StatusCode, String)> {
    let username = query.username.trim();
    if username.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "username is required".to_string()));
    }

    let client = state
        .x_client
        .as_ref()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "X API not configured".to_string()))?;

    if let Some(token) = get_user_token(&state).await {
        let profile = client
            .fetch_user_by_username_with_token(username, &token)
            .await
            .map_err(|err| (StatusCode::BAD_GATEWAY, err))?;
        return Ok(Json(profile));
    }

    let profile = client
        .fetch_user_by_username(username)
        .await
        .map_err(|err| (StatusCode::BAD_GATEWAY, err))?;

    Ok(Json(profile))
}

async fn x_me_handler(
    State(state): State<AppState>,
) -> Result<Json<XUserProfile>, (StatusCode, String)> {
    let client = state
        .x_client
        .as_ref()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "X API not configured".to_string()))?;
    let token = get_user_token(&state)
        .await
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "X OAuth not connected".to_string()))?;

    let profile = client
        .fetch_me_with_token(&token)
        .await
        .map_err(|err| (StatusCode::BAD_GATEWAY, err))?;

    Ok(Json(profile))
}

async fn x_oauth_start(
    State(state): State<AppState>,
) -> Result<Json<XOAuthStartResponse>, (StatusCode, String)> {
    let config = oauth_config().ok_or_else(|| {
        (StatusCode::BAD_REQUEST, "X OAuth not configured".to_string())
    })?;
    let state_value = random_token(16);
    let verifier = random_token(32);
    let challenge = code_challenge(&verifier);

    {
        let mut guard = state.oauth_state.lock().await;
        guard.insert(state_value.clone(), verifier);
    }

    let auth_url = format!(
        "https://twitter.com/i/oauth2/authorize?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
        urlencoding::encode(&config.client_id),
        urlencoding::encode(&config.redirect_uri),
        urlencoding::encode(&config.scope),
        urlencoding::encode(&state_value),
        urlencoding::encode(&challenge),
    );

    Ok(Json(XOAuthStartResponse { auth_url }))
}

async fn x_oauth_callback(
    State(state): State<AppState>,
    Query(query): Query<XOAuthCallbackQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if let Some(error) = query.error {
        let message = query.error_description.unwrap_or(error);
        let redirect = format!("/?x_oauth=error&message={}", urlencoding::encode(&message));
        return Ok(axum::response::Redirect::to(&redirect));
    }

    let code = query
        .code
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing code".to_string()))?;
    let state_value = query
        .state
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing OAuth state".to_string()))?;

    let verifier = {
        let mut guard = state.oauth_state.lock().await;
        guard.remove(&state_value)
    }
    .ok_or_else(|| (StatusCode::BAD_REQUEST, "Invalid OAuth state".to_string()))?;

    let config = oauth_config()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "X OAuth not configured".to_string()))?;

    let token = exchange_oauth_code(&config, &code, &verifier)
        .await
        .map_err(|err| (StatusCode::BAD_GATEWAY, err))?;

    {
        let mut guard = state.x_user_token.lock().await;
        *guard = Some(token);
    }

    Ok(axum::response::Redirect::to("/?x_oauth=success"))
}

async fn x_oauth_status(
    State(state): State<AppState>,
) -> Result<Json<XOAuthStatus>, StatusCode> {
    let enabled = oauth_config().is_some();
    let connected = get_user_token(&state).await.is_some();
    Ok(Json(XOAuthStatus { enabled, connected }))
}

async fn get_or_create_channel(
    state: &AppState,
    request_id: &str,
) -> broadcast::Sender<StreamEvent> {
    let mut guard = state.channels.lock().await;
    if let Some(sender) = guard.get(request_id) {
        return sender.clone();
    }
    let (sender, _) = broadcast::channel(256);
    guard.insert(request_id.to_string(), sender.clone());
    sender
}

fn send_event(sender: &broadcast::Sender<StreamEvent>, event: &str, message: &str) {
    let _ = sender.send(StreamEvent {
        event: event.to_string(),
        message: message.to_string(),
        timestamp_ms: now_ms(),
    });
}

fn schedule_cleanup(channels: Arc<Mutex<HashMap<String, broadcast::Sender<StreamEvent>>>>, request_id: String) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(10)).await;
        let mut guard = channels.lock().await;
        guard.remove(&request_id);
    });
}

fn start_progress(sender: broadcast::Sender<StreamEvent>) -> Arc<AtomicBool> {
    let done = Arc::new(AtomicBool::new(false));
    let done_flag = done.clone();
    tokio::spawn(async move {
        let mut elapsed = 0;
        while !done_flag.load(Ordering::Relaxed) {
            send_event(&sender, "progress", &format!("Waiting on Grok... {}s", elapsed));
            elapsed += 1;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });
    done
}

fn random_token(len: usize) -> String {
    let mut bytes = vec![0u8; len];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let digest = hasher.finalize();
    URL_SAFE_NO_PAD.encode(digest)
}

struct OAuthConfig {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    scope: String,
    token_url: String,
    auth_mode: OAuthAuthMode,
}

fn oauth_config() -> Option<OAuthConfig> {
    let client_id = std::env::var("X_OAUTH_CLIENT_ID").ok()?;
    let client_secret = std::env::var("X_OAUTH_CLIENT_SECRET").ok()?;
    let redirect_uri = std::env::var("X_OAUTH_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:8787/api/x/oauth/callback".to_string());
    let scope = std::env::var("X_OAUTH_SCOPE").unwrap_or_else(|_| "users.read".to_string());
    let token_url = std::env::var("X_OAUTH_TOKEN_URL")
        .unwrap_or_else(|_| "https://api.twitter.com/2/oauth2/token".to_string());
    let auth_mode = match std::env::var("X_OAUTH_AUTH_MODE")
        .unwrap_or_else(|_| "basic".to_string())
        .to_lowercase()
        .as_str()
    {
        "body" => OAuthAuthMode::Body,
        _ => OAuthAuthMode::Basic,
    };
    Some(OAuthConfig {
        client_id,
        client_secret,
        redirect_uri,
        scope,
        token_url,
        auth_mode,
    })
}

async fn exchange_oauth_code(
    config: &OAuthConfig,
    code: &str,
    verifier: &str,
) -> Result<XUserToken, String> {
    let mut params = vec![
        ("grant_type", "authorization_code"),
        ("client_id", config.client_id.as_str()),
        ("code", code),
        ("redirect_uri", config.redirect_uri.as_str()),
        ("code_verifier", verifier),
    ];
    if matches!(config.auth_mode, OAuthAuthMode::Body) {
        params.push(("client_secret", config.client_secret.as_str()));
    }

    let mut request = reqwest::Client::new().post(&config.token_url);
    if matches!(config.auth_mode, OAuthAuthMode::Basic) {
        request = request.basic_auth(&config.client_id, Some(&config.client_secret));
    }

    let response = request
        .form(&params)
        .send()
        .await
        .map_err(|err| format!("X OAuth token request failed: {}", err))?;

    let status = response.status();
    if !status.is_success() {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| String::new());
        let detail = error_body.trim();
        if detail.is_empty() {
            return Err(format!("X OAuth token error: {}", status));
        }
        return Err(format!("X OAuth token error: {} {}", status, detail));
    }

    let body: OAuthTokenResponse = response
        .json()
        .await
        .map_err(|err| format!("X OAuth token parse failed: {}", err))?;

    let expires_in = body.expires_in.unwrap_or(3600);
    let expires_at = Instant::now() + Duration::from_secs(expires_in.saturating_sub(30));

    Ok(XUserToken {
        access_token: body.access_token,
        expires_at,
    })
}

async fn get_user_token(state: &AppState) -> Option<String> {
    let mut guard = state.x_user_token.lock().await;
    if let Some(token) = guard.as_ref() {
        if Instant::now() < token.expires_at {
            return Some(token.access_token.clone());
        }
    }
    guard.take();
    None
}

#[derive(Deserialize)]
struct OAuthTokenResponse {
    access_token: String,
    expires_in: Option<u64>,
}

fn generate_request_id() -> String {
    let counter = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("req-{}-{}", now_ms(), counter)
}

fn generate_snapshot_id() -> String {
    let counter = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("snap-{}-{}", now_ms(), counter)
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn snapshot_path() -> PathBuf {
    if let Ok(path) = std::env::var("SIM_SNAPSHOT_PATH") {
        return PathBuf::from(path);
    }
    PathBuf::from("data").join("snapshots.json")
}

fn user_profiles_path() -> PathBuf {
    if let Ok(path) = std::env::var("USER_PROFILES_PATH") {
        return PathBuf::from(path);
    }
    PathBuf::from("data").join("user_profiles.json")
}

fn chrono_like_timestamp() -> String {
    now_ms().to_string()
}
