use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    routing::{get, post},
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
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::{broadcast, Mutex};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tower_http::services::{ServeDir, ServeFile};

use crate::api::{ApiSimulationRequest, ApiSimulationResponse};
use crate::llm::{prompt_for_text, LlmClient};
use crate::snapshots::{Snapshot, SnapshotStore};
use crate::x_api::{XApiClient, XUserProfile};
use virality_sim::simulate_with_llm;

#[derive(Clone)]
struct AppState {
    llm_client: Option<LlmClient>,
    x_client: Option<XApiClient>,
    channels: Arc<Mutex<HashMap<String, broadcast::Sender<StreamEvent>>>>,
    snapshots: Arc<SnapshotStore>,
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

static REQUEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub async fn serve(args: crate::ServeArgs) -> Result<(), String> {
    let snapshot_path = snapshot_path();
    let snapshot_store = SnapshotStore::load(snapshot_path).await?;
    let state = AppState {
        llm_client: LlmClient::from_env(None),
        x_client: XApiClient::from_env(),
        channels: Arc::new(Mutex::new(HashMap::new())),
        snapshots: Arc::new(snapshot_store),
    };

    let web_root = args.web_root;
    let index_path = format!("{}/index.html", web_root.trim_end_matches('/'));
    let static_service = ServeDir::new(web_root).not_found_service(ServeFile::new(index_path));

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/simulate", post(simulate_handler))
        .route("/api/simulate/stream", get(stream_handler))
        .route("/api/x/profile", get(x_profile_handler))
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
    let input = request.into_input().map_err(|err| (StatusCode::BAD_REQUEST, err))?;
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

    let output = simulate_with_llm(
        &input,
        llm_result.as_ref().map(|result| &result.score),
        llm_result.as_ref().map(|result| &result.trace),
    );
    if let Some(sender) = channel.as_ref() {
        send_event(sender, "done", "Simulation complete");
        schedule_cleanup(state.channels.clone(), request_id.clone());
    }

    let response = ApiSimulationResponse::from_output(output, warnings, request_id);
    Ok(Json(response))
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

    let profile = client
        .fetch_user_by_username(username)
        .await
        .map_err(|err| (StatusCode::BAD_GATEWAY, err))?;

    Ok(Json(profile))
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

fn chrono_like_timestamp() -> String {
    now_ms().to_string()
}
