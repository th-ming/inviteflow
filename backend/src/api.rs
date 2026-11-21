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
#[allow(clippy::type_complexity)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Db,
    pub limiter: Arc<Mutex<HashMap<String, (u64, Instant)>>>,
    pub http: reqwest::Client,
    pub admin_db: GithubDb,
    pub content_db: GithubDb,
    pub sessions: Arc<Mutex<HashMap<String, (String, String, Instant)>>>,
    pub cache: Arc<Mutex<Option<(Instant, Vec<crate::model::ResponseRecord>)>>>,
    pub content_cache: Arc<Mutex<Option<(Instant, Value)>>>,
}

impl AppState {
    pub fn new(config: Arc<Config>, db: Db, admin_db: GithubDb, content_db: GithubDb) -> Self {
        Self {
            config,
            db,
            limiter: Arc::new(Mutex::new(HashMap::new())),
            http: reqwest::Client::new(),
            admin_db,
            content_db,
            sessions: Arc::new(Mutex::new(HashMap::new())),
            cache: Arc::new(Mutex::new(None)),
            content_cache: Arc::new(Mutex::new(None)),
        }
    }
}

const CACHE_TTL_SECS: u64 = 30;

fn item(name: &str, image: &str, message: &str, special: bool) -> Value {
    json!({ "name": name, "image": image, "message": message, "special": special })
}

fn default_content() -> Value {
    json!({
        "home": {
            "eyebrow": "Một lời mời",
            "title": "Làm người yêu anh nhé?",
            "lead": "Mình cùng nhau đi chơi nha. Một buổi chiều, một ly nước, và một câu hỏi.",
            "images": [
                "https://i.pinimg.com/originals/54/22/99/54229922820d7c505a7ab53d783b310b.gif",
                "https://i.pinimg.com/originals/44/06/b8/4406b868368695db27f93d9780e67f6b.gif",
                "https://i.pinimg.com/736x/a1/9c/1a/a19c1a969a8a82f5cde6a7848feaa807.jpg",
                "https://i.pinimg.com/736x/b8/d1/ae/b8d1ae6b9861b6e625b95e3b89bf9313.jpg"
            ]
        },
        "thanks": {
            "title": "Cảm ơn bạn đã bấm nút",
            "lead": "Chọn một ngày mình cùng rảnh, rồi tính tiếp nhé.",
            "spam": ["Bấm đi mà", "Click đi", "Thử đi", "Đừng ngại", "Bấm mạnh vào"]
        },
        "date": {
            "title": "Em rảnh vào hàm nè",
            "lead": "Ngày nào: chọn một ngày nha"
        },
        "food": {
            "title": "Em muốn ăn món gì?",
            "items": [
                item("Cơm", "https://i.pinimg.com/736x/ac/f2/14/acf21487e6638097f8e8922b153c1ffa.jpg", "Một Cơm sẽ được phục vụ", false),
                item("Bún Bò", "https://i.pinimg.com/736x/f4/41/e4/f441e4d163c5cc83ce64f29098ad34fa.jpg", "Một Bún Bò sẽ được phục vụ", false),
                item("Phở Cuốn", "https://i.pinimg.com/736x/6e/06/fe/6e06fe0a41b99505ebd6eec9175251f8.jpg", "Một Phở Cuốn sẽ được phục vụ", false),
                item("Nem Nướng", "https://i.pinimg.com/736x/dc/c9/c2/dcc9c227c62d5146b7f4785d5788dc35.jpg", "Một Nem Nướng sẽ được phục vụ", false),
                item("Bún Cá", "https://i.pinimg.com/736x/b8/ae/1e/b8ae1e54f6f0a340c9ab911c55983f07.jpg", "Một Bún Cá sẽ được phục vụ", false),
                item("Mì Cay", "https://i.pinimg.com/736x/4e/a7/c2/4ea7c29e0afdbed1ac53d914a617a7fc.jpg", "Một Mì Cay sẽ được phục vụ", false),
                item("Ăn Gì Cũng Được", "https://i.pinimg.com/736x/15/c5/36/15c536783014eb6fa4bbc953021b3eb6.jpg", "Ăn gì cho em tự chọn nha. Không nữa thì ăn anh nè", true)
            ]
        },
        "drink": {
            "title": "Em muốn uống gì?",
            "items": [
                item("Trà Sữa", "https://i.pinimg.com/736x/87/2a/09/872a0953896df1c6e757fd54b845da23.jpg", "Một ly Trà Sữa tuyệt vời sắp được phục vụ", false),
                item("Cafe Sữa Đá", "https://i.pinimg.com/736x/1f/c8/72/1fc872840630055919672fce03723af9.jpg", "Một ly Cafe Sữa Đá tuyệt vời sắp được phục vụ", false),
                item("Nước Ép Cam", "https://i.pinimg.com/736x/3b/5c/fc/3b5cfc41457081509f53f280020e6cd9.jpg", "Một ly Nước Ép Cam tuyệt vời sắp được phục vụ", false),
                item("Sinh Tố Bơ", "https://i.pinimg.com/736x/00/d4/29/00d4296dc338a1b18c6d15ba68d9d0d3.jpg", "Một ly Sinh Tố Bơ tuyệt vời sắp được phục vụ", false),
                item("Nước Dừa", "https://i.pinimg.com/736x/f7/46/76/f74676e64d0ccd46daf08d419c0fbe5d.jpg", "Một ly Nước Dừa tuyệt vời sắp được phục vụ", false),
                item("Trà Chanh", "https://i.pinimg.com/736x/c6/4d/26/c64d26e91c7c1a09916bcc3223da561b.jpg", "Một ly Trà Chanh tuyệt vời sắp được phục vụ", false),
                item("Uống Gì Cũng Được", "https://i.pinimg.com/736x/6f/36/c8/6f36c874861a46b8e4208454157a1d6b.jpg", "Cho em chọn nhé :33", true)
            ]
        },
        "place": {
            "title": "Xong em muốn đi chơi đâu nữa nà?",
            "items": [
                item("Công Viên", "https://i.pinimg.com/736x/c9/64/58/c9645891fc4c84d400a93c00c8dc42a2.jpg", "Một buổi đi chơi Công Viên thật tuyệt vời nhé", false),
                item("Rạp Phim", "https://i.pinimg.com/736x/1f/5e/cc/1f5ecca4b80819393a320a95d14c51d0.jpg", "Một buổi đi chơi Rạp Phim thật tuyệt vời nhé", false),
                item("Quán Cafe", "https://i.pinimg.com/736x/c7/b8/fc/c7b8fcfc09f0c709458ba1e657405c15.jpg", "Một buổi đi chơi Quán Cafe thật tuyệt vời nhé", false),
                item("Nhà Ma", "https://dulich9.com/wp-content/uploads/2024/01/Dia-chi-cac-nha-ma-o-Ha-Noi-03.webp", "Một buổi đi chơi Nhà Ma thật tuyệt vời nhé", false),
                item("Dạo Quanh VNUA", "https://www.vnua.edu.vn/DATA/0/IMAGES/2017/07/hocvien/2017-2707hocvien004.JPG", "Một buổi đi chơi Dạo Quanh VNUA thật tuyệt vời nhé", false),
                item("Nơi Có Chúng Ta", "https://i.pinimg.com/736x/b6/08/3c/b6083cc8a66ab145ec71efce64ad96b2.jpg", "Đi đâu miễn đi cùng em là được", true)
            ]
        },
        "done": {
            "title": "Hẹn gặp bạn",
            "closing": "Đi đâu miễn đi cùng em là được"
        }
    })
}

