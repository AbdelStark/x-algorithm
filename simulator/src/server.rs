use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, routing::post, Json, Router};
use std::net::SocketAddr;
use tower_http::services::{ServeDir, ServeFile};

use crate::api::{ApiSimulationRequest, ApiSimulationResponse};
use crate::llm::LlmClient;
use virality_sim::simulate_with_llm;

#[derive(Clone)]
struct AppState {
    llm_client: Option<LlmClient>,
}

pub async fn serve(args: crate::ServeArgs) -> Result<(), String> {
    let state = AppState {
        llm_client: LlmClient::from_env(None),
    };

    let web_root = args.web_root;
    let index_path = format!("{}/index.html", web_root.trim_end_matches('/'));
    let static_service = ServeDir::new(web_root).not_found_service(ServeFile::new(index_path));

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/simulate", post(simulate_handler))
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
    let input = request.into_input().map_err(|err| (StatusCode::BAD_REQUEST, err))?;

    let mut warnings = Vec::new();
    let llm_result = if use_ai {
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

    let output = simulate_with_llm(
        &input,
        llm_result.as_ref().map(|result| &result.score),
        llm_result.as_ref().map(|result| &result.trace),
    );
    let response = ApiSimulationResponse::from_output(output, warnings);
    Ok(Json(response))
}
