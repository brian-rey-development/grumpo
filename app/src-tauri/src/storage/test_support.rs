use focus_core::test_support::{named_app, site};

use super::sessions::NewSession;
use super::Store;

pub fn store() -> Store {
    Store::open_in_memory().unwrap()
}

/// Inserts a finished session in one call.
pub fn record(store: &Store, bundle_id: &str, name: &str, host: Option<&str>, span: (i64, i64)) {
    let id = store
        .open_session(&NewSession {
            app: named_app(bundle_id, name),
            domain: host.map(site),
            started_at: span.0,
        })
        .unwrap();
    store.extend_session(id, span.1).unwrap();
}
