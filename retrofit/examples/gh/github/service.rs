use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

use anyhow::Result;
use serde::Serialize;
use structopt::StructOpt;

use retrofit::*;

use crate::github::{Repo, Tag, Team, Topics};

const GITHUB_JSON_V3: &str = "application/vnd.github.v3+json";
const GITHUB_JSON_PREVIEW: &str = "application/vnd.github.mercy-preview+json";

#[service(base_url = "https://api.github.com")]
#[client(
    connect_timeout = Some(Duration::from_secs(5)),
    no_gzip,
    user_agent = "gh/1.0",
)]
#[default_headers(accept = GITHUB_JSON_V3)]
pub trait GithubService {
    /// Get a repository
    #[get("/repos/{owner}/{repo}")]
    fn get_repo(&self, owner: &str, repo: &str) -> Repo;

    /// Update a repository
    #[patch("/repos/{owner}/{repo}")]
    #[request(body)]
    fn update_repo(&self, owner: &str, repo: &str, body: UpdateRepo) -> Repo;

    /// Delete a repository
    #[delete("/repos/{owner}/{repo}")]
    fn delete_repo(&self, owner: &str, repo: &str) -> ();

    /// List repositories for a user
    #[get("/users/{username}/repos")]
    #[request(query)]
    fn list_repo(&self, username: &str, query: &ListRepo) -> Vec<Repo>;

    /// List repository languages
    #[get("/repos/{owner}/{repo}/languages")]
    fn list_repo_languages(&self, owner: &str, repo: &str) -> HashMap<String, usize>;

    /// List repository tags
    #[get("/repos/{owner}/{repo}/tags")]
    #[request(query = pagination)]
    fn list_repo_tags(&self, owner: &str, repo: &str, pagination: &Pagination) -> Vec<Tag>;

    /// List repository teams
    #[get("/repos/{owner}/{repo}/teams")]
    #[request(query = pagination)]
    fn list_repo_teams(&self, owner: &str, repo: &str, pagination: &Pagination) -> Vec<Team>;

    /// Get all repository topics
    #[get("/repos/{owner}/{repo}/topics")]
    #[headers(accept = GITHUB_JSON_PREVIEW)]
    fn get_repo_topics(&self, owner: &str, repo: &str) -> Topics;

    /// Replace all repository topics
    #[put("/repos/{owner}/{repo}/topics")]
    #[request(body = topics)]
    fn replace_repo_topics(&self, owner: &str, repo: &str, topics: Topics) -> Topics;
}

#[derive(Clone, Debug, Default, Serialize, Body, StructOpt)]
pub struct Pagination {
    /// Results per page (max 100)
    #[structopt(short = "c", long)]
    pub per_page: Option<usize>,

    /// Page number of the results to fetch.
    #[structopt(short, long)]
    pub page: Option<usize>,
}

#[derive(Clone, Debug, Default, Serialize, Body)]
pub struct ListRepo {
    #[serde(rename = "type")]
    pub ty: Option<RepoType>,
    pub sort: Option<RepoSort>,
    pub direction: Option<Direction>,
    #[serde(flatten)]
    pub pagination: Pagination,
}

#[derive(Clone, Debug, Default, Serialize, Body)]
pub struct UpdateRepo {
    /// The name of the repository.
    pub name: Option<String>,
    /// A short description of the repository.
    pub description: Option<String>,
    /// A URL with more information about the repository.
    pub homepage: Option<String>,
    /// Either `true` to make the repository private or `false` to make it public.
    pub private: Option<bool>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RepoType {
    All,
    Owner,
    Member,
}

impl FromStr for RepoType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "all" => Ok(RepoType::All),
            "owner" => Ok(RepoType::Owner),
            "member" => Ok(RepoType::Member),
            _ => Err(format!("unexpected `{}`", s)),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RepoSort {
    Created,
    Updated,
    Pushed,
    FullName,
}

impl FromStr for RepoSort {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "created" => Ok(RepoSort::Created),
            "updated" => Ok(RepoSort::Updated),
            "pushed" => Ok(RepoSort::Pushed),
            "full_name" => Ok(RepoSort::FullName),
            _ => Err(format!("unexpected `{}`", s)),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Asc,
    Desc,
}

impl FromStr for Direction {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asc" => Ok(Direction::Asc),
            "desc" => Ok(Direction::Desc),
            _ => Err(format!("unexpected `{}`", s)),
        }
    }
}
