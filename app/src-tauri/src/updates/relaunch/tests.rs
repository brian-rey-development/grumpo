use std::path::{Path, PathBuf};

use super::bundle_of;

#[test]
fn finds_the_bundle_three_levels_up() {
    assert_eq!(
        bundle_of(Path::new("/Applications/Grumpo.app/Contents/MacOS/Grumpo")),
        Some(PathBuf::from("/Applications/Grumpo.app"))
    );
}

#[test]
fn a_bare_binary_has_no_bundle() {
    assert_eq!(bundle_of(Path::new("/usr/local/bin/grumpo")), None);
    assert_eq!(
        bundle_of(Path::new("/tmp/target/release/deps/grumpo")),
        None
    );
}
