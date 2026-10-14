use super::*;

#[test]
fn maps_automation_statuses() {
    assert_eq!(map_automation_status(0), PermissionState::Granted);
    assert_eq!(map_automation_status(-1743), PermissionState::Denied);
    assert_eq!(map_automation_status(-1744), PermissionState::NotDetermined);
    assert_eq!(map_automation_status(-600), PermissionState::NotRunning);
    assert_eq!(map_automation_status(-50), PermissionState::NotDetermined);
}

#[test]
fn untrusted_accessibility_is_denied_so_the_prompt_can_be_offered() {
    assert_eq!(accessibility_state_after(false), PermissionState::Denied);
    assert_eq!(accessibility_state_after(true), PermissionState::Granted);
}

#[test]
fn deep_links_target_the_matching_pane() {
    assert!(pane_url(Permission::Accessibility).ends_with("Privacy_Accessibility"));
    assert!(pane_url(Permission::Automation(Browser::Chrome)).ends_with("Privacy_Automation"));
}

#[test]
fn a_stale_grant_is_reset_for_this_app_and_accessibility_only() {
    assert_eq!(
        accessibility_reset_args("io.github.brian-rey-development.grumpo"),
        [
            "reset",
            "Accessibility",
            "io.github.brian-rey-development.grumpo"
        ]
    );
}
