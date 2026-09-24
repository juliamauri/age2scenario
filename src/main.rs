mod minimap;
mod scenario;

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Multipart, State},
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use scenario::{ScenarioError, ScenarioInfo};
use serde::Serialize;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;

type ApiError = (StatusCode, Json<ErrorResponse>);
const PARSER_QUEUE_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Clone)]
struct AppState {
    parser_slots: Arc<Semaphore>,
}

struct ScenarioUpload {
    file_name: String,
    bytes: axum::body::Bytes,
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../web/index.html"))
}

async fn healthz() -> &'static str {
    "ok"
}

fn error_response(status: StatusCode, message: &str) -> ApiError {
    (
        status,
        Json(ErrorResponse {
            error: message.to_string(),
        }),
    )
}

fn scenario_error_response(error: ScenarioError) -> ApiError {
    let (status, message) = match &error {
        ScenarioError::ParseFailed(_) => {
            tracing::warn!(
                error = %error,
                "Invalid scenario uploaded"
            );

            (
                StatusCode::BAD_REQUEST,
                "Invalid or unsupported scenario file",
            )
        }

        ScenarioError::FileNotFound
        | ScenarioError::ProcessFailed(_)
        | ScenarioError::InvalidOutput(_)
        | ScenarioError::ProcessTerminated
        | ScenarioError::ParserTimedOut => {
            tracing::error!(
                error = %error,
                "Scenario parser failed"
            );

            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
        }
    };

    error_response(status, message)
}

fn validate_scenario_filename(file_name: Option<String>) -> Result<String, ApiError> {
    let file_name = match file_name {
        Some(file_name) => file_name,
        None => {
            return Err(error_response(
                StatusCode::BAD_REQUEST,
                "Uploaded field has no filename",
            ));
        }
    };

    if std::path::Path::new(&file_name)
        .extension()
        .and_then(|extension| extension.to_str())
        != Some("aoe2scenario")
    {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "File must be an .aoe2scenario file",
        ));
    }

    Ok(file_name)
}

fn store_scenario_tempfile(bytes: &[u8]) -> Result<tempfile::NamedTempFile, ApiError> {
    let mut temp_file = match tempfile::NamedTempFile::new() {
        Ok(file) => file,
        Err(error) => {
            tracing::error!(
                error = %error,
                "tempfile::NamedTempFile::new failed"
            );

            return Err(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unable to store the scenario",
            ));
        }
    };

    match temp_file.write_all(bytes) {
        Ok(()) => {}
        Err(error) => {
            tracing::error!(
                error = %error,
                "tempfile::NamedTempFile::write_all failed"
            );

            return Err(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unable to store the scenario",
            ));
        }
    }
    Ok(temp_file)
}

async fn parse_scenario_with_slot(
    state: &AppState,
    path: &std::path::Path,
) -> Result<ScenarioInfo, ApiError> {
    let result = {
        let _permit =
            match tokio::time::timeout(PARSER_QUEUE_TIMEOUT, state.parser_slots.acquire()).await {
                Ok(Ok(permit)) => permit,

                Ok(Err(error)) => {
                    tracing::error!(
                        error = %error,
                        "Unable to acquire parser slot"
                    );

                    return Err(error_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Internal server error",
                    ));
                }

                Err(error) => {
                    tracing::warn!(
                        error = %error,
                        "Timed out waiting for parser slot"
                    );

                    return Err(error_response(
                        StatusCode::SERVICE_UNAVAILABLE,
                        "Server is busy processing another scenario",
                    ));
                }
            };

        scenario::parse_scenario(path).await
    };

    match result {
        Ok(scenario) => Ok(scenario),
        Err(error) => Err(scenario_error_response(error)),
    }
}

async fn read_scenario_upload(multipart: &mut Multipart) -> Result<ScenarioUpload, ApiError> {
    match multipart.next_field().await {
        Ok(Some(field)) => {
            let field_name = field.name().map(str::to_string);
            if field_name.as_deref() != Some("scenario") {
                return Err(error_response(
                    StatusCode::BAD_REQUEST,
                    "Expected multipart field named 'scenario'",
                ));
            }

            let file_name = field.file_name().map(str::to_string);
            let file_name = validate_scenario_filename(file_name)?;

            match field.bytes().await {
                Ok(bytes) => Ok(ScenarioUpload { file_name, bytes }),
                Err(err) => {
                    tracing::warn!(
                        error = %err,
                        "Unable to read multipart field"
                    );

                    Err(error_response(
                        StatusCode::BAD_REQUEST,
                        "Unable to read uploaded file",
                    ))
                }
            }
        }
        Ok(None) => Err(error_response(StatusCode::BAD_REQUEST, "No field received")),
        Err(err) => {
            tracing::warn!(
                error = %err,
                "Invalid multipart request"
            );
            Err(error_response(
                StatusCode::BAD_REQUEST,
                "Invalid upload request",
            ))
        }
    }
}

async fn receive_scenario(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<ScenarioInfo>, ApiError> {
    let upload = read_scenario_upload(&mut multipart).await?;
    tracing::debug!(
                    filename = ?upload.file_name,
                    size = upload.bytes.len(),
    "Scenario upload received"
                );

    let temp_file = store_scenario_tempfile(upload.bytes.as_ref())?;

    let scenario = parse_scenario_with_slot(&state, temp_file.path()).await?;
    tracing::info!(
        width = scenario.width,
        height = scenario.height,
        size = scenario.terrain.len(),
        "Scenario parsed"
    );

    Ok(Json(scenario))
}

fn render_minimap_png(scenario: &ScenarioInfo) -> Result<Vec<u8>, ApiError> {
    let minimap = minimap::render_isometric_minimap(scenario);

    let image = image::DynamicImage::ImageRgb8(minimap);
    let mut buffer = std::io::Cursor::new(Vec::new());

    match image.write_to(&mut buffer, image::ImageFormat::Png) {
        Ok(()) => {}
        Err(error) => {
            tracing::error!(
                        error = %error,
                        "image.write_to failed"
            );

            return Err(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unable to render the scenario",
            ));
        }
    }

    Ok(buffer.into_inner())
}

async fn receive_scenario_minimap(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Response, ApiError> {
    let upload = read_scenario_upload(&mut multipart).await?;
    tracing::debug!(
                    filename = ?upload.file_name,
                    size = upload.bytes.len(),
    "Scenario upload received"
                );

    let temp_file = store_scenario_tempfile(upload.bytes.as_ref())?;

    let scenario = parse_scenario_with_slot(&state, temp_file.path()).await?;

    let png_bytes = render_minimap_png(&scenario)?;

    Ok(([(header::CONTENT_TYPE, "image/png")], png_bytes).into_response())
}

#[tokio::main]
async fn main() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let state = AppState {
        parser_slots: Arc::new(Semaphore::new(1)),
    };
    let app = Router::new()
        .route("/", get(index))
        .route("/healthz", get(healthz))
        .route(
            "/api/scenario",
            post(receive_scenario).layer(DefaultBodyLimit::max(10 * 1024 * 1024)),
        )
        .route(
            "/api/scenario/minimap",
            post(receive_scenario_minimap).layer(DefaultBodyLimit::max(10 * 1024 * 1024)),
        )
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
