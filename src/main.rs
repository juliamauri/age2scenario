mod scenario;

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Multipart},
    http::StatusCode,
    response::Html,
    routing::{get, post},
};
use scenario::{ScenarioError, ScenarioInfo};
use serde::Serialize;
use std::io::Write;
use tokio::net::TcpListener;

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../web/index.html"))
}

async fn healthz() -> &'static str {
    "ok"
}

async fn receive_scenario(
    mut multipart: Multipart,
) -> Result<Json<ScenarioInfo>, (StatusCode, Json<ErrorResponse>)> {
    match multipart.next_field().await {
        Ok(Some(field)) => {
            let field_name = field.name().map(str::to_string);
            let file_name = field.file_name().map(str::to_string);

            if field_name.as_deref() != Some("scenario") {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Expected multipart field named 'scenario'".to_string(),
                    }),
                ));
            }

            let file_name = match file_name.as_deref() {
                Some(file_name) => file_name,
                None => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: "Uploaded field has no filename".to_string(),
                        }),
                    ));
                }
            };

            if std::path::Path::new(file_name)
                .extension()
                .and_then(|extension| extension.to_str())
                != Some("aoe2scenario")
            {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "File must be an .aoe2scenario file".to_string(),
                    }),
                ));
            }

            match field.bytes().await {
                Ok(bytes) => {
                    tracing::debug!(
                        field = ?field_name,
                        filename = ?file_name,
                        size = bytes.len(),
                        "Scenario upload received"
                    );

                    let mut temp_file = match tempfile::NamedTempFile::new() {
                        Ok(file) => file,
                        Err(error) => {
                            tracing::error!(
                                error = %error,
                                "tempfile::NamedTempFile::new failed"
                            );

                            return Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(ErrorResponse {
                                    error: "Unable to store the scenario".to_string(),
                                }),
                            ));
                        }
                    };

                    match temp_file.write_all(&bytes) {
                        Ok(()) => {}
                        Err(error) => {
                            tracing::error!(
                                error = %error,
                                "tempfile::NamedTempFile::write_all failed"
                            );

                            return Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(ErrorResponse {
                                    error: "Unable to store the scenario".to_string(),
                                }),
                            ));
                        }
                    }

                    let result = scenario::parse_scenario(temp_file.path()).await;
                    match result {
                        Ok(scenario) => {
                            tracing::info!(
                                width = scenario.width,
                                height = scenario.height,
                                "Scenario parsed"
                            );
                            Ok(Json(scenario))
                        }
                        Err(error) => {
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
                                | ScenarioError::InvalidOutput(_) => {
                                    tracing::error!(
                                        error = %error,
                                        "Scenario parser failed"
                                    );

                                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
                                }
                            };
                            Err((
                                status,
                                Json(ErrorResponse {
                                    error: message.to_string(),
                                }),
                            ))
                        }
                    }
                }
                Err(err) => {
                    tracing::warn!(
                        error = %err,
                        "Unable to read multipart field"
                    );

                    Err((
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: "Unable to read uploaded file".to_string(),
                        }),
                    ))
                }
            }
        }
        Ok(None) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "No field received".to_string(),
            }),
        )),
        Err(err) => {
            tracing::warn!(
                error = %err,
                "Invalid multipart request"
            );
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Invalid upload request".to_string(),
                }),
            ))
        }
    }
}

#[tokio::main]
async fn main() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let app = Router::new()
        .route("/", get(index))
        .route("/healthz", get(healthz))
        .route(
            "/api/scenario",
            post(receive_scenario).layer(DefaultBodyLimit::max(10 * 1024 * 1024)),
        );

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
