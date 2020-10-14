use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Display, Serialize, Deserialize)]
#[display(fmt = "{} {}", name, commit)]
pub struct Tag {
    pub name: String,
    pub commit: Commit,
    pub zipball_url: String,
    pub tarball_url: String,
    pub node_id: String,
}

#[derive(Debug, Display, Serialize, Deserialize)]
#[display(fmt = "#{}", sha)]
pub struct Commit {
    pub sha: String,
    pub url: String,
}
