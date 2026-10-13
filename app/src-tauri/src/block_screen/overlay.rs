//! The native half of the block screen takeover. It sizes the window to a whole display and
//! lifts it above the menu bar and the Dock, including over apps in native full screen.
//! Everything here calls into the system UI toolkit, so it runs on the main thread only.

use std::ffi::c_void;

use objc2::rc::Retained;
use objc2_app_kit::{
    NSApplication, NSColor, NSEvent, NSScreen, NSStatusWindowLevel, NSWindow,
    NSWindowCollectionBehavior,
};
use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSSize};
use tauri::window::Color;
use tauri::WebviewWindow;

use super::screen_choice::{self, Point, Rect};

/// Present on every Space, including a full screen app's, never moves with Mission Control,
/// and stays out of the Cmd-Tab style window cycle.
const BEHAVIOR: NSWindowCollectionBehavior = NSWindowCollectionBehavior::CanJoinAllSpaces
    .union(NSWindowCollectionBehavior::FullScreenAuxiliary)
    .union(NSWindowCollectionBehavior::Stationary)
    .union(NSWindowCollectionBehavior::IgnoresCycle);

/// `--color-surface` from `src/design/tokens.css`, which the page paints for each appearance.
const LIGHT_SURFACE: [u8; 3] = [0xff, 0xf6, 0xea];
const DARK_SURFACE: [u8; 3] = [0x26, 0x26, 0x2b];
const MAX_CHANNEL: f64 = 255.0;

#[derive(Debug, thiserror::Error)]
pub enum OverlayError {
    #[error("the block screen can only be restyled on the main thread")]
    NotMainThread,
    #[error("the block screen has no native window")]
    NoNativeWindow,
    #[error("no display to cover")]
    NoDisplay,
    #[error(transparent)]
    Window(#[from] tauri::Error),
}

/// What the window shows before the page paints. It must match the page, or the first frame
/// flashes the wrong shade.
pub fn surface_color() -> Color {
    let [red, green, blue] = surface_rgb(MainThreadMarker::new().is_some_and(is_dark));
    Color(red, green, blue, u8::MAX)
}

/// Covers the display under the cursor and lifts the window above everything but system
/// alerts. The frame is set directly instead of through the window API, which defers it, so
/// the window is already in place when it is ordered front.
pub fn cover_display(
    window: &WebviewWindow,
    main: Option<&WebviewWindow>,
) -> Result<(), OverlayError> {
    let mtm = MainThreadMarker::new().ok_or(OverlayError::NotMainThread)?;
    let native = native_window(mtm, window)?.ok_or(OverlayError::NoNativeWindow)?;
    let main_screen = match main {
        Some(main) => native_window(mtm, main)?.and_then(|native| native.screen()),
        None => None,
    };
    let frame = choose_frame(mtm, main_screen.as_deref()).ok_or(OverlayError::NoDisplay)?;
    native.setFrame_display(frame, false);
    native.setLevel(NSStatusWindowLevel);
    native.setCollectionBehavior(BEHAVIOR);
    native.setHasShadow(false);
    native.setOpaque(true);
    native.setBackgroundColor(Some(&surface(mtm)));
    Ok(())
}

fn choose_frame(mtm: MainThreadMarker, main_screen: Option<&NSScreen>) -> Option<NSRect> {
    let screens: Vec<Rect> = NSScreen::screens(mtm)
        .iter()
        .map(|screen| from_ns(screen.frame()))
        .collect();
    let cursor = NSEvent::mouseLocation();
    let cursor = Point {
        x: cursor.x,
        y: cursor.y,
    };
    screen_choice::choose(cursor, &screens, main_screen.map(|s| from_ns(s.frame()))).map(to_ns)
}

fn native_window(
    mtm: MainThreadMarker,
    window: &WebviewWindow,
) -> Result<Option<Retained<NSWindow>>, OverlayError> {
    let target = window.ns_window()?.cast_const();
    Ok(NSApplication::sharedApplication(mtm)
        .windows()
        .iter()
        .find(|candidate| std::ptr::eq(Retained::as_ptr(candidate).cast::<c_void>(), target)))
}

fn surface(mtm: MainThreadMarker) -> Retained<NSColor> {
    let [red, green, blue] = surface_rgb(is_dark(mtm));
    let channel = |value: u8| f64::from(value) / MAX_CHANNEL;
    NSColor::colorWithSRGBRed_green_blue_alpha(channel(red), channel(green), channel(blue), 1.0)
}

fn surface_rgb(dark: bool) -> [u8; 3] {
    if dark {
        DARK_SURFACE
    } else {
        LIGHT_SURFACE
    }
}

/// Every dark appearance name, vibrant and high contrast ones included, carries the word.
fn is_dark(mtm: MainThreadMarker) -> bool {
    let appearance = NSApplication::sharedApplication(mtm).effectiveAppearance();
    appearance.name().to_string().contains("Dark")
}

fn from_ns(rect: NSRect) -> Rect {
    Rect {
        x: rect.origin.x,
        y: rect.origin.y,
        width: rect.size.width,
        height: rect.size.height,
    }
}

fn to_ns(rect: Rect) -> NSRect {
    NSRect::new(
        NSPoint::new(rect.x, rect.y),
        NSSize::new(rect.width, rect.height),
    )
}

#[cfg(test)]
mod tests;
