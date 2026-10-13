use super::*;

const fn rect(x: f64, y: f64, width: f64, height: f64) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}

const fn at(x: f64, y: f64) -> Point {
    Point { x, y }
}

const PRIMARY: Rect = rect(0.0, 0.0, 1440.0, 900.0);
const RIGHT: Rect = rect(1440.0, 0.0, 2560.0, 1440.0);
const ABOVE: Rect = rect(-200.0, 900.0, 1920.0, 1080.0);

#[test]
fn the_display_under_the_cursor_wins() {
    let screens = [PRIMARY, RIGHT, ABOVE];

    assert_eq!(choose(at(2000.0, 700.0), &screens, None), Some(RIGHT));
    assert_eq!(choose(at(10.0, 10.0), &screens, None), Some(PRIMARY));
}

#[test]
fn a_display_arranged_above_or_left_of_the_primary_one_is_found() {
    let screens = [PRIMARY, RIGHT, ABOVE];

    assert_eq!(choose(at(-150.0, 1500.0), &screens, None), Some(ABOVE));
}

#[test]
fn the_cursor_beats_the_main_window() {
    let screens = [PRIMARY, RIGHT];

    assert_eq!(
        choose(at(2000.0, 700.0), &screens, Some(PRIMARY)),
        Some(RIGHT)
    );
}

#[test]
fn the_top_row_of_pixels_still_belongs_to_its_display() {
    let screens = [PRIMARY, RIGHT];

    assert_eq!(choose(at(2000.0, 1440.0), &screens, None), Some(RIGHT));
}

#[test]
fn a_cursor_outside_every_display_falls_back_to_the_main_window() {
    let screens = [PRIMARY, RIGHT];

    assert_eq!(
        choose(at(-5000.0, -5000.0), &screens, Some(RIGHT)),
        Some(RIGHT)
    );
}

#[test]
fn without_a_main_window_the_primary_display_is_used() {
    let screens = [PRIMARY, RIGHT];

    assert_eq!(choose(at(-5000.0, -5000.0), &screens, None), Some(PRIMARY));
}

#[test]
fn no_displays_and_no_main_window_leave_nothing_to_cover() {
    assert_eq!(choose(at(0.0, 0.0), &[], None), None);
}

#[test]
fn the_main_window_is_used_even_when_no_display_is_listed() {
    assert_eq!(choose(at(0.0, 0.0), &[], Some(RIGHT)), Some(RIGHT));
}
