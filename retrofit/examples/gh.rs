use std::collections::HashMap;
use std::str::FromStr;

use anyhow::Result;
use chrono::{DateTime, Utc};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use retrofit::{args, default_headers, get, service, Service};

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

#[derive(Debug, Display)]
pub enum RepoType {
    #[display(fmt = "all")]
    All,
    #[display(fmt = "owner")]
    Owner,
    #[display(fmt = "member")]
    Member,
}

impl FromStr for RepoType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "all" => Ok(RepoType::All),
            "owner" => Ok(RepoType::Owner),
            "member" => Ok(RepoType::Member),
            _ => Err(s.to_string()),
        }
    }
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
#[default_headers(accept = "application/vnd.github.v3+json", user_agent = "gh/1.0")]
pub trait GithubService: Service {
    #[get("users/{username}/repos?type={ty}")]
    #[args(ty = ty.map(|ty| ty.to_string()).unwrap_or_default())]
    fn list_repo(&self, username: &str, ty: Option<RepoType>) -> Result<Vec<Repo>, Self::Error>;

    #[get("repos/{owner}/{repo}/languages")]
    fn list_repo_languages(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<HashMap<String, usize>, Self::Error>;

    #[get("repos/{owner}/{repo}/tags")]
    fn list_repo_tags(&self, owner: &str, repo: &str) -> Result<Vec<Tag>, Self::Error>;
}

mod opt {
    use structopt::StructOpt;

    use super::RepoType;

    #[derive(Debug, StructOpt)]
    #[structopt(about = "Work seamlessly with GitHub from the command line.")]
    pub struct Opt {
        #[structopt(short = "u", long, default_value = "https://api.github.com/")]
        pub base_url: String,

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
            #[structopt(short, long)]
            r#type: Option<RepoType>,
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

    let github = github_service(&opt.base_url);

    match opt.cmd {
        Cmd::Repo {
            repo: Repo::List { username, r#type },
        } => {
            for repo in github.list_repo(&username, r#type)? {
                println!("{}", repo);
            }
        }
        Cmd::Repo {
            repo: Repo::Languages { owner, repo },
        } => {
            for (lang, bytes) in github.list_repo_languages(&owner, &repo)? {
                println!("{}: {}", lang, bytes);
            }
        }
        Cmd::Repo {
            repo: Repo::Tags { owner, repo },
        } => {
            for tag in github.list_repo_tags(&owner, &repo)? {
                println!("{}", tag);
            }
        }
        _ => unimplemented!(),
    }

    Ok(())
}
