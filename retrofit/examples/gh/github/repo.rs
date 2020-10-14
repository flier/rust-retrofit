use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use retrofit::Body;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repo {
    pub id: usize,
    pub node_id: String,
    pub name: String,
    pub full_name: String,
    pub private: bool,
    pub description: Option<String>,
    pub forks_count: usize,
    pub stargazers_count: usize,
    pub watchers_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub pushed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Body)]
pub struct Topics {
    pub names: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Team {
    pub id: usize,
    pub node_id: String,
    pub name: String,
    pub description: Option<String>,
    pub url: String,
    pub html_url: String,
}
