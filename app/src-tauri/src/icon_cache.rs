//! Bounded cache of rendered app icons, so scrolling a long app list does not render the
//! same bundle again.

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

use crate::concurrency::LockExt;

pub const ICON_CACHE_CAPACITY: usize = 256;

/// A looked-up icon. `Missing` is cached too, since an app without an icon will not grow one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Icon {
    Found(String),
    Missing,
}

impl Icon {
    pub fn into_data_url(self) -> Option<String> {
        match self {
            Self::Found(data_url) => Some(data_url),
            Self::Missing => None,
        }
    }
}

#[derive(Debug)]
pub struct IconCache {
    capacity: usize,
    inner: Mutex<Entries>,
}

#[derive(Debug, Default)]
struct Entries {
    icons: HashMap<String, Icon>,
    recency: VecDeque<String>,
}

impl Default for IconCache {
    fn default() -> Self {
        Self::new(ICON_CACHE_CAPACITY)
    }
}

impl IconCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            inner: Mutex::default(),
        }
    }

    pub fn get(&self, bundle_id: &str) -> Option<Icon> {
        let mut entries = self.lock();
        let icon = entries.icons.get(bundle_id).cloned()?;
        entries.touch(bundle_id);
        Some(icon)
    }

    pub fn insert(&self, bundle_id: &str, icon: Icon) {
        let mut entries = self.lock();
        if entries.icons.insert(bundle_id.to_owned(), icon).is_some() {
            entries.touch(bundle_id);
            return;
        }
        entries.recency.push_back(bundle_id.to_owned());
        while entries.recency.len() > self.capacity {
            if let Some(oldest) = entries.recency.pop_front() {
                entries.icons.remove(&oldest);
            }
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Entries> {
        self.inner.lock_or_recover()
    }
}

impl Entries {
    fn touch(&mut self, bundle_id: &str) {
        if let Some(position) = self.recency.iter().position(|key| key == bundle_id) {
            if let Some(key) = self.recency.remove(position) {
                self.recency.push_back(key);
            }
        }
    }
}

#[cfg(test)]
mod tests;
