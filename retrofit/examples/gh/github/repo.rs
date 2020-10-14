use chrono::{DateTime, Utc};
use derive_more::Display;
use serde::{Deserialize, Serialize};

use retrofit::Body;

#[derive(Debug, Clone, Display, Serialize, Deserialize)]
#[display(
    fmt = "{:40} watch: {:>4}, star: {:>4}, fork: {:>4}",
    name,
    watchers_count,
    stargazers_count,
    forks_count
)]
pub struct Repo {
    pub id: usize,
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
