use core::fmt::Arguments;
use std::error::Error;
use std::path::PathBuf;

use clap::{AppSettings, Parser};
#[derive(Parser)]
#[clap(version, about, long_about = None)]
#[clap(global_setting(AppSettings::NoAutoHelp))]
#[clap(global_setting(AppSettings::NoAutoVersion))]
struct Cli {
    /// The versioning operation to perform
    operation: Option<String>,

    /// Specify the version file explicitly
    #[clap(short, long = "input", parse(from_os_str), value_name = "INPUT_FILE")]
    input_file: Option<PathBuf>,

    /// Actually do the update
    #[clap(short, long)]
    update: bool,
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
        use clap::IntoApp;
        let matches = Cli::command().try_get_matches_from(args)?;

        if matches.is_present("version") {
            output!(self.log, "{}", Cli::command().get_version().unwrap_or(""));
            return Ok(());
        }

        if matches.is_present("help") {
            let mut output = Vec::new();
            let mut cmd = Cli::command();

            cmd.write_help(&mut output).unwrap();

            output!(self.log, "{}", String::from_utf8(output).unwrap());
            return Ok(());
        }

        use clap::FromArgMatches;
        let _cli = Cli::from_arg_matches(&matches)?;

        Ok(())
    }
}
