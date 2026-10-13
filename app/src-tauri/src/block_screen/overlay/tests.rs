use std::path::Path;

use super::*;

const SURFACE_TOKEN: &str = "--color-surface:";
const DARK_BLOCK: &str = "prefers-color-scheme: dark";
const HEX_DIGITS: usize = 6;

fn tokens() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/design/tokens.css");
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn surface_after(css: &str, marker: &str) -> [u8; 3] {
    let from = css
        .find(marker)
        .unwrap_or_else(|| panic!("no {marker} in tokens.css"));
    let rest = &css[from..];
    let declaration =
        &rest[rest.find(SURFACE_TOKEN).expect("no surface token") + SURFACE_TOKEN.len()..];
    let value = declaration[..declaration.find(';').expect("unterminated declaration")].trim();
    let hex = value
        .strip_prefix('#')
        .unwrap_or_else(|| panic!("{value} is not a hex colour"));
    assert_eq!(hex.len(), HEX_DIGITS, "{value}");
    let channel = |at: usize| u8::from_str_radix(&hex[at..at + 2], 16).unwrap();
    [channel(0), channel(2), channel(4)]
}

#[test]
fn the_light_surface_is_the_one_the_page_paints() {
    assert_eq!(LIGHT_SURFACE, surface_after(&tokens(), ":root"));
}

#[test]
fn the_dark_surface_is_the_one_the_page_paints() {
    assert_eq!(DARK_SURFACE, surface_after(&tokens(), DARK_BLOCK));
}

#[test]
fn the_window_color_before_the_page_paints_follows_the_appearance() {
    assert_eq!(surface_rgb(false), LIGHT_SURFACE);
    assert_eq!(surface_rgb(true), DARK_SURFACE);
}
