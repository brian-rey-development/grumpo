use super::*;

fn parse_strs(args: &[&str]) -> Result<Command, ArgsError> {
    parse(args.iter().map(OsString::from))
}

#[test]
fn parses_run_with_an_owner() {
    assert_eq!(
        parse_strs(&["run", "--owner-uid", "501"]),
        Ok(Command::Run { owner_uid: 501 })
    );
}

#[test]
fn parses_import_legacy_and_strip_hosts() {
    assert_eq!(
        parse_strs(&["import-legacy", "/usr/local/bin/focus-hosts"]),
        Ok(Command::ImportLegacy {
            script: PathBuf::from("/usr/local/bin/focus-hosts")
        })
    );
    assert_eq!(parse_strs(&["strip-hosts"]), Ok(Command::StripHosts));
}

#[test]
fn rejects_bad_input_with_a_specific_error() {
    let cases: [(&[&str], ArgsError); 11] = [
        (&[], ArgsError::MissingCommand),
        (&["serve"], ArgsError::UnknownCommand("serve".into())),
        (&["run"], ArgsError::Missing("--owner-uid <uid>")),
        (
            &["run", "--owner-uid"],
            ArgsError::Missing("a value for --owner-uid"),
        ),
        (
            &["run", "--owner-uid", "abc"],
            ArgsError::InvalidUid("abc".into()),
        ),
        (
            &["run", "--owner-uid", "-1"],
            ArgsError::InvalidUid("-1".into()),
        ),
        (
            &["run", "--owner", "5"],
            ArgsError::Unexpected("--owner".into()),
        ),
        (
            &["run", "--owner-uid", "5", "x"],
            ArgsError::Unexpected("x".into()),
        ),
        (&["import-legacy"], ArgsError::Missing("<script-path>")),
        (&["run", "--owner-uid", "0"], ArgsError::RootUid),
        (&["run", "--owner-uid", "4294967295"], ArgsError::NoUserUid),
    ];
    for (args, expected) in cases {
        assert_eq!(parse_strs(args), Err(expected), "args {args:?}");
    }
    assert_eq!(
        parse_strs(&["strip-hosts", "--force"]),
        Err(ArgsError::Unexpected("--force".into()))
    );
}

#[test]
fn the_neighbours_of_the_reserved_uids_are_accepted() {
    for uid in [1_u32, 501, u32::MAX - 1] {
        assert_eq!(
            parse_strs(&["run", "--owner-uid", &uid.to_string()]),
            Ok(Command::Run { owner_uid: uid })
        );
    }
}

#[test]
fn ping_takes_no_arguments_and_stays_out_of_the_usage_text() {
    assert_eq!(parse_strs(&["ping"]), Ok(Command::Ping));
    assert_eq!(
        parse_strs(&["ping", "now"]),
        Err(ArgsError::Unexpected("now".into()))
    );
    assert!(!USAGE.contains("ping"));
}

#[test]
fn a_uid_beyond_u32_is_invalid() {
    assert!(matches!(
        parse_strs(&["run", "--owner-uid", "99999999999"]),
        Err(ArgsError::InvalidUid(_))
    ));
}
