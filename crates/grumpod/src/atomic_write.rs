use std::fs::{self, File, OpenOptions, Permissions};
use std::io::{self, Write};
use std::os::unix::fs::{fchown, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};

use tracing::warn;

const TEMP_SUFFIX: &str = ".tmp";
const MAX_NAME_ATTEMPTS: u32 = 16;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Owner {
    pub uid: u32,
    pub gid: u32,
}

impl Owner {
    pub fn of(path: &Path) -> io::Result<Self> {
        let metadata = fs::metadata(path)?;
        Ok(Self {
            uid: metadata.uid(),
            gid: metadata.gid(),
        })
    }
}

/// Replaces `target` with `bytes` so a reader sees the old file or the new one, never a mix.
///
/// The temporary file lives in the target's directory because rename is only atomic within
/// one filesystem. Its name is unique to this process and call and is created exclusively
/// without following symlinks, so two writers can never share or hijack one, whatever the
/// callers' locking does. A crash can leave a temporary file behind, which
/// [`remove_stale_temps`] clears. The file is fsynced before the rename and the directory after
/// it, so a power cut cannot undo the rename once this returns.
pub fn write_atomic(
    target: &Path,
    bytes: &[u8],
    mode: u32,
    owner: Option<Owner>,
) -> io::Result<()> {
    write_atomic_with(target, bytes, mode, owner, sync_directory)
}

fn write_atomic_with(
    target: &Path,
    bytes: &[u8],
    mode: u32,
    owner: Option<Owner>,
    sync_directory: impl FnOnce(&Path) -> io::Result<()>,
) -> io::Result<()> {
    let directory = target
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "target has no directory"))?;
    let (temp, file) = TempFile::create(directory, target, mode)?;
    fill(file, bytes, mode, owner)?;
    fs::rename(temp.path(), target)?;
    temp.keep();
    if let Err(error) = sync_directory(directory) {
        warn!(
            directory = %directory.display(),
            %error,
            "could not fsync the directory, the new file is in place but may not survive a power cut"
        );
    }
    Ok(())
}

fn sync_directory(directory: &Path) -> io::Result<()> {
    File::open(directory)?.sync_all()
}

struct TempFile {
    path: PathBuf,
    kept: bool,
}

impl TempFile {
    fn create(directory: &Path, target: &Path, mode: u32) -> io::Result<(Self, File)> {
        let name = target.file_name().unwrap_or_default().to_string_lossy();
        for _ in 0..MAX_NAME_ATTEMPTS {
            let path = directory.join(unique_temp_name(&name));
            match open_exclusive(&path, mode) {
                Ok(file) => return Ok((Self { path, kept: false }, file)),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "no free temporary file name",
        ))
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn keep(mut self) {
        self.kept = true;
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        if self.kept {
            return;
        }
        if let Err(error) = fs::remove_file(&self.path) {
            warn!(path = %self.path.display(), %error, "could not remove the temporary file");
        }
    }
}

fn unique_temp_name(name: &str) -> String {
    let count = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!(".{name}.{}.{count}{TEMP_SUFFIX}", process::id())
}

/// `O_EXCL` already refuses an existing path, symlinks included, and `O_NOFOLLOW` states the same
/// intent on the open itself, so a later change to the other flags cannot make it follow one.
fn open_exclusive(path: &Path, mode: u32) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .custom_flags(libc::O_NOFOLLOW)
        .mode(mode)
        .open(path)
}

/// Removes the temporary files that an interrupted write of `target` left in its directory,
/// including the fixed-name one earlier versions used, and returns how many it removed.
///
/// Callers must hold the instance lock, because a live writer's temporary file looks exactly
/// like a stale one.
pub fn remove_stale_temps(target: &Path) -> usize {
    let (Some(directory), Some(name)) = (target.parent(), target.file_name()) else {
        return 0;
    };
    let Ok(entries) = fs::read_dir(directory) else {
        return 0;
    };
    let name = name.to_string_lossy();
    entries
        .filter_map(Result::ok)
        .filter(|entry| is_temp_name(&entry.file_name().to_string_lossy(), &name))
        .filter(|entry| remove_temp(&entry.path()))
        .count()
}

fn is_temp_name(candidate: &str, name: &str) -> bool {
    let Some(middle) = candidate
        .strip_prefix('.')
        .and_then(|rest| rest.strip_prefix(name))
        .and_then(|rest| rest.strip_suffix(TEMP_SUFFIX))
    else {
        return false;
    };
    middle.is_empty() || is_pid_and_count(middle)
}

fn is_pid_and_count(text: &str) -> bool {
    text.strip_prefix('.')
        .and_then(|rest| rest.split_once('.'))
        .is_some_and(|(pid, count)| is_number(pid) && is_number(count))
}

fn is_number(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())
}

fn remove_temp(path: &Path) -> bool {
    match fs::remove_file(path) {
        Ok(()) => {
            warn!(path = %path.display(), "removed a temporary file left by an interrupted write");
            true
        }
        Err(error) => {
            warn!(path = %path.display(), %error, "could not remove a stale temporary file");
            false
        }
    }
}

fn fill(mut file: File, bytes: &[u8], mode: u32, owner: Option<Owner>) -> io::Result<()> {
    file.set_permissions(Permissions::from_mode(mode))?;
    if let Some(owner) = owner {
        chown_if_needed(&file, owner)?;
    }
    file.write_all(bytes)?;
    file.sync_all()
}

fn chown_if_needed(file: &File, owner: Owner) -> io::Result<()> {
    let metadata = file.metadata()?;
    if metadata.uid() == owner.uid && metadata.gid() == owner.gid {
        return Ok(());
    }
    fchown(file, Some(owner.uid), Some(owner.gid))
}

#[cfg(test)]
mod tests;
