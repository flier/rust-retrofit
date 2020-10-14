use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub commit: Commit,
    pub zipball_url: String,
    pub tarball_url: String,
    pub node_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Commit {
    pub sha: String,
    pub url: String,
}
