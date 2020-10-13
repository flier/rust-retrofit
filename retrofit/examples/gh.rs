use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

use anyhow::Result;
use chrono::{DateTime, Utc};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use retrofit::{client, default_headers, get, request, service};

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

#[service(base_url = "https://api.github.com")]
#[client(
    connect_timeout = Some(Duration::from_secs(5)),
    no_gzip,
    user_agent = "gh/1.0",
)]
#[default_headers(accept = "application/vnd.github.v3+json")]
pub trait GithubService {
    /// Get a repository
    #[get("/repos/{owner}/{repo}")]
    fn get_repo(&self, owner: &str, repo: &str) -> Repo;

    /// List repositories for a user
    #[get("/users/{username}/repos")]
    #[request(query)]
    fn list_repo(&self, username: &str, query: &ListRepoQuery) -> Vec<Repo>;

    /// List repository languages
    #[get("/repos/{owner}/{repo}/languages")]
    fn list_repo_languages(&self, owner: &str, repo: &str) -> HashMap<String, usize>;

    /// List repository tags
    #[get("/repos/{owner}/{repo}/tags")]
    #[request(query)]
    fn list_repo_tags(&self, owner: &str, repo: &str, query: &Pagination) -> Vec<Tag>;
}

#[derive(Clone, Debug, Default, Serialize, StructOpt)]
pub struct Pagination {
    /// Results per page (max 100)
    #[structopt(short = "c", long)]
    pub per_page: Option<usize>,

    /// Page number of the results to fetch.
    #[structopt(short, long)]
    pub page: Option<usize>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ListRepoQuery {
    #[serde(rename = "type")]
    pub ty: Option<RepoType>,
    pub sort: Option<RepoSort>,
    pub direction: Option<Direction>,
    #[serde(flatten)]
    pub pagination: Pagination,
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

mod opt {
    use structopt::StructOpt;

    use super::{Direction, Pagination, RepoSort, RepoType};

    #[derive(Debug, StructOpt)]
    #[structopt(about = "Work seamlessly with GitHub from the command line.")]
    pub struct Opt {
        #[structopt(flatten)]
        pub pagination: Pagination,

        #[structopt(subcommand)]
        pub cmd: Cmd,
    }

    #[derive(Debug, StructOpt)]
    pub enum Cmd {
        /// Manage gists
        Gist {
            #[structopt(subcommand)]
            gist: Gist,
        },
        /// Manage issues
        Issue {
            #[structopt(subcommand)]
            issue: Issue,
        },
        /// Create, clone, fork, and view repositories
        Repo {
            #[structopt(subcommand)]
            repo: Repo,
        },
    }

    #[derive(Debug, StructOpt)]
    pub enum Gist {
        /// Create a new gist
        Create {},
        /// Edit one of your gists
        Edit {},
        /// List your gists
        List {},
        /// View a gist
        View {},
    }

    #[derive(Debug, StructOpt)]
    pub enum Issue {
        /// Close issue
        Close {},
        /// Create a new issue
        Create {},
        /// List and filter issues in this repository
        List {},
        /// Reopen issue
        Reopen {},
        /// Show status of relevant issues
        Status {},
        /// View an issue
        View {},
    }

    #[derive(Debug, StructOpt)]
    pub enum Repo {
        /// Clone a repository locally
        Clone {},
        /// Create a new repository
        Create {},
        /// Create a fork of a repository
        Fork {},
        /// View a repository
        View {},
        /// Lists repositories for the user.
        List {
            /// Can be one of `all`, `owner`, `member`.
            #[structopt(short, long = "type")]
            ty: Option<RepoType>,

            /// Can be one of `created`, `updated`, `pushed`, `full_name`.
            #[structopt(short, long)]
            sort: Option<RepoSort>,

            /// Can be one of `asc` or `desc`. Default: `asc` when using `full_name`, otherwise `desc`
            #[structopt(short, long)]
            direction: Option<Direction>,

            /// Give the specified user.
            username: String,
        },
        /// List repository languages
        Languages { owner: String, repo: String },
        /// List repository tags
        Tags { owner: String, repo: String },
    }
}

fn main() -> Result<()> {
    use opt::*;

    tracing_subscriber::fmt::init();

    let opt = Opt::from_args();

    let github = github_service();

    match opt.cmd {
        Cmd::Repo { repo } => match repo {
            Repo::List {
                username,
                ty,
                sort,
                direction,
            } => {
                let query = ListRepoQuery {
                    ty,
                    sort,
                    direction,
                    pagination: opt.pagination,
                };
                for repo in github.list_repo(&username, &query)? {
                    println!("{}", repo);
                }
            }
            Repo::Languages { owner, repo } => {
                for (lang, bytes) in github.list_repo_languages(&owner, &repo)? {
                    println!("{}: {}", lang, bytes);
                }
            }
            Repo::Tags { owner, repo } => {
                for tag in github.list_repo_tags(&owner, &repo, &opt.pagination)? {
                    println!("{}", tag);
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }

    Ok(())
}
