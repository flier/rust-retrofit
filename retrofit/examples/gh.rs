use std::str::FromStr;

use anyhow::Result;
use chrono::{DateTime, Utc};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use tracing::trace;

use retrofit::{default_headers, get, service, Service};

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

#[service(base_url = "https://api.github.com/")]
#[default_headers(accept = "application/vnd.github.v3+json")]
pub trait GithubService: Service {
    #[get("users/{username}/repos")]
    fn list_repo(&self, username: &str, r#type: Option<RepoType>)
        -> Result<Vec<Repo>, Self::Error>;
}

fn github_service(base_url: &str) -> impl GithubService {
    static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

    struct Client {
        client: reqwest::blocking::Client,
        base_url: String,
    }

    impl Service for Client {
        type Error = reqwest::Error;
    }

    impl GithubService for Client {
        fn list_repo(
            &self,
            username: &str,
            r#type: Option<RepoType>,
        ) -> Result<Vec<Repo>, Self::Error> {
            let url = format!(
                "{base_url}users/{username}/repos",
                base_url = self.base_url,
                username = username
            );
            let mut req = self.client.get(&url);
            if let Some(ty) = r#type {
                req = req.query(&[("type", ty.to_string())])
            }
            trace!(?req, "req");
            let res = req.send()?;
            trace!(?res, "res");
            let text = res.text()?;
            trace!(%text, "text");
            Ok(serde_json::from_reader(std::io::Cursor::new(text)).unwrap())
        }
    }

    Client {
        client: reqwest::blocking::Client::builder()
            .user_agent(APP_USER_AGENT)
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::ACCEPT,
                    reqwest::header::HeaderValue::from_static("application/vnd.github.v3+json"),
                );
                headers
            })
            .build()
            .expect("client"),
        base_url: base_url.to_string(),
    }
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
            let repos = github.list_repo(&username, r#type)?;

            for repo in repos {
                println!("{}", repo);
            }
        }
        _ => unimplemented!(),
    }

    Ok(())
}
