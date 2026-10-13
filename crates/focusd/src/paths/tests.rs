use super::*;

fn lookup(pairs: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> Option<OsString> {
    move |name| {
        pairs
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| OsString::from(value))
    }
}

#[test]
fn defaults_follow_the_installed_layout() {
    let paths = Paths::from_lookup(lookup(&[]));
    assert_eq!(
        paths.state_dir,
        PathBuf::from("/Library/Application Support/Focus")
    );
    assert_eq!(paths.hosts_file, PathBuf::from("/etc/hosts"));
    assert_eq!(paths.socket, PathBuf::from("/var/run/focusd.sock"));
    assert_eq!(paths.log_file, None);
    assert!(paths.flushes_dns());
}

#[test]
fn each_variable_overrides_its_own_path() {
    let paths = Paths::from_lookup(lookup(&[
        ("FOCUSD_STATE_DIR", "/tmp/s"),
        ("FOCUSD_HOSTS_FILE", "/tmp/h"),
        ("FOCUSD_SOCKET", "/tmp/k"),
        ("FOCUSD_LOG_FILE", "/tmp/l"),
    ]));
    assert_eq!(paths.state_dir, PathBuf::from("/tmp/s"));
    assert_eq!(paths.hosts_file, PathBuf::from("/tmp/h"));
    assert_eq!(paths.socket, PathBuf::from("/tmp/k"));
    assert_eq!(paths.log_file, Some(PathBuf::from("/tmp/l")));
    assert!(!paths.flushes_dns());
}

#[test]
fn empty_values_fall_back_to_defaults() {
    let paths = Paths::from_lookup(lookup(&[("FOCUSD_SOCKET", "")]));
    assert_eq!(paths.socket, PathBuf::from("/var/run/focusd.sock"));
}
