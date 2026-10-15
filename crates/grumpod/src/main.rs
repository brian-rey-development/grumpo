mod args;
mod atomic_write;
mod cli;
mod clock;
mod commands;
mod dns;
mod hosts_writer;
mod instance_lock;
mod log_rotation;
mod paths;
mod reconciler;
mod server;
mod service;
mod shared;
mod state;
mod state_dir;
#[expect(unsafe_code, reason = "dup2 has no stable std API")]
mod sys;

use std::process::ExitCode;

use tracing_subscriber::EnvFilter;

use args::{Command, USAGE};
use paths::Paths;

const EXIT_FAILURE: u8 = 1;
const EXIT_USAGE: u8 = 2;
const DAEMON_LOG_FILTER: &str = "info";
const CLI_LOG_FILTER: &str = "warn";

fn main() -> ExitCode {
    let command = match args::parse(std::env::args_os().skip(1)) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("grumpod: {error}\n\n{USAGE}");
            return ExitCode::from(EXIT_USAGE);
        }
    };
    init_logging(&command);
    match cli::execute(command, &Paths::from_env()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(%error, "grumpod failed");
            ExitCode::from(EXIT_FAILURE)
        }
    }
}

fn init_logging(command: &Command) {
    let default = match command {
        Command::Run { .. } => DAEMON_LOG_FILTER,
        _ => CLI_LOG_FILTER,
    };
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default));
    let installed = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .try_init();
    if let Err(error) = installed {
        eprintln!("grumpod: logging is unavailable: {error}");
    }
}
