//! Opening the app's database at start. A damaged file is replaced and the person is told once,
//! and a database from a newer Grumpo stops the app with a dialog, because running an old
//! schema over newer data could lose it.

use std::path::Path;

use crate::notify::{Notice, Notifier};
use crate::platform::Dialogs;
use crate::storage::{open_or_recover, Opened, StorageError, Store};

const RESET_TITLE: &str = "Usage history was reset";
const RESET_BODY: &str =
    "Grumpo couldn't read its history file, so it started a new one. The old file is still in the Grumpo data folder.";
const NEWER_TITLE: &str = "Update Grumpo";
const NEWER_MESSAGE: &str = "This Mac has usage history from a newer version of Grumpo. Install the latest version of Grumpo to keep using it.";
const FAILED_TITLE: &str = "Grumpo can't start";
const FAILED_MESSAGE: &str = "Grumpo couldn't open its usage history. Open Grumpo again, and if that doesn't help, the log in Library/Logs/Grumpo says what went wrong.";

pub fn open_database(
    path: &Path,
    dialogs: &dyn Dialogs,
    notifier: &dyn Notifier,
    now_unix: i64,
) -> Result<Store, StorageError> {
    match open_or_recover(path, now_unix) {
        Ok(Opened::Ready(store)) => Ok(store),
        Ok(Opened::Recovered { store, kept_as }) => {
            tracing::warn!(kept_as = %kept_as.display(), "started a new database");
            notifier.notify(Notice {
                title: RESET_TITLE.to_owned(),
                body: RESET_BODY.to_owned(),
            });
            Ok(store)
        }
        Err(error) => {
            tracing::error!(%error, "could not open the database");
            dialogs.alert(alert_title(&error), alert_message(&error));
            Err(error)
        }
    }
}

fn alert_title(error: &StorageError) -> &'static str {
    match error {
        StorageError::SchemaTooNew { .. } => NEWER_TITLE,
        _ => FAILED_TITLE,
    }
}

fn alert_message(error: &StorageError) -> &'static str {
    match error {
        StorageError::SchemaTooNew { .. } => NEWER_MESSAGE,
        _ => FAILED_MESSAGE,
    }
}

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
