use std::fs;

use tempfile::TempDir;

use super::*;

const DATABASE: &str = "focus.db";

struct Gone;

impl LegacyAppProbe for Gone {
    fn is_running(&self) -> bool {
        false
    }
}

struct Running;

impl LegacyAppProbe for Running {
    fn is_running(&self) -> bool {
        true
    }
}

struct Support {
    root: TempDir,
}

impl Support {
    fn new() -> Self {
        Self {
            root: tempfile::tempdir().unwrap(),
        }
    }

    fn legacy(&self) -> PathBuf {
        self.root.path().join(LEGACY_IDENTIFIER)
    }

    fn current(&self) -> PathBuf {
        self.root
            .path()
            .join("io.github.brian-rey-development.grumpo")
    }

    fn write(dir: &Path, name: &str, text: &str) {
        fs::create_dir_all(dir).unwrap();
        fs::write(dir.join(name), text).unwrap();
    }
}

fn read(dir: &Path, name: &str) -> String {
    fs::read_to_string(dir.join(name)).unwrap()
}

#[test]
fn a_fresh_install_has_nothing_to_move_and_creates_nothing() {
    let support = Support::new();

    let used = adopt_legacy_data(&support.current(), &Gone);

    assert_eq!(used, support.current());
    assert!(!support.legacy().exists());
    assert!(!support.current().exists());
}

#[test]
fn old_data_only_moves_across_with_every_file() {
    let support = Support::new();
    Support::write(&support.legacy(), DATABASE, "history");
    Support::write(&support.legacy(), "focus.db-wal", "journal");

    let used = adopt_legacy_data(&support.current(), &Gone);

    assert_eq!(used, support.current());
    assert_eq!(read(&used, DATABASE), "history");
    assert_eq!(read(&used, "focus.db-wal"), "journal");
    assert!(!support.legacy().exists());
}

#[test]
fn both_present_keeps_the_new_folder_and_leaves_the_old_one_alone() {
    let support = Support::new();
    Support::write(&support.legacy(), DATABASE, "old history");
    Support::write(&support.current(), DATABASE, "new history");

    let used = adopt_legacy_data(&support.current(), &Gone);

    assert_eq!(used, support.current());
    assert_eq!(read(&support.current(), DATABASE), "new history");
    assert_eq!(read(&support.legacy(), DATABASE), "old history");
}

#[test]
fn an_empty_new_folder_left_by_an_interrupted_start_is_filled_by_the_move() {
    let support = Support::new();
    Support::write(&support.legacy(), DATABASE, "history");
    fs::create_dir_all(support.current()).unwrap();

    let used = adopt_legacy_data(&support.current(), &Gone);

    assert_eq!(used, support.current());
    assert_eq!(read(&used, DATABASE), "history");
    assert!(!support.legacy().exists());
}

#[test]
fn running_it_again_after_a_move_changes_nothing() {
    let support = Support::new();
    Support::write(&support.legacy(), DATABASE, "history");

    adopt_legacy_data(&support.current(), &Gone);
    let used = adopt_legacy_data(&support.current(), &Gone);

    assert_eq!(used, support.current());
    assert_eq!(read(&used, DATABASE), "history");
}

#[cfg(unix)]
#[test]
fn a_refused_move_keeps_using_the_old_folder_and_retries_on_the_next_start() {
    use std::os::unix::fs::PermissionsExt;

    let support = Support::new();
    Support::write(&support.legacy(), DATABASE, "history");
    let locked = fs::Permissions::from_mode(0o500);
    fs::set_permissions(support.root.path(), locked).unwrap();

    let used = adopt_legacy_data(&support.current(), &Gone);

    fs::set_permissions(support.root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(used, support.legacy());
    assert_eq!(read(&used, DATABASE), "history");

    let retried = adopt_legacy_data(&support.current(), &Gone);
    assert_eq!(retried, support.current());
    assert_eq!(read(&retried, DATABASE), "history");
}

#[test]
fn nothing_moves_while_the_earlier_app_runs_and_the_next_start_finishes_the_job() {
    let support = Support::new();
    Support::write(&support.legacy(), DATABASE, "history");

    let used = adopt_legacy_data(&support.current(), &Running);

    assert_eq!(used, support.legacy());
    assert_eq!(read(&used, DATABASE), "history");
    assert!(
        !support.current().exists(),
        "no fresh folder hides the history"
    );

    let later = adopt_legacy_data(&support.current(), &Gone);
    assert_eq!(later, support.current());
    assert_eq!(read(&later, DATABASE), "history");
    assert!(!support.legacy().exists());
}

#[test]
fn a_running_earlier_app_changes_nothing_when_there_is_no_old_data() {
    let support = Support::new();

    let used = adopt_legacy_data(&support.current(), &Running);

    assert_eq!(used, support.current());
}

#[test]
fn a_running_earlier_app_does_not_undo_the_choice_of_the_new_folder() {
    let support = Support::new();
    Support::write(&support.legacy(), DATABASE, "old history");
    Support::write(&support.current(), DATABASE, "new history");

    let used = adopt_legacy_data(&support.current(), &Running);

    assert_eq!(used, support.current());
}
