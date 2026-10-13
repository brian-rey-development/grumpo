use std::sync::Mutex;

use super::{Notice, Notifier};

use crate::concurrency::LockExt;

#[derive(Debug, Default)]
pub struct RecordingNotifier {
    notices: Mutex<Vec<Notice>>,
}

impl RecordingNotifier {
    pub fn notices(&self) -> Vec<Notice> {
        self.notices.lock_or_recover().clone()
    }
}

impl Notifier for RecordingNotifier {
    fn notify(&self, notice: Notice) {
        self.notices.lock_or_recover().push(notice);
    }
}
