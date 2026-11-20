use std::{io::ErrorKind, path::PathBuf, sync::Arc};

use base64::Engine;
use tokio::sync::Mutex;

use crate::{config::Config, model::ResponseRecord};

#[derive(Clone)]
pub enum Db {
    File(FileDb),
    Firebase(FirebaseDb),
    Github(GithubDb),
}

impl Db {
    pub fn from_config(cfg: &Config) -> Self {
        if cfg.db_driver.eq_ignore_ascii_case("firebase") {
            Db::Firebase(FirebaseDb::new(&cfg.database_url, &cfg.database_path))
        } else if cfg.db_driver.eq_ignore_ascii_case("github") {
            Db::Github(GithubDb::new(
                &cfg.github_repo,
                &cfg.github_path,
                &cfg.github_token,
            ))
        } else {
            Db::File(FileDb::new(cfg.data_file.clone()))
        }
    }

    pub async fn list(&self) -> Result<Vec<ResponseRecord>, String> {
        match self {
            Db::File(db) => db.read().await,
            Db::Firebase(db) => db.list().await,
            Db::Github(db) => Ok(db.read().await?.0),
        }
    }

    pub async fn insert(&self, record: ResponseRecord) -> Result<(), String> {
        match self {
            Db::File(db) => db.insert(record).await,
            Db::Firebase(db) => db.insert(record).await,
            Db::Github(db) => {
                let (mut records, sha) = db.read().await?;
                records.push(record);
                db.write(&records, sha).await
            }
        }
    }

    pub async fn remove(&self, id: &str) -> Result<bool, String> {
        match self {
            Db::File(db) => db.remove(id).await,
            Db::Firebase(db) => db.remove(id).await,
            Db::Github(db) => {
                let (mut records, sha) = db.read().await?;
                let before = records.len();
                records.retain(|item| item.id != id);
                let removed = records.len() != before;
                db.write(&records, sha).await?;
                Ok(removed)
            }
        }
    }

    pub async fn clear(&self) -> Result<(), String> {
        match self {
            Db::File(db) => db.write(&[]).await,
            Db::Firebase(db) => db.clear().await,
            Db::Github(db) => {
                let (_, sha) = db.read().await?;
                db.write(&[], sha).await
            }
        }
    }
}

#[derive(Clone)]
pub struct FileDb {
    path: PathBuf,
    lock: Arc<Mutex<()>>,
}

impl FileDb {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            lock: Arc::new(Mutex::new(())),
        }
    }

    async fn read(&self) -> Result<Vec<ResponseRecord>, String> {
        match tokio::fs::read_to_string(&self.path).await {
            Ok(content) => serde_json::from_str(&content).map_err(|e| format!("DB corrupt: {e}")),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(Vec::new()),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn write(&self, records: &[ResponseRecord]) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| e.to_string())?;
        }
        let content = serde_json::to_string_pretty(records).map_err(|e| e.to_string())?;
        let temp = self.path.with_extension("tmp");
        tokio::fs::write(&temp, content)
            .await
            .map_err(|e| e.to_string())?;
        tokio::fs::rename(temp, &self.path)
            .await
            .map_err(|e| e.to_string())
    }

    async fn insert(&self, record: ResponseRecord) -> Result<(), String> {
        let _guard = self.lock.lock().await;
        let mut records = self.read().await?;
        records.push(record);
        self.write(&records).await
    }

    async fn remove(&self, id: &str) -> Result<bool, String> {
        let _guard = self.lock.lock().await;
        let mut records = self.read().await?;
        let before = records.len();
        records.retain(|item| item.id != id);
        let removed = records.len() != before;
        self.write(&records).await?;
        Ok(removed)
    }
}

#[derive(Clone)]
pub struct FirebaseDb {
    client: reqwest::Client,
    url: String,
}

impl FirebaseDb {
    pub fn new(base: &str, path: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            url: format!("{}/{}", base.trim_end_matches('/'), path.trim_matches('/')),
        }
    }

    fn node(&self, id: Option<&str>) -> String {
        match id {
            Some(id) => format!("{}/{id}.json", self.url),
            None => format!("{}.json", self.url),
        }
    }

    async fn list(&self) -> Result<Vec<ResponseRecord>, String> {
        let response = self
            .client
            .get(self.node(None))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !response.status().is_success() {
            return Err(format!("Firebase trả {}", response.status()));
        }
        let map: std::collections::HashMap<String, ResponseRecord> =
            response.json().await.map_err(|e| e.to_string())?;
        let mut records: Vec<ResponseRecord> = map.into_values().collect();
        records.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(records)
    }

    async fn insert(&self, record: ResponseRecord) -> Result<(), String> {
        let response = self
            .client
            .put(self.node(Some(&record.id)))
            .json(&record)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Firebase trả {}", response.status()))
        }
    }

    async fn remove(&self, id: &str) -> Result<bool, String> {
        let exists = self.list().await?.iter().any(|item| item.id == id);
        let response = self
            .client
            .delete(self.node(Some(id)))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if response.status().is_success() {
            Ok(exists)
        } else {
            Err(format!("Firebase trả {}", response.status()))
        }
    }

    async fn clear(&self) -> Result<(), String> {
        let response = self
            .client
            .delete(self.node(None))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Firebase trả {}", response.status()))
        }
    }
}

#[derive(Clone)]
pub struct GithubDb {
    client: reqwest::Client,
    api: String,
    token: String,
    lock: Arc<Mutex<()>>,
}

impl GithubDb {
    pub fn new(repo: &str, path: &str, token: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            api: format!("https://api.github.com/repos/{repo}/contents/{path}"),
            token: token.to_string(),
            lock: Arc::new(Mutex::new(())),
        }
    }

    async fn read(&self) -> Result<(Vec<ResponseRecord>, Option<String>), String> {
        let response = self
            .client
            .get(&self.api)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("User-Agent", "inviteflow")
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok((Vec::new(), None));
        }
        if !response.status().is_success() {
            return Err(format!("GitHub trả {}", response.status()));
        }
        let body: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        let encoded = body["content"].as_str().unwrap_or("").replace('\n', "");
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|e| e.to_string())?;
        let records: Vec<ResponseRecord> =
            serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        let sha = body["sha"].as_str().map(str::to_string);
        Ok((records, sha))
    }

    async fn write(&self, records: &[ResponseRecord], sha: Option<String>) -> Result<(), String> {
        let _guard = self.lock.lock().await;
        let content = serde_json::to_string_pretty(records).map_err(|e| e.to_string())?;
        let encoded = base64::engine::general_purpose::STANDARD.encode(content.as_bytes());
        let mut payload = serde_json::json!({
            "message": "update responses",
            "content": encoded,
        });
        if let Some(sha) = sha {
            payload["sha"] = serde_json::json!(sha);
        }
        let response = self
            .client
            .put(&self.api)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("User-Agent", "inviteflow")
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("GitHub trả {}", response.status()))
        }
    }
}
