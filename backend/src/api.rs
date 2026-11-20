use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Instant};

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, SaltString};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use axum::{
    Json, Router,
    extract::{ConnectInfo, Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header::AUTHORIZATION},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::Mutex;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use uuid::Uuid;

use crate::{
    config::Config,
    db::{Db, GithubDb},
    model::NewResponse,
    validate::validate,
};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Db,
    pub limiter: Arc<Mutex<HashMap<String, (u64, Instant)>>>,
    pub http: reqwest::Client,
    pub admin_db: GithubDb,
    pub sessions: Arc<Mutex<HashMap<String, (String, String, Instant)>>>,
}

impl AppState {
    pub fn new(config: Arc<Config>, db: Db, admin_db: GithubDb) -> Self {
        Self {
            config,
            db,
            limiter: Arc::new(Mutex::new(HashMap::new())),
            http: reqwest::Client::new(),
            admin_db,
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[derive(Debug, Deserialize)]
struct LoginPayload {
    password: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    token: String,
    role: String,
    label: String,
}

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| e.to_string())
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

async fn authorized(state: &AppState, headers: &HeaderMap) -> Option<(String, String)> {
    let provided = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))?
        .trim();

    {
        let sessions = state.sessions.lock().await;
        if let Some((role, label, created)) = sessions.get(provided) {
            if created.elapsed() < std::time::Duration::from_secs(86400) {
                return Some((role.clone(), label.clone()));
            }
        }
    }

    state
        .config
        .admin_users
        .iter()
        .find(|user| equals(provided, &user.token))
        .map(|user| (user.role.clone(), user.label.clone()))
}

fn client_ip(headers: &HeaderMap, addr: SocketAddr) -> String {
    let forwarded: Option<&str> = headers
        .get("cf-connecting-ip")
        .and_then(|value| value.to_str().ok())
        .or_else(|| {
            headers
                .get("x-forwarded-for")
                .and_then(|value| value.to_str().ok())
        })
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty());
    forwarded
        .map(str::to_string)
        .unwrap_or_else(|| addr.ip().to_string())
}

async fn locate(
    client: &reqwest::Client,
    ip: &str,
) -> (Option<String>, Option<String>, Option<String>) {
    if ip.is_empty() || ip == "127.0.0.1" || ip.starts_with("10.") || ip.starts_with("192.168.") {
        return (None, None, None);
    }
    let url = format!("https://ipwho.is/{ip}");
    let response = match client
        .get(url)
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return (None, None, None),
    };
    let body: Value = match response.json().await {
        Ok(body) => body,
        Err(_) => return (None, None, None),
    };
    let pick = |key: &str| {
        body[key]
            .as_str()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    };
    (pick("country"), pick("region"), pick("city"))
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
    headers: HeaderMap,
    body: Result<Json<NewResponse>, axum::extract::rejection::JsonRejection>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let input = body
        .map_err(|e| ApiError(StatusCode::BAD_REQUEST, e.to_string()))?
        .0;
    let ip = client_ip(&headers, addr);
    rate_check(&state, ip.clone()).await?;

    let mut record = validate(input).map_err(|e| ApiError(StatusCode::BAD_REQUEST, e))?;
    record.ip = Some(ip.clone());
    let (country, region, city) = locate(&state.http, &ip).await;
    record.country = country;
    record.region = region;
    record.city = city;
    state
        .db
        .insert(record.clone())
        .await
        .map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok((StatusCode::CREATED, Json(json!({ "id": record.id }))))
}

