#![expect(
    clippy::unwrap_used,
    reason = "test helpers outside #[test] functions are not covered by allow-unwrap-in-tests"
)]

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../focus-core/tests/fixtures/focus-hosts.sh"
);
const MANAGED_HOSTS: &str = "127.0.0.1 localhost\n\
    # >>> grumpo: managed block, do not edit >>>\n0.0.0.0 a.com\n# <<< grumpo: managed block <<<\n";

fn grumpod(args: &[&str], state_dir: &Path, hosts: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grumpod"))
        .args(args)
        .env("GRUMPOD_STATE_DIR", state_dir)
        .env("GRUMPOD_HOSTS_FILE", hosts)
        .env("GRUMPOD_SOCKET", state_dir.join("unused.sock"))
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn bad_arguments_print_usage_and_exit_with_2() {
    let dir = tempfile::tempdir().unwrap();
    for args in [
        &[][..],
        &["bogus"],
        &["run"],
        &["run", "--owner-uid", "x"],
        &["run", "--owner-uid", "0"],
        &["run", "--owner-uid", "4294967295"],
    ] {
        let output = grumpod(args, dir.path(), &dir.path().join("hosts"));
        assert_eq!(output.status.code(), Some(2), "args {args:?}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("usage:"), "{stderr}");
    }
}

#[test]
fn strip_hosts_removes_the_block_and_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let hosts = dir.path().join("hosts");
    fs::write(&hosts, MANAGED_HOSTS).unwrap();

    let first = grumpod(&["strip-hosts"], dir.path(), &hosts);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(fs::read_to_string(&hosts).unwrap(), "127.0.0.1 localhost\n");

    let second = grumpod(&["strip-hosts"], dir.path(), &hosts);
    assert_eq!(second.status.code(), Some(0));
    assert!(stdout(&second).contains("no managed block"));
}

#[test]
fn import_legacy_creates_state_once_and_then_skips() {
    let dir = tempfile::tempdir().unwrap();
    let state_dir = dir.path().join("state");
    let hosts = dir.path().join("hosts");

    let first = grumpod(&["import-legacy", FIXTURE], &state_dir, &hosts);
    assert_eq!(first.status.code(), Some(0), "{first:?}");
    assert!(stdout(&first).starts_with("imported "));
    let saved = fs::read_to_string(state_dir.join("state.json")).unwrap();
    assert!(saved.contains("youtube.com"));

    let second = grumpod(&["import-legacy", FIXTURE], &state_dir, &hosts);
    assert_eq!(second.status.code(), Some(0));
    assert!(stdout(&second).contains("skipped"));
    assert_eq!(
        fs::read_to_string(state_dir.join("state.json")).unwrap(),
        saved
    );
}

#[test]
fn import_legacy_fails_with_1_when_the_script_is_missing() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("nope.sh");
    let output = grumpod(
        &["import-legacy", &missing.to_string_lossy()],
        &dir.path().join("state"),
        &dir.path().join("hosts"),
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(!dir.path().join("state").join("state.json").exists());
}
