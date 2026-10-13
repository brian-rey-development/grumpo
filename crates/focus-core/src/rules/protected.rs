/// Apps Grumpo must never block, since quitting any of them would break the Mac or Grumpo itself,
/// each with the name to show for it. Terminal is left out on purpose, because people may
/// want to block it.
pub(super) const PROTECTED_APPS: [(&str, &str); 6] = [
    ("io.github.brian-rey-development.grumpo", "Grumpo"),
    // The app's earlier identifier, which a copy that was never removed still runs under.
    ("io.github.brian-rey-development.focus", "Grumpo"),
    ("com.apple.finder", "Finder"),
    ("com.apple.loginwindow", "The login window"),
    ("com.apple.systempreferences", "System Settings"),
    ("com.apple.dock", "The Dock"),
];

/// Whether `bundle_id` names a protected app, ignoring case like the system does.
pub fn is_protected(bundle_id: &str) -> bool {
    find(bundle_id).is_some()
}

pub(super) fn display_name(bundle_id: &str) -> &str {
    find(bundle_id).map_or(bundle_id, |(_, name)| name)
}

fn find(bundle_id: &str) -> Option<&'static (&'static str, &'static str)> {
    PROTECTED_APPS
        .iter()
        .find(|(id, _)| id.eq_ignore_ascii_case(bundle_id))
}

#[cfg(test)]
mod tests;
