//! Guards against the places that list the commands drifting apart: the app manifest in
//! `build.rs`, the capability files, the invoke handler in `lib.rs` and the typed wrappers the
//! UI calls them through in `commands.ts`.

const MAIN_COMMANDS: [&str; 23] = [
    "get_day_report",
    "get_week_report",
    "get_limit_usage",
    "get_focus_status",
    "save_rules",
    "request_unlock",
    "confirm_unlock",
    "cancel_unlock",
    "relock",
    "list_known_apps",
    "get_app_icon",
    "get_settings",
    "update_settings",
    "get_permissions",
    "request_permission",
    "open_permission_settings",
    "export_data",
    "delete_all_data",
    "get_clock_style",
    "install_service",
    "get_onboarding_state",
    "complete_onboarding",
    "reset_onboarding",
];

const BLOCKER_COMMANDS: [&str; 2] = ["get_block_screen", "dismiss_block_screen"];

const BUILD_SCRIPT: &str = include_str!("../../build.rs");
const MAIN_CAPABILITY: &str = include_str!("../../capabilities/commands.json");
const BLOCKER_CAPABILITY: &str = include_str!("../../capabilities/blocker.json");
const LIB: &str = include_str!("../lib.rs");
const UI_COMMANDS: &str = include_str!("../../../src/ipc/commands.ts");

fn all_commands() -> impl Iterator<Item = &'static str> {
    MAIN_COMMANDS.into_iter().chain(BLOCKER_COMMANDS)
}

fn permission(name: &str) -> String {
    format!("\"allow-{}\"", name.replace('_', "-"))
}

/// The keys of the `CommandMap` interface, which are the lines shaped `name: { args: ... }`.
fn ui_command_names() -> Vec<String> {
    UI_COMMANDS
        .lines()
        .filter_map(|line| {
            let (name, rest) = line.trim().split_once(": { args:")?;
            let is_name = name.chars().all(|c| c.is_ascii_lowercase() || c == '_');
            (is_name && !name.is_empty() && rest.contains("result:")).then(|| name.to_owned())
        })
        .collect()
}

#[test]
fn every_command_is_in_the_app_manifest() {
    for name in all_commands() {
        assert!(BUILD_SCRIPT.contains(&format!("\"{name}\"")), "{name}");
    }
}

#[test]
fn every_main_command_is_granted_to_the_main_window() {
    for name in MAIN_COMMANDS {
        assert!(MAIN_CAPABILITY.contains(&permission(name)), "{name}");
    }
}

#[test]
fn the_block_screen_commands_are_granted_to_the_block_screen_window_only() {
    for name in BLOCKER_COMMANDS {
        assert!(BLOCKER_CAPABILITY.contains(&permission(name)), "{name}");
        assert!(!MAIN_CAPABILITY.contains(&permission(name)), "{name}");
    }
    for name in MAIN_COMMANDS {
        assert!(!BLOCKER_CAPABILITY.contains(&permission(name)), "{name}");
    }
}

#[test]
fn every_command_is_registered_with_the_invoke_handler() {
    for name in all_commands() {
        assert!(LIB.contains(&format!("::{name},")), "{name}");
    }
}

#[test]
fn the_manifest_lists_no_command_more_than_once() {
    for name in all_commands() {
        let needle = format!("\"{name}\"");
        assert_eq!(BUILD_SCRIPT.matches(&needle).count(), 1, "{name}");
    }
}

#[test]
fn the_ui_wrappers_list_exactly_the_same_commands() {
    let mut in_ui = ui_command_names();
    let mut expected: Vec<String> = all_commands().map(str::to_owned).collect();
    in_ui.sort();
    expected.sort();

    assert_eq!(in_ui.len(), expected.len(), "{in_ui:?}");
    assert_eq!(in_ui, expected);
}
