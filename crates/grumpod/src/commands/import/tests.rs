use super::*;
use focus_core::test_support::now_at;

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../focus-core/tests/fixtures/focus-hosts.sh"
);

fn now() -> Now {
    now_at(2026, 10, 12, 9, 0)
}

#[test]
fn imports_the_real_legacy_script_into_a_new_state() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileStateStore::open(dir.path()).unwrap();
    let outcome = import_legacy(&store, Path::new(FIXTURE), &now()).unwrap();
    let ImportOutcome::Imported { sites, windows } = outcome else {
        panic!("expected an import, got {outcome:?}");
    };
    assert!(sites > 10);
    assert_eq!(windows, 2);
    let saved = store.load(&now()).unwrap().unwrap();
    assert_eq!(saved.rules.blocked_sites.len(), sites);
    assert_eq!(saved.rules.schedule.windows.len(), 2);
}

#[test]
fn never_overwrites_an_existing_state() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("state.json"), "anything").unwrap();
    let store = FileStateStore::open(dir.path()).unwrap();
    let outcome = import_legacy(&store, Path::new(FIXTURE), &now()).unwrap();
    assert_eq!(outcome, ImportOutcome::AlreadyInitialized);
    assert_eq!(
        fs::read_to_string(dir.path().join("state.json")).unwrap(),
        "anything"
    );
}

#[test]
fn reports_an_unreadable_script_without_creating_state() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileStateStore::open(dir.path()).unwrap();
    let error = import_legacy(&store, &dir.path().join("missing"), &now()).unwrap_err();
    assert!(matches!(error, ImportError::ReadScript { .. }));
    assert!(!dir.path().join("state.json").exists());
}

#[test]
fn rejects_a_script_without_domains() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("script.sh");
    fs::write(&script, "#!/bin/bash\necho hi\n").unwrap();
    let state_dir = dir.path().join("state");
    let store = FileStateStore::open(&state_dir).unwrap();
    let error = import_legacy(&store, &script, &now()).unwrap_err();
    assert!(matches!(error, ImportError::NoDomains(_)));
    assert!(!state_dir.join("state.json").exists());
}
