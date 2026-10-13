//! Owner-only file creation, so usage history is never readable by other local users.

use std::fs::{DirBuilder, File, OpenOptions};
use std::io;
use std::path::Path;

#[cfg(unix)]
const DIR_MODE: u32 = 0o700;
#[cfg(unix)]
const FILE_MODE: u32 = 0o600;

#[cfg(unix)]
fn restrict_file(options: &mut OpenOptions) -> &mut OpenOptions {
    std::os::unix::fs::OpenOptionsExt::mode(options, FILE_MODE)
}

#[cfg(not(unix))]
fn restrict_file(options: &mut OpenOptions) -> &mut OpenOptions {
    options
}

/// Creates missing directories only, so an existing parent such as `Application Support`
/// keeps its permissions.
pub fn create_dir_all(dir: &Path) -> io::Result<()> {
    let mut builder = DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, DIR_MODE);
    builder.create(dir)
}

/// Creates `path` if absent without touching existing content.
pub fn ensure_file(path: &Path) -> io::Result<()> {
    let mut options = OpenOptions::new();
    restrict_file(options.create(true).append(true)).open(path)?;
    Ok(())
}

/// Opens a fresh file, failing if `path` exists, which also refuses to follow a planted symlink.
pub fn create_new_file(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    restrict_file(options.create_new(true).write(true)).open(path)
}
