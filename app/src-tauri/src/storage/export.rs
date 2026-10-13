use std::collections::BTreeMap;
use std::fs;
use std::io::{BufWriter, ErrorKind, IntoInnerError, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;

use super::{migrations, private_fs, StorageError, Store};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportedSession {
    started_at: i64,
    ended_at: i64,
    app_id: String,
    app_name: String,
    domain: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Export {
    exported_at: i64,
    schema_version: u32,
    settings: BTreeMap<String, String>,
    sessions: Vec<ExportedSession>,
}

impl Store {
    /// Writes every session and setting to `path` as JSON, atomically and readable only by
    /// the owner. Rows are copied out first so the database lock is not held during disk I/O.
    pub fn export_json(&self, path: &Path, exported_at: i64) -> Result<(), StorageError> {
        let export = Export {
            exported_at,
            schema_version: migrations::latest_version(),
            settings: self.all_settings()?,
            sessions: self.all_sessions()?,
        };
        write_atomically(path, &export)
    }

    fn all_sessions(&self) -> Result<Vec<ExportedSession>, StorageError> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT started_at, ended_at, app_id, app_name, domain
                 FROM sessions ORDER BY started_at, id",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(ExportedSession {
                    started_at: row.get(0)?,
                    ended_at: row.get(1)?,
                    app_id: row.get(2)?,
                    app_name: row.get(3)?,
                    domain: row.get(4)?,
                })
            })?;
            Ok(rows.collect::<Result<_, _>>()?)
        })
    }
}

fn write_atomically(path: &Path, export: &Export) -> Result<(), StorageError> {
    let temp = temp_path(path);
    discard(&temp);
    let result = write_and_rename(&temp, path, export);
    if result.is_err() {
        discard(&temp);
    }
    result
}

fn discard(temp: &Path) {
    if let Err(error) = fs::remove_file(temp) {
        if error.kind() != ErrorKind::NotFound {
            tracing::debug!(%error, "could not remove a temporary export file");
        }
    }
}

fn write_and_rename(temp: &Path, target: &Path, export: &Export) -> Result<(), StorageError> {
    let file = private_fs::create_new_file(temp).map_err(StorageError::io("create export file"))?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, export)?;
    writer
        .write_all(b"\n")
        .map_err(StorageError::io("write export"))?;
    let file = writer
        .into_inner()
        .map_err(IntoInnerError::into_error)
        .map_err(StorageError::io("flush export"))?;
    file.sync_all().map_err(StorageError::io("sync export"))?;
    fs::rename(temp, target).map_err(StorageError::io("replace export file"))
}

fn temp_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".tmp");
    path.with_file_name(name)
}

#[cfg(test)]
mod tests;
