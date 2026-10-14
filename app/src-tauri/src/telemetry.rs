use std::any::Any;
use std::env;
use std::fs;
use std::io;
use std::panic;
use std::path::{Path, PathBuf};

use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_appender::rolling::{InitError, RollingFileAppender, Rotation};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

const DEFAULT_LOG_FILTER: &str = "info";
const LOG_DIR_FROM_HOME: &str = "Library/Logs/Grumpo";
const LOG_FILE_PREFIX: &str = "grumpo";
const LOG_FILE_SUFFIX: &str = "log";
const MAX_LOG_FILES: usize = 7;

#[derive(Debug, thiserror::Error)]
enum FileLogError {
    #[error("HOME is not set")]
    NoHome,
    #[error(transparent)]
    Appender(#[from] InitError),
    #[error("could not create the log folder: {0}")]
    CreateDir(#[source] io::Error),
}

/// Installs the global subscriber. The returned guard flushes the file writer on drop, so the
/// caller must hold it until the app exits. `None` means file logging is unavailable.
#[must_use = "dropping the guard stops file logging"]
pub fn init() -> Option<WorkerGuard> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_FILTER));
    let (file_layer, guard, file_error) = match file_writer() {
        Ok((writer, guard)) => (
            Some(fmt::layer().with_ansi(false).with_writer(writer)),
            Some(guard),
            None,
        ),
        Err(error) => (None, None, Some(error)),
    };
    let stderr_layer = cfg!(debug_assertions).then(|| fmt::layer().with_writer(io::stderr));

    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(stderr_layer)
        .init();

    if let Some(error) = file_error {
        tracing::warn!(%error, "file logging is unavailable");
    }
    guard
}

/// Sends panics to the log before the default hook runs. Without it a panic in a worker
/// thread of the shipped app leaves no trace, because nothing is attached to its stderr. The
/// payload can hold a domain or app name, so only the location is logged at error level.
pub fn install_panic_hook() {
    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let location = info.location().map(ToString::to_string);
        log_panic(location.as_deref(), payload_text(info.payload()));
        previous(info);
    }));
}

fn log_panic(location: Option<&str>, message: &str) {
    tracing::error!(location = location.unwrap_or("unknown"), "the app panicked");
    tracing::debug!(message, "the panic payload");
}

fn payload_text(payload: &(dyn Any + Send)) -> &str {
    payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("a panic with no text")
}

fn file_writer() -> Result<(NonBlocking, WorkerGuard), FileLogError> {
    file_writer_in(&log_dir()?)
}

/// The appender lists the folder when it starts and fails if it is missing, which it is on
/// the very first run, so the folder is created first.
fn file_writer_in(dir: &Path) -> Result<(NonBlocking, WorkerGuard), FileLogError> {
    fs::create_dir_all(dir).map_err(FileLogError::CreateDir)?;
    let appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix(LOG_FILE_PREFIX)
        .filename_suffix(LOG_FILE_SUFFIX)
        .max_log_files(MAX_LOG_FILES)
        .build(dir)?;
    Ok(tracing_appender::non_blocking(appender))
}

fn log_dir() -> Result<PathBuf, FileLogError> {
    let home = env::var_os("HOME").ok_or(FileLogError::NoHome)?;
    Ok(PathBuf::from(home).join(LOG_DIR_FROM_HOME))
}

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
