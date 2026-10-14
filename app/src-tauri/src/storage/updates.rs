use super::{StorageError, Store};

const SETTING_KEY: &str = "auto_update";
const ENABLED: &str = "1";
const DISABLED: &str = "0";

impl Store {
    pub fn auto_update(&self) -> Result<bool, StorageError> {
        Ok(self.setting(SETTING_KEY)?.as_deref() != Some(DISABLED))
    }

    pub fn set_auto_update(&self, enabled: bool) -> Result<(), StorageError> {
        self.set_setting(SETTING_KEY, if enabled { ENABLED } else { DISABLED })
    }
}

#[cfg(test)]
mod tests;
