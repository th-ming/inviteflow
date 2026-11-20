use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseRecord {
    pub id: String,
    pub consent: String,
    pub free_day: Option<String>,
    pub food: Option<String>,
    pub drink: Option<String>,
    pub place: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct NewResponse {
    pub consent: String,
    pub free_day: Option<String>,
    pub food: Option<String>,
    pub drink: Option<String>,
    pub place: Option<String>,
}
