use super::*;

#[test]
fn browsers_round_trip_through_their_ids() {
    for original in Browser::ALL {
        assert_eq!(browser(browser_id(original)), original);
    }
}

#[test]
fn permission_kinds_map_onto_platform_permissions() {
    assert_eq!(
        Permission::from(PermissionKind::Accessibility),
        Permission::Accessibility
    );
    assert_eq!(
        Permission::from(PermissionKind::Automation {
            browser: BrowserId::Brave
        }),
        Permission::Automation(Browser::Brave)
    );
}

#[test]
fn states_map_one_to_one() {
    let cases = [
        (PermissionState::Granted, PermissionStatus::Granted),
        (PermissionState::Denied, PermissionStatus::Denied),
        (
            PermissionState::NotDetermined,
            PermissionStatus::NotDetermined,
        ),
        (PermissionState::NotRunning, PermissionStatus::NotRunning),
        (PermissionState::Unsupported, PermissionStatus::Unsupported),
    ];
    for (state, expected) in cases {
        assert_eq!(permission_status(state), expected);
    }
}

#[test]
fn only_not_determined_is_refined_by_what_the_tracker_saw() {
    let denied = true;
    let cases = [
        (
            PermissionState::NotDetermined,
            denied,
            PermissionStatus::Denied,
        ),
        (PermissionState::Granted, denied, PermissionStatus::Granted),
        (
            PermissionState::NotRunning,
            denied,
            PermissionStatus::NotRunning,
        ),
        (
            PermissionState::Unsupported,
            denied,
            PermissionStatus::Unsupported,
        ),
        (
            PermissionState::NotDetermined,
            false,
            PermissionStatus::NotDetermined,
        ),
    ];
    for (probed, observed, expected) in cases {
        assert_eq!(merge_with_health(probed, observed), expected, "{probed:?}");
    }
}

#[test]
fn a_denial_from_the_system_always_wins() {
    for observed in [false, true] {
        assert_eq!(
            merge_with_health(PermissionState::Denied, observed),
            PermissionStatus::Denied
        );
    }
}