fn valid_url(value: &str) -> bool {
    value.starts_with("https://") || value.starts_with("http://") || value.is_empty()
}

fn validate_content(value: &Value) -> Result<(), String> {
    let obj = value.as_object().ok_or("content phải là object")?;
    for key in ["home", "thanks", "date", "food", "drink", "place", "done"] {
        let page = obj.get(key).ok_or(format!("thiếu trang {key}"))?;
        let page = page
            .as_object()
            .ok_or(format!("trang {key} phải là object"))?;
        if let Some(title) = page.get("title") {
            let title = title.as_str().ok_or("title phải là chuỗi")?;
            if title.len() > 200 {
                return Err("title quá dài".into());
            }
        }
        if let Some(lead) = page.get("lead") {
            let lead = lead.as_str().ok_or("lead phải là chuỗi")?;
            if lead.len() > 300 {
                return Err("lead quá dài".into());
            }
        }
        if let Some(eyebrow) = page.get("eyebrow")
            && !eyebrow.is_string()
        {
            return Err("eyebrow phải là chuỗi".into());
        }
        if let Some(images) = page.get("images") {
            let images = images.as_array().ok_or("images phải là mảng")?;
            if images.len() > 8 {
                return Err("tối đa 8 ảnh".to_string());
            }
            for image in images {
                let image = image.as_str().ok_or("image phải là chuỗi")?;
                if image.len() > 500 || !valid_url(image) {
                    return Err("image không hợp lệ".into());
                }
            }
        }
        if let Some(spam) = page.get("spam") {
            let spam = spam.as_array().ok_or("spam phải là mảng")?;
            if spam.len() > 10 {
                return Err("tối đa 10 câu spam".to_string());
            }
            for text in spam {
                let text = text.as_str().ok_or("spam phải là chuỗi")?;
                if text.len() > 60 {
                    return Err("câu spam quá dài".into());
                }
            }
        }
        if let Some(items) = page.get("items") {
            let items = items.as_array().ok_or("items phải là mảng")?;
            if items.is_empty() {
                return Err("cần ít nhất 1 lựa chọn".to_string());
            }
            if items.len() > 30 {
                return Err("tối đa 30 lựa chọn".to_string());
            }
            for entry in items {
                let entry = entry.as_object().ok_or("item phải là object")?;
                let name = entry.get("name").and_then(|v| v.as_str()).unwrap_or("");
                if name.is_empty() || name.len() > 100 {
                    return Err("tên lựa chọn không hợp lệ".into());
                }
                let image = entry.get("image").and_then(|v| v.as_str()).unwrap_or("");
                if image.len() > 500 || !valid_url(image) {
                    return Err("ảnh lựa chọn không hợp lệ".into());
                }
                let message = entry.get("message").and_then(|v| v.as_str()).unwrap_or("");
                if message.len() > 200 {
                    return Err("message quá dài".into());
                }
            }
        }
    }
    Ok(())
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
        if let Some((role, label, created)) = sessions.get(provided)
            && created.elapsed() < std::time::Duration::from_secs(86400)
        {
            return Some((role.clone(), label.clone()));
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

    invalidate(&state);
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
    invalidate(&state);
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
    let records = cached_responses(&state)
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
    invalidate(&state);
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
    invalidate(&state);
    Ok(Json(json!({ "cleared": true })))
}

async fn cached_responses(state: &AppState) -> Result<Vec<crate::model::ResponseRecord>, String> {
    {
        let cache = state.cache.lock().await;
        if let Some((at, records)) = cache.as_ref()
            && at.elapsed() < std::time::Duration::from_secs(CACHE_TTL_SECS)
        {
            return Ok(records.clone());
        }
    }
    let records = state.db.list().await?;
    *state.cache.lock().await = Some((Instant::now(), records.clone()));
    Ok(records)
}

fn invalidate(state: &AppState) {
    if let Ok(mut cache) = state.cache.try_lock() {
        *cache = None;
    }
    if let Ok(mut cache) = state.content_cache.try_lock() {
        *cache = None;
    }
}

async fn get_content(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    {
        let cache = state.content_cache.lock().await;
        if let Some((at, content)) = cache.as_ref()
            && at.elapsed() < std::time::Duration::from_secs(CACHE_TTL_SECS)
        {
            return Ok(Json(content.clone()));
        }
    }
    let (value, _) = state
        .content_db
        .read_value()
        .await
        .map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let content = if value.is_null() {
        default_content()
    } else {
        value
    };
    *state.content_cache.lock().await = Some((Instant::now(), content.clone()));
    Ok(Json(content))
}

async fn put_content(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let (role, _) = authorized(&state, &headers)
        .await
        .ok_or_else(|| ApiError(StatusCode::UNAUTHORIZED, "Cần token".into()))?;
    if role != "admin" && role != "editor" {
        return Err(ApiError(
            StatusCode::FORBIDDEN,
            "Không có quyền sửa nội dung".into(),
        ));
    }
    validate_content(&input).map_err(|e| ApiError(StatusCode::BAD_REQUEST, e))?;
    let (_, sha) = state
        .content_db
        .read_value()
        .await
        .map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    state
        .content_db
        .write_value(&input, sha)
        .await
        .map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    invalidate(&state);
    Ok(Json(json!({ "saved": true })))
}

async fn export_all(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    authorized(&state, &headers)
        .await
        .ok_or_else(|| ApiError(StatusCode::UNAUTHORIZED, "Cần token".into()))?;
    let (content, _) = state
        .content_db
        .read_value()
        .await
        .map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let content = if content.is_null() {
        default_content()
    } else {
        content
    };
    let responses = state
        .db
        .list()
        .await
        .map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(json!({ "content": content, "responses": responses })))
}

