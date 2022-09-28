use colored::Colorize;
use core::fmt::Arguments;
use git_extra::{error, GitExtraLog, GitExtraTool};

struct GitExtraLogger;

impl GitExtraLogger {
    fn new() -> GitExtraLogger {
        GitExtraLogger {}
    }
}

impl GitExtraLog for GitExtraLogger {
    fn output(self: &Self, args: Arguments) {
        println!("{}", args);
    }
    fn warning(self: &Self, args: Arguments) {
        eprintln!("{}", format!("warning: {}", args).yellow());
    }
    fn error(self: &Self, args: Arguments) {
        eprintln!("{}", format!("error: {}", args).red());
    }
}

fn main() {
    let logger = GitExtraLogger::new();

    if let Err(error) = GitExtraTool::new(&logger).run(std::env::args_os()) {
        error!(logger, "{}", error);
        std::process::exit(1);
    }
}
