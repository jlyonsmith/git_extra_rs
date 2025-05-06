mod log_macros;

use clap::{CommandFactory, Parser, Subcommand};
use core::fmt::Arguments;
use duct::cmd;
use easy_error::{self, bail, ResultExt};
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
    static ref RE_FILE: Regex = RegexBuilder::new("^file://.*$").build().unwrap();
}

const DEFAULT_CUSTOMIZER_NAME: &str = "customize.ts";

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
    /// Commands to quickly start projects
    QuickStart {
        /// Quick start sub-commands
        #[clap(subcommand)]
        quick_start: QuickStartCommands,
    },
}

#[derive(Debug, Subcommand)]
enum QuickStartCommands {
    /// List all available named repositories in your `$HOME/.config/git_extra/repos.toml`
    List {
        #[clap(short, long)]
        list: bool,
    },
    /// Create a new project by cloning a repo and running a customization script
    Create {
        /// A name or URL of a Git repository to clone
        url_or_name: String,
        /// Name of the directory to clone the repo into
        directory: String,
        /// The name of the customization file relative to the new project directory
        #[clap(short, long)]
        customizer: Option<String>,
    },
}

#[derive(Debug, Deserialize)]
struct ReposFile {
    #[serde(flatten)]
    repos: HashMap<String, RepoEntry>,
}

#[derive(Debug, Deserialize)]
struct RepoEntry {
    description: Option<String>,
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
    pub fn new(log: &'a dyn GitExtraLog) -> GitExtraTool<'a> {
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
            Commands::QuickStart { quick_start } => match quick_start {
                QuickStartCommands::List { list: _ } => {
                    self.quick_start_list()?;
                }
                QuickStartCommands::Create {
                    url_or_name,
                    directory,
                    customizer,
                } => {
                    self.quick_start_create(url_or_name, directory, customizer)?;
                }
            },
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

    fn quick_start_list(self: &Self) -> Result<(), Box<dyn Error>> {
        let file = self.read_repos_file()?;

        if !file.repos.is_empty() {
            use colored::Colorize;

            let width = file.repos.keys().map(|s| s.len()).max().unwrap() + 3;
            let empty_string = "".to_string();

            for (name, entry) in file.repos.iter() {
                output!(
                    self.log,
                    "{:width$} {}\n{:width$} {}",
                    name,
                    &entry.origin,
                    "",
                    entry
                        .description
                        .as_ref()
                        .unwrap_or(&empty_string)
                        .bright_white(),
                );
            }
        }

        Ok(())
    }

    fn quick_start_create(
        self: &Self,
        opt_url_or_name: &String,
        opt_dir: &String,
        opt_customizer: &Option<String>,
    ) -> Result<(), Box<dyn Error>> {
        let file = self.read_repos_file()?;
        let url: String;

        // Customizer is command line or default
        let mut customizer_file_name = String::new();

        if RE_SSH.is_match(opt_url_or_name) || RE_HTTPS.is_match(opt_url_or_name) {
            url = opt_url_or_name.to_owned();
        } else if RE_FILE.is_match(opt_url_or_name) {
            url = opt_url_or_name.clone().split_off("file://".len());
        } else if let Some(entry) = file.repos.get(opt_url_or_name) {
            url = entry.origin.to_owned();

            // Customizer is command line, file entry or default
            customizer_file_name = opt_customizer.as_ref().map_or(
                entry
                    .customizer
                    .as_ref()
                    .map_or(DEFAULT_CUSTOMIZER_NAME.to_string(), |e| e.to_owned()),
                |e| e.to_owned(),
            );
        } else {
            bail!(
                "Repository name '{}' must start with https://, git@ or file://",
                opt_url_or_name
            );
        }

        if customizer_file_name.is_empty() {
            customizer_file_name = opt_customizer
                .as_ref()
                .map_or(DEFAULT_CUSTOMIZER_NAME.to_string(), |e| e.to_owned());
        }

        let new_dir_path = PathBuf::from(opt_dir);
        let customizer_file_path = new_dir_path.join(&customizer_file_name);

        cmd!("git", "clone", url.as_str(), new_dir_path.as_path())
            .run()
            .context(format!("Unable to run `git clone` for '{}'", url.as_str()))?;

        if let Ok(_) = fs::File::open(&customizer_file_path) {
            output!(self.log, "Running the customization script");

            cmd!(&customizer_file_path, new_dir_path.file_name().unwrap())
                .dir(new_dir_path.as_path())
                .run()
                .context(format!(
                    "There was a problem running customizer file '{}'",
                    customizer_file_path.to_string_lossy()
                ))?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_test() {
        struct TestLogger;

        impl TestLogger {
            fn new() -> TestLogger {
                TestLogger {}
            }
        }

        impl GitExtraLog for TestLogger {
            fn output(self: &Self, _args: Arguments) {}
            fn warning(self: &Self, _args: Arguments) {}
            fn error(self: &Self, _args: Arguments) {}
        }

        let logger = TestLogger::new();
        let mut tool = GitExtraTool::new(&logger);
        let args: Vec<std::ffi::OsString> = vec!["".into(), "--help".into()];

        tool.run(args).unwrap();
    }
}
