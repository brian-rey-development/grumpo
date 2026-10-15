use super::*;

const OWN_UID: u32 = 501;
const OTHER_UID: u32 = 502;

#[test]
fn root_is_trusted() {
    assert!(judge_server(Ok(ROOT_UID), OWN_UID).is_ok());
}

#[test]
fn our_own_user_is_trusted_for_development_runs() {
    assert!(judge_server(Ok(OWN_UID), OWN_UID).is_ok());
}

#[test]
fn another_user_is_an_impostor_even_when_we_are_root() {
    for own_uid in [OWN_UID, ROOT_UID] {
        let failure = judge_server(Ok(OTHER_UID), own_uid).unwrap_err();
        assert!(
            matches!(failure, ServerIdentityError::Impostor { uid: OTHER_UID }),
            "{failure:?}"
        );
    }
}

#[test]
fn a_failed_lookup_is_not_trusted() {
    let failure = judge_server(Err(io::Error::other("no credentials")), OWN_UID).unwrap_err();
    assert!(
        matches!(failure, ServerIdentityError::Unverifiable(_)),
        "{failure:?}"
    );
}

#[test]
fn a_server_running_as_this_user_passes_the_real_check() {
    let (client, _server) = UnixStream::pair().unwrap();
    assert_eq!(peer_uid(&client).unwrap(), effective_uid());
    assert!(verify_server(&client).is_ok());
}
