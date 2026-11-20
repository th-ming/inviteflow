use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Instant};

use axum::{
    Json, Router,
    extract::{ConnectInfo, Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header::AUTHORIZATION},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use serde_json::{Value, json};
use tokio::sync::Mutex;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};

use crate::{config::Config, db::Db, model::NewResponse, validate::validate};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Db,
    pub limiter: Arc<Mutex<HashMap<String, (u64, Instant)>>>,
}

impl AppState {
    pub fn new(config: Arc<Config>, db: Db) -> Self {
        Self {
            config,
            db,
            limiter: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

struct ApiError(StatusCode, String);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({ "error": self.1 }))).into_response()
    }
}

fn equals(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b) {
        diff |= x ^ y;
    }
    diff == 0
}

fn authorized(state: &AppState, headers: &HeaderMap) -> bool {
    headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .is_some_and(|token| equals(token.trim(), &state.config.admin_token))
}

async fn rate_check(state: &AppState, ip: String) -> Result<(), ApiError> {
    let mut map = state.limiter.lock().await;
    let now = Instant::now();
    let entry = map.entry(ip).or_insert((0, now));
    if now.duration_since(entry.1).as_millis() as u64 >= state.config.rate_window_ms {
        *entry = (0, now);
    }
    if entry.0 >= state.config.rate_limit {
        return Err(ApiError(
            StatusCode::TOO_MANY_REQUESTS,
            "Quá nhiều request".into(),
        ));
    }
    entry.0 += 1;
    Ok(())
}

async fn health() -> Json<Value> {
    Json(json!({ "ok": true }))
}

async fn create(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    body: Result<Json<NewResponse>, axum::extract::rejection::JsonRejection>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let input = body
        .map_err(|e| ApiError(StatusCode::BAD_REQUEST, e.to_string()))?
        .0;
    rate_check(&state, addr.ip().to_string()).await?;

    let record = validate(input).map_err(|e| ApiError(StatusCode::BAD_REQUEST, e))?;
    state
        .db
        .insert(record.clone())
        .await
        .map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok((StatusCode::CREATED, Json(json!({ "id": record.id }))))
}

async fn list(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Value>, ApiError> {
    if !authorized(&state, &headers) {
        return Err(ApiError(StatusCode::UNAUTHORIZED, "Cần token".into()));
    }
    let records = state
        .db
        .list()
        .await
        .map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(json!(records)))
}

async fn remove_one(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    if !authorized(&state, &headers) {
        return Err(ApiError(StatusCode::UNAUTHORIZED, "Cần token".into()));
    }
    let removed = state
        .db
        .remove(&id)
        .await
        .map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(json!({ "removed": removed })))
}

async fn clear(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Value>, ApiError> {
    if !authorized(&state, &headers) {
        return Err(ApiError(StatusCode::UNAUTHORIZED, "Cần token".into()));
    }
    state
        .db
        .clear()
        .await
        .map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(json!({ "cleared": true })))
}

pub fn router(state: AppState) -> Router {
    let origin = state.config.allowed_origin.clone();
    let allowed = HeaderValue::from_str(&origin).unwrap_or_else(|_| HeaderValue::from_static("*"));
    let cors = CorsLayer::new()
        .allow_origin(allowed)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/health", get(health))
        .route("/health", get(health))
        .route("/kaithhealth", get(health))
        .route("/api/responses", post(create).get(list).delete(clear))
        .route("/api/responses/{id}", delete(remove_one))
        .fallback_service(ServeDir::new(state.config.frontend_dir.clone()))
        .layer(cors)
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::FileDb;
    use tower::ServiceExt;

    fn state() -> AppState {
        let config = Arc::new(Config {
            host: "127.0.0.1".into(),
            port: 0,
            admin_token: "test-token".into(),
            db_driver: "file".into(),
            database_url: "https://example.firebaseio.com".into(),
            database_path: "responses".into(),
            data_file: std::env::temp_dir()
                .join(format!("inviteflow-test-{}.json", uuid::Uuid::new_v4())),
            github_token: String::new(),
            github_repo: "th-ming/inviteflow-db".into(),
            github_path: "responses.json".into(),
            allowed_origin: "*".into(),
            rate_limit: 10,
            rate_window_ms: 60_000,
            frontend_dir: std::env::temp_dir(),
        });
        AppState::new(
            config.clone(),
            Db::File(FileDb::new(config.data_file.clone())),
        )
    }

    #[tokio::test]
    async fn health_works() {
        let response = router(state())
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn create_then_list_needs_token() {
        let app = router(state());
        let create_response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/responses")
                    .header("content-type", "application/json")
                    .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345))))
                    .body(axum::body::Body::from(r#"{"consent":"Vâng","food":"Cơm"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(create_response.status(), StatusCode::CREATED);

        let denied = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/responses")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);

        let allowed = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/responses")
                    .header("authorization", "Bearer test-token")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(allowed.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn invalid_consent_is_rejected() {
        let response = router(state())
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/responses")
                    .header("content-type", "application/json")
                    .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345))))
                    .body(axum::body::Body::from(r#"{"consent":"Không"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn token_compare_is_exact() {
        assert!(equals("abc", "abc"));
        assert!(!equals("abc", "abcd"));
        assert!(!equals("abc", "abd"));
    }
}
