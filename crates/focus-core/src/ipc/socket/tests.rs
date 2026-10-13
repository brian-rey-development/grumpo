use super::*;

#[test]
fn defaults_to_the_installed_socket() {
    assert_eq!(socket_path(|_| None), PathBuf::from("/var/run/focusd.sock"));
}

#[test]
fn the_variable_overrides_the_default() {
    let lookup = |name: &str| (name == SOCKET_ENV).then(|| OsString::from("/tmp/k"));
    assert_eq!(socket_path(lookup), PathBuf::from("/tmp/k"));
}

#[test]
fn an_empty_value_falls_back_to_the_default() {
    let lookup = |_: &str| Some(OsString::new());
    assert_eq!(socket_path(lookup), PathBuf::from(DEFAULT_SOCKET_PATH));
}
