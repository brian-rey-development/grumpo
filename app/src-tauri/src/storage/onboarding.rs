use super::{StorageError, Store};

const SETTING_KEY: &str = "onboarding_completed";
const COMPLETED: &str = "1";
const PENDING: &str = "0";

impl Store {
    /// A missing or unrecognized value reads as not completed, so the worst a damaged setting
    /// does is show the welcome flow once more.
    pub fn onboarding_completed(&self) -> Result<bool, StorageError> {
        Ok(self.setting(SETTING_KEY)?.as_deref() == Some(COMPLETED))
    }

    pub fn set_onboarding_completed(&self, completed: bool) -> Result<(), StorageError> {
        self.set_setting(SETTING_KEY, if completed { COMPLETED } else { PENDING })
    }
}

#[cfg(test)]
mod tests;
