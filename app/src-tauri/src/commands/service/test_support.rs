use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use crate::platform::{InstallOutcome, ServiceInstallError, ServiceInstaller};

#[derive(Debug)]
pub struct FakeInstaller {
    result: Mutex<Option<Result<InstallOutcome, ServiceInstallError>>>,
    runs: AtomicUsize,
}

impl FakeInstaller {
    pub fn ending_with(result: Result<InstallOutcome, ServiceInstallError>) -> Self {
        Self {
            result: Mutex::new(Some(result)),
            runs: AtomicUsize::new(0),
        }
    }

    pub fn runs(&self) -> usize {
        self.runs.load(Ordering::SeqCst)
    }
}

impl ServiceInstaller for FakeInstaller {
    fn install(&self) -> Result<InstallOutcome, ServiceInstallError> {
        self.runs.fetch_add(1, Ordering::SeqCst);
        self.result
            .lock()
            .unwrap()
            .take()
            .expect("the fake installer runs once")
    }
}
