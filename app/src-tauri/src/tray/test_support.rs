//! Fakes for the menu bar: a main thread that runs jobs only when told to, and a view that
//! records what it was asked to show and from which thread.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use std::thread;

use super::view::{MainThread, TrayView};

/// Queues jobs and runs none of them until told to, like a main thread that is busy.
#[derive(Default)]
pub struct StalledMain {
    queued: Mutex<Vec<Box<dyn FnOnce() + Send>>>,
}

impl StalledMain {
    pub fn queued(&self) -> usize {
        self.queued.lock().unwrap().len()
    }

    pub fn run_all(&self) {
        let jobs: Vec<_> = self.queued.lock().unwrap().drain(..).collect();
        for job in jobs {
            job();
        }
    }
}

impl MainThread for StalledMain {
    fn post(&self, job: Box<dyn FnOnce() + Send>) {
        self.queued.lock().unwrap().push(job);
    }
}

#[derive(Default)]
pub struct RecordingView {
    pub lines: Mutex<Vec<String>>,
    pub totals: Mutex<Vec<String>>,
    pub relock: Mutex<Vec<bool>>,
    pub quit_labels: Mutex<Vec<String>>,
    pub touched_by_other_threads: AtomicU32,
    owner: Mutex<Option<thread::ThreadId>>,
}

impl RecordingView {
    fn note_caller(&self) {
        let mut owner = self.owner.lock().unwrap();
        let current = thread::current().id();
        if owner.get_or_insert(current) != &current {
            self.touched_by_other_threads.fetch_add(1, Ordering::SeqCst);
        }
    }
}

impl TrayView for RecordingView {
    fn set_status_line(&self, line: &str) {
        self.note_caller();
        self.lines.lock().unwrap().push(line.to_owned());
    }

    fn set_relock_enabled(&self, enabled: bool) {
        self.note_caller();
        self.relock.lock().unwrap().push(enabled);
    }

    fn set_quit_label(&self, label: &str) {
        self.note_caller();
        self.quit_labels.lock().unwrap().push(label.to_owned());
    }

    fn set_total(&self, title: &str) {
        self.note_caller();
        self.totals.lock().unwrap().push(title.to_owned());
    }
}
