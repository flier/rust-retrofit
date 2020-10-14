use anyhow::Result;
use structopt::StructOpt;

mod github;

use self::github::service::*;

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
    /// List repository teams
    Teams { owner: String, repo: String },
    /// Get all repository topics
    Topics { owner: String, repo: String },
}

fn main() -> Result<()> {
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
                for repo in github.list_repo(
                    &username,
                    &ListRepo {
                        ty,
                        sort,
                        direction,
                        pagination: opt.pagination,
                    },
                )? {
                    println!(
                        "{:40} watch: {:>4}, star: {:>4}, fork: {:>4}",
                        repo.name, repo.watchers_count, repo.stargazers_count, repo.forks_count
                    );
                }
            }
            Repo::Languages { owner, repo } => {
                for (lang, bytes) in github.list_repo_languages(&owner, &repo)? {
                    println!("{}: {}", lang, bytes);
                }
            }
            Repo::Tags { owner, repo } => {
                for tag in github.list_repo_tags(&owner, &repo, &opt.pagination)? {
                    println!("{}\t#{}", tag.name, &tag.commit.sha[..8]);
                }
            }
            Repo::Teams { owner, repo } => {
                for team in github.list_repo_teams(&owner, &repo, &opt.pagination)? {
                    println!("{}\t{}", team.name, team.description.unwrap_or_default());
                }
            }
            Repo::Topics { owner, repo } => {
                let topics = github.get_repo_topics(&owner, &repo)?;

                println!("{}", topics.names.join(","));
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }

    Ok(())
}
