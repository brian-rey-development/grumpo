use std::fs;

use super::*;
use crate::hosts_writer::HostsFile;

fn hosts_with(content: &str) -> (tempfile::TempDir, HostsFile, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("hosts");
    fs::write(&path, content).unwrap();
    (dir, HostsFile::new(path.clone(), false), path)
}

#[test]
fn removes_current_and_legacy_blocks_and_keeps_the_rest() {
    let content = "127.0.0.1 localhost\n\
        # >>> grumpo: managed block, do not edit >>>\n0.0.0.0 a.com\n# <<< grumpo: managed block <<<\n\
        # FOCUS-HOSTS START\n0.0.0.0 b.com\n# FOCUS-HOSTS END\n";
    let (_dir, hosts, path) = hosts_with(content);
    assert!(strip_hosts(&hosts).unwrap().changed);
    assert_eq!(fs::read_to_string(path).unwrap(), "127.0.0.1 localhost\n");
}

#[test]
fn leaves_a_clean_file_untouched() {
    let (_dir, hosts, path) = hosts_with("127.0.0.1 localhost\n");
    assert!(!strip_hosts(&hosts).unwrap().changed);
    assert_eq!(fs::read_to_string(path).unwrap(), "127.0.0.1 localhost\n");
}

#[test]
fn a_missing_file_is_not_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let hosts = HostsFile::new(dir.path().join("hosts"), false);
    assert!(!strip_hosts(&hosts).unwrap().changed);
}

#[test]
fn a_non_utf8_hosts_file_is_stripped_and_its_other_bytes_survive() {
    let mut content = b"127.0.0.1 localhost\n# caf\xE9\n".to_vec();
    content.extend_from_slice(
        b"# >>> grumpo: managed block, do not edit >>>\n0.0.0.0 a.com\n# <<< grumpo: managed block <<<\n",
    );
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("hosts");
    fs::write(&path, content).unwrap();
    let hosts = HostsFile::new(path.clone(), false);
    assert!(strip_hosts(&hosts).unwrap().changed);
    assert_eq!(fs::read(path).unwrap(), b"127.0.0.1 localhost\n# caf\xE9\n");
}
