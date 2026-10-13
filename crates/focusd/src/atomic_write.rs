use std::fs::{self, File, OpenOptions, Permissions};
use std::io::{self, Write};
use std::os::unix::fs::{fchown, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use tracing::warn;

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
/// one filesystem. Its name is fixed, which is safe because every writer in this daemon holds
/// the instance lock, and a leftover from a crashed run is simply removed first. The file is
/// fsynced before the rename and the directory after it, so a power cut cannot undo the rename
/// once this returns.
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
        let path = directory.join(format!(".{name}.tmp"));
        match fs::remove_file(&path) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error),
            _ => {}
        }
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(mode)
            .open(&path)?;
        Ok((Self { path, kept: false }, file))
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
