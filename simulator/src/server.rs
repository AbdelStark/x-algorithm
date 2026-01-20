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
use serde::Serialize;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::{broadcast, Mutex};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tower_http::services::{ServeDir, ServeFile};

use crate::api::{ApiSimulationRequest, ApiSimulationResponse};
use crate::llm::LlmClient;
use virality_sim::simulate_with_llm;

#[derive(Clone)]
struct AppState {
    llm_client: Option<LlmClient>,
    channels: Arc<Mutex<HashMap<String, broadcast::Sender<StreamEvent>>>>,
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

static REQUEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub async fn serve(args: crate::ServeArgs) -> Result<(), String> {
    let state = AppState {
        llm_client: LlmClient::from_env(None),
        channels: Arc::new(Mutex::new(HashMap::new())),
    };

    let web_root = args.web_root;
    let index_path = format!("{}/index.html", web_root.trim_end_matches('/'));
    let static_service = ServeDir::new(web_root).not_found_service(ServeFile::new(index_path));

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/simulate", post(simulate_handler))
        .route("/api/simulate/stream", get(stream_handler))
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

    let mut warnings = Vec::new();
    let llm_result = if use_ai {
        if let Some(sender) = channel.as_ref() {
            send_event(sender, "start", "Preparing Grok prompt");
        }
        match &state.llm_client {
            Some(client) => {
                if let Some(sender) = channel.as_ref() {
                    send_event(sender, "calling", "Calling Grok API");
                }
                match client.score_text(&input.text).await {
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

async fn get_or_create_channel(
    state: &AppState,
    request_id: &str,
) -> broadcast::Sender<StreamEvent> {
    let mut guard = state.channels.lock().await;
    if let Some(sender) = guard.get(request_id) {
        return sender.clone();
    }
    let (sender, _) = broadcast::channel(32);
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

fn generate_request_id() -> String {
    let counter = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("req-{}-{}", now_ms(), counter)
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}