async fn edit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<NewResponse>,
) -> Result<Json<Value>, ApiError> {
    let (role, _) = authorized(&state, &headers)
        .await
        .ok_or_else(|| ApiError(StatusCode::UNAUTHORIZED, "Cần token".into()))?;
    if role != "admin" && role != "editor" {
        return Err(ApiError(StatusCode::FORBIDDEN, "Không có quyền sửa".into()));
    }

    let existing = state
        .db
        .list()
        .await
        .map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e))?
        .into_iter()
        .find(|item| item.id == id)
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "Không tìm thấy lượt trả lời".into()))?;

    let mut record = validate(input.clone()).map_err(|e| ApiError(StatusCode::BAD_REQUEST, e))?;
    record.id = existing.id.clone();
    record.created_at = existing.created_at.clone();
    record.ip = existing.ip.clone();
    record.country = existing.country.clone();
    record.region = existing.region.clone();
    record.city = existing.city.clone();
    record.visitor_id = input.visitor_id.clone().or(existing.visitor_id.clone());
    record.user_agent = input.user_agent.clone().or(existing.user_agent.clone());
    record.language = input.language.clone().or(existing.language.clone());
    record.timezone = input.timezone.clone().or(existing.timezone.clone());

    let updated = state
        .db
        .update(record)
        .await
        .map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(json!({ "updated": updated })))
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>, ApiError> {
    let (value, _) = state
        .admin_db
        .read_value()
        .await
        .map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let stored = value["password_hash"]
        .as_str()
        .ok_or_else(|| ApiError(StatusCode::UNAUTHORIZED, "Chưa cấu hình mật khẩu".into()))?;
    let parsed = PasswordHash::new(stored).map_err(|_| {
        ApiError(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Hash không hợp lệ".into(),
        )
    })?;
    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed)
        .map_err(|_| ApiError(StatusCode::UNAUTHORIZED, "Mật khẩu không đúng".into()))?;

    let token = Uuid::new_v4().to_string();
    let role = value["role"].as_str().unwrap_or("admin").to_string();
    let label = value["label"].as_str().unwrap_or("Chủ").to_string();
    state
        .sessions
        .lock()
        .await
        .insert(token.clone(), (role.clone(), label.clone(), Instant::now()));
    Ok(Json(LoginResponse { token, role, label }))
}

async fn me(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Value>, ApiError> {
    let (role, label) = authorized(&state, &headers)
        .await
        .ok_or_else(|| ApiError(StatusCode::UNAUTHORIZED, "Cần token".into()))?;
    Ok(Json(json!({ "role": role, "label": label })))
}

async fn list(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Value>, ApiError> {
    authorized(&state, &headers)
        .await
        .ok_or_else(|| ApiError(StatusCode::UNAUTHORIZED, "Cần token".into()))?;
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
    let (role, _) = authorized(&state, &headers)
        .await
        .ok_or_else(|| ApiError(StatusCode::UNAUTHORIZED, "Cần token".into()))?;
    if role != "admin" && role != "editor" {
        return Err(ApiError(StatusCode::FORBIDDEN, "Không có quyền xóa".into()));
    }
    let removed = state
        .db
        .remove(&id)
        .await
        .map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(json!({ "removed": removed })))
}

async fn clear(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Value>, ApiError> {
    let (role, _) = authorized(&state, &headers)
        .await
        .ok_or_else(|| ApiError(StatusCode::UNAUTHORIZED, "Cần token".into()))?;
    if role != "admin" {
        return Err(ApiError(
            StatusCode::FORBIDDEN,
            "Chỉ quản trị viên được xóa toàn bộ".into(),
        ));
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
        .route("/api/me", get(me))
        .route("/api/login", post(login))
        .route("/health", get(health))
        .route("/kaithhealth", get(health))
        .route("/api/responses", post(create).get(list).delete(clear))
        .route("/api/responses/{id}", delete(remove_one).patch(edit))
        .fallback_service(ServeDir::new(state.config.frontend_dir.clone()))
        .layer(cors)
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AdminUser;
    use crate::db::FileDb;
    use tower::ServiceExt;

    fn state() -> AppState {
        let config = Arc::new(Config {
            host: "127.0.0.1".into(),
            port: 0,
            admin_token: "test-token".into(),
            admin_users: vec![AdminUser {
                token: "test-token".into(),
                role: "admin".into(),
                label: "Chủ".into(),
            }],
            admin_file: "admin.json".into(),
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
            GithubDb::new("th-ming/inviteflow-db", "admin.json", ""),
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
