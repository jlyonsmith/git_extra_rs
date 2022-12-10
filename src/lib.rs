mod log_macros;

use clap::{CommandFactory, Parser, Subcommand};
use core::fmt::Arguments;
use duct::cmd;
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::{env, path::PathBuf};

lazy_static! {
    static ref RE_ORIGIN: Regex =
        RegexBuilder::new("^(?P<name>[a-zA-Z0-9\\-]+)\\s+(?P<repo>.*)\\s+\\(fetch\\)$")
            .multi_line(true)
            .build()
            .unwrap();
    static ref RE_SSH: Regex =
        RegexBuilder::new("^git@(?P<domain>[a-z0-9\\-\\.]+):(?P<user>[a-zA-Z0-9\\-_]+)/(?P<project>[a-zA-Z0-9\\-_]+)\\.git$")
            .build()
            .unwrap();
    static ref RE_HTTPS: Regex =
        RegexBuilder::new("^https://([a-zA-Z0-9\\-_]+@)?(?P<domain>[a-z0-9\\-\\.]+)/(?P<user>[a-zA-Z0-9\\-_]+)/(?P<project>[a-zA-Z0-9\\-_]+)\\.git$")
            .build()
            .unwrap();
}

const DEFAULT_PROJECT_NAME: &str = "new_project";
const DEFAULT_CUSTOMIZER_NAME: &str = "customizer";

/// The git_extra CLI
#[derive(Debug, Parser)]
#[clap(version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Browse to origin repository web page
    Browse {
        /// Override name of the origin repository
        #[clap(long)]
        origin: Option<String>,
    },
    /// Quickly start a project by cloning a repo and running a customization script
    QuickStart {
        /// A name or URL of a Git repository to clone
        url_or_name: Option<String>,
        /// Name of the directory to clone the repo into
        directory: Option<String>,
        /// List all available named repositories in your `$HOME/.config/git_extra/repos.toml` file
        #[clap(short, long)]
        list: bool,
    },
}

#[derive(Debug, Deserialize)]
struct ReposFile {
    #[serde(flatten)]
    repos: HashMap<String, RepoEntry>,
}

#[derive(Debug, Deserialize)]
struct RepoEntry {
    origin: String,
    customizer: Option<String>,
}

pub trait GitExtraLog {
    fn output(self: &Self, args: Arguments);
    fn warning(self: &Self, args: Arguments);
    fn error(self: &Self, args: Arguments);
}

pub struct GitExtraTool<'a> {
    log: &'a dyn GitExtraLog,
}

impl<'a> GitExtraTool<'a> {
    pub fn new(log: &'a dyn GitExtraLog) -> GitExtraTool {
        GitExtraTool { log }
    }

    pub fn run(
        self: &mut Self,
        args: impl IntoIterator<Item = std::ffi::OsString>,
    ) -> Result<(), Box<dyn Error>> {
        let matches = match Cli::command().try_get_matches_from(args) {
            Ok(m) => m,
            Err(err) => {
                output!(self.log, "{}", err.to_string());
                return Ok(());
            }
        };
        use clap::FromArgMatches;
        let cli = Cli::from_arg_matches(&matches)?;

        match &cli.command {
            Commands::Browse { origin } => {
                self.browse_to_remote(&origin)?;
            }
            Commands::QuickStart {
                url_or_name,
                directory,
                list,
            } => {
                self.quick_start(url_or_name, directory, *list)?;
            }
        }

        Ok(())
    }

    fn browse_to_remote(self: &Self, origin: &Option<String>) -> Result<(), Box<dyn Error>> {
        let origin_name = match origin {
            Some(s) => s.to_owned(),
            None => "origin".to_string(),
        };
        let output = cmd!("git", "remote", "-vv").read()?;

        for cap_origin in RE_ORIGIN.captures_iter(&output) {
            if &cap_origin["name"] != origin_name {
                continue;
            }

            match RE_SSH
                .captures(&cap_origin["repo"])
                .or(RE_HTTPS.captures(&cap_origin["repo"]))
            {
                Some(cap_repo) => {
                    let url = format!(
                        "https://{}/{}/{}",
                        &cap_repo["domain"], &cap_repo["user"], &cap_repo["project"]
                    );
                    output!(self.log, "Opening URL '{}'", url);
                    opener::open_browser(url)?;
                    return Ok(());
                }
                None => continue,
            }
        }

        Ok(())
    }

    fn read_repos_file(self: &Self) -> Result<ReposFile, Box<dyn Error>> {
        let mut repos_file = PathBuf::from(env::var("HOME")?);

        repos_file.push(".config/git_extra/repos.toml");

        match fs::read_to_string(repos_file.as_path()) {
            Ok(s) => Ok(toml::from_str(&s)?),
            Err(_) => {
                warning!(self.log, "'{}' not found", repos_file.to_string_lossy());
                Ok(ReposFile {
                    repos: HashMap::new(),
                })
            }
        }
    }

    fn quick_start(
        self: &Self,
        opt_url_or_name: &Option<String>,
        opt_dir: &Option<String>,
        list: bool,
    ) -> Result<(), Box<dyn Error>> {
        let file = self.read_repos_file()?;

        if list {
            if !file.repos.is_empty() {
                let width = file.repos.keys().map(|s| s.len()).max().unwrap();

                for (name, entry) in file.repos.iter() {
                    output!(self.log, "{:width$} {}", name, entry.origin);
                }
            }

            return Ok(());
        }

        let url: String;
        let mut customizer_name = DEFAULT_CUSTOMIZER_NAME.to_string();

        if let Some(arg) = opt_url_or_name {
            if RE_SSH.is_match(arg) || RE_HTTPS.is_match(arg) {
                url = arg.to_owned();
            } else if let Some(entry) = file.repos.get(arg) {
                url = entry.origin.to_owned();

                customizer_name = entry
                    .customizer
                    .as_ref()
                    .map_or(customizer_name, |e| e.to_owned());
            } else {
                return Err(From::from(format!("Repository '{}' not found", arg)));
            }
        } else {
            return Err(From::from(format!("Must supply a URL or name")));
        }

        let new_dir_path = PathBuf::from(opt_dir.as_deref().unwrap_or(DEFAULT_PROJECT_NAME));
        let customizer_file_path = new_dir_path.join(&customizer_name);

        cmd!("git", "clone", url, new_dir_path.as_path()).run()?;

        if let Ok(_) = fs::File::open(&customizer_file_path) {
            cmd!(&customizer_file_path, new_dir_path.file_name().unwrap())
                .dir(new_dir_path.as_path())
                .run()?;
        } else {
            warning!(
                self.log,
                "Customization file '{}' not found",
                customizer_file_path.to_string_lossy()
            )
        }

        Ok(())
    }
}
