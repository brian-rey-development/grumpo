//! Which display the block screen covers. Every value is in the system's global space, where
//! the origin is the bottom left of the primary display, so no scale factors are involved.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    /// The edges count as inside, because the cursor reports the full height of a display
    /// while it rests on the top row of pixels.
    fn contains(self, point: Point) -> bool {
        (self.x..=self.x + self.width).contains(&point.x)
            && (self.y..=self.y + self.height).contains(&point.y)
    }
}

/// The display under the cursor, else the one the main window is on, else the primary one,
/// which the system lists first.
pub fn choose(cursor: Point, screens: &[Rect], main_window_screen: Option<Rect>) -> Option<Rect> {
    screens
        .iter()
        .copied()
        .find(|screen| screen.contains(cursor))
        .or(main_window_screen)
        .or_else(|| screens.first().copied())
}

#[cfg(test)]
mod tests;
