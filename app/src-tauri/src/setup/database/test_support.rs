use std::sync::Mutex;

use crate::concurrency::LockExt;
use crate::platform::Dialogs;

#[derive(Debug, Default)]
pub struct RecordingDialogs {
    alerts: Mutex<Vec<(String, String)>>,
}

impl RecordingDialogs {
    pub fn alerts(&self) -> Vec<(String, String)> {
        self.alerts.lock_or_recover().clone()
    }
}

impl Dialogs for RecordingDialogs {
    fn alert(&self, title: &str, message: &str) {
        self.alerts
            .lock_or_recover()
            .push((title.to_owned(), message.to_owned()));
    }
}