async fn restore_all(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let (role, _) = authorized(&state, &headers)
        .await
        .ok_or_else(|| ApiError(StatusCode::UNAUTHORIZED, "Cần token".into()))?;
    if role != "admin" {
        return Err(ApiError(
            StatusCode::FORBIDDEN,
            "Chỉ quản trị viên được phục hồi".into(),
        ));
    }
    let content = input
        .get("content")
        .cloned()
        .ok_or_else(|| ApiError(StatusCode::BAD_REQUEST, "thiếu content".into()))?;
    validate_content(&content).map_err(|e| ApiError(StatusCode::BAD_REQUEST, e))?;
    let responses = input
        .get("responses")
        .cloned()
        .ok_or_else(|| ApiError(StatusCode::BAD_REQUEST, "thiếu responses".into()))?;
    let responses: Vec<crate::model::ResponseRecord> =
        serde_json::from_value(responses).map_err(|e| {
            ApiError(
                StatusCode::BAD_REQUEST,
                format!("responses không hợp lệ: {e}"),
            )
        })?;

    let (_, content_sha) = state
        .content_db
        .read_value()
        .await
        .map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    state
        .content_db
        .write_value(&content, content_sha)
        .await
        .map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    state
        .db
        .clear()
        .await
        .map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    for record in responses {
        state
            .db
            .insert(record)
            .await
            .map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    invalidate(&state);
    Ok(Json(json!({ "restored": true })))
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
        .route("/api/content", get(get_content).put(put_content))
        .route("/api/export", get(export_all))
        .route("/api/restore", post(restore_all))
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
            GithubDb::new("th-ming/inviteflow-db", "content.json", ""),
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
