use clap::{CommandFactory, Parser, Subcommand};
use core::fmt::Arguments;
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use std::error::Error;
use std::process::Command;

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
}

pub trait GitExtraLog {
    fn output(self: &Self, args: Arguments);
    fn warning(self: &Self, args: Arguments);
    fn error(self: &Self, args: Arguments);
}

#[macro_export]
macro_rules! output {
  ($log: expr, $fmt: expr) => {
    $log.output(format_args!($fmt))
  };
  ($log: expr, $fmt: expr, $($args: tt)+) => {
    $log.output(format_args!($fmt, $($args)+))
  };
}
#[macro_export]
macro_rules! warning {
  ($log: expr, $fmt: expr) => {
    $log.warning(format_args!($fmt))
  };
  ($log: expr, $fmt: expr, $($args: tt)+) => {
    $log.warning(format_args!($fmt, $($args)+))
  };
}

#[macro_export]
macro_rules! error {
  ($log: expr, $fmt: expr) => {
    $log.error(format_args!($fmt))
  };
  ($log: expr, $fmt: expr, $($args: tt)+) => {
    $log.error(format_args!($fmt, $($args)+))
  };
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
                let remote_name = match origin {
                    Some(s) => s.to_owned(),
                    None => "origin".to_string(),
                };

                self.get_remote(&remote_name)?;
            }
        }

        Ok(())
    }

    fn get_remote(self: &Self, origin_name: &str) -> Result<(), Box<dyn Error>> {
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
        let output = Command::new("git").arg("remote").arg("-vv").output()?;
        let text = String::from_utf8_lossy(&output.stdout).to_string();

        for cap_origin in RE_ORIGIN.captures_iter(&text) {
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
}
