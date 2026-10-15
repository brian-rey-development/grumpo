use std::os::unix::fs::{symlink, PermissionsExt};

use super::*;

fn hosts_in(dir: &tempfile::TempDir) -> (HostsFile, PathBuf) {
    let path = dir.path().join("hosts");
    (HostsFile::new(path.clone(), false), path)
}

#[test]
fn writes_preserve_the_mode_and_leave_no_temporary_file() {
    let dir = tempfile::tempdir().unwrap();
    let (hosts, path) = hosts_in(&dir);
    fs::write(&path, "127.0.0.1 localhost\n").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
    let _flush = hosts
        .write(b"127.0.0.1 localhost\n0.0.0.0 x.com\n")
        .unwrap();
    assert_eq!(
        hosts.read().unwrap(),
        b"127.0.0.1 localhost\n0.0.0.0 x.com\n"
    );
    assert_eq!(fs::metadata(&path).unwrap().mode() & 0o777, 0o640);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn writes_through_a_symlink_without_replacing_it() {
    let dir = tempfile::tempdir().unwrap();
    let real_dir = dir.path().join("private");
    fs::create_dir(&real_dir).unwrap();
    let real = real_dir.join("hosts");
    fs::write(&real, "old\n").unwrap();
    let link = dir.path().join("hosts");
    symlink(&real, &link).unwrap();
    let hosts = HostsFile::new(link.clone(), false);
    let _flush = hosts.write(b"new\n").unwrap();
    assert!(fs::symlink_metadata(&link)
        .unwrap()
        .file_type()
        .is_symlink());
    assert_eq!(fs::read_to_string(&real).unwrap(), "new\n");
    assert_eq!(hosts.read().unwrap(), b"new\n");
    assert_eq!(fs::read_dir(&real_dir).unwrap().count(), 1);
}

#[test]
fn a_missing_file_is_read_as_not_found_and_created_with_the_default_mode() {
    let dir = tempfile::tempdir().unwrap();
    let (hosts, path) = hosts_in(&dir);
    assert_eq!(hosts.read().unwrap_err().kind(), io::ErrorKind::NotFound);
    let _flush = hosts.write(b"x\n").unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"x\n");
    assert_eq!(fs::metadata(&path).unwrap().mode() & 0o777, 0o644);
}

#[test]
fn a_non_utf8_file_is_read_and_replaced_like_any_other() {
    let dir = tempfile::tempdir().unwrap();
    let (hosts, path) = hosts_in(&dir);
    fs::write(&path, [0x23, 0x20, 0xE9, 0x0A]).unwrap();
    assert_eq!(hosts.read().unwrap(), [0x23, 0x20, 0xE9, 0x0A]);
    let _flush = hosts.write(&[0x23, 0x20, 0xE9, 0x0A, 0x78, 0x0A]).unwrap();
    assert_eq!(
        fs::read(&path).unwrap(),
        [0x23, 0x20, 0xE9, 0x0A, 0x78, 0x0A]
    );
}

#[test]
fn a_flush_is_requested_only_for_the_real_hosts_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("hosts");
    let real = HostsFile::new(path.clone(), true);
    assert_eq!(real.write(b"a\n").unwrap(), PendingFlush::NEEDED);
    let fake = HostsFile::new(path, false);
    assert_eq!(fake.write(b"b\n").unwrap(), PendingFlush::NONE);
}

#[test]
fn stale_temporary_files_beside_the_hosts_file_are_removed() {
    let dir = tempfile::tempdir().unwrap();
    let (hosts, path) = hosts_in(&dir);
    fs::write(&path, "127.0.0.1 localhost\n").unwrap();
    fs::write(dir.path().join(".hosts.4242.7.tmp"), "half").unwrap();

    hosts.remove_stale_temps();

    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    assert_eq!(hosts.read().unwrap(), b"127.0.0.1 localhost\n");
}
