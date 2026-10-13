use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use super::test_support::FakeSurface;
use super::*;
use crate::dto::{BlockReason, BlockTarget, UNASSIGNED_ID};

fn screen(domain: &str) -> BlockScreen {
    BlockScreen {
        id: UNASSIGNED_ID,
        target: BlockTarget::Site {
            domain: domain.to_owned(),
        },
        reason: BlockReason::Schedule,
        until: Some(1_800_000_000),
        strict: true,
    }
}

fn stamped(id: u64, domain: &str) -> BlockScreen {
    BlockScreen {
        id,
        ..screen(domain)
    }
}

fn hub_over(surface: &Arc<FakeSurface>) -> BlockScreenHub {
    BlockScreenHub::new(surface.clone())
}

fn ignore() -> Fallback {
    Box::new(|| {})
}

#[test]
fn nothing_is_shown_before_the_first_block() {
    let hub = hub_over(&Arc::default());

    assert_eq!(hub.current(), None);
}

#[test]
fn a_block_is_remembered_and_sent_to_the_window() {
    let surface = Arc::new(FakeSurface::default());
    let hub = hub_over(&surface);

    hub.present(screen("youtube.com"), ignore());

    assert_eq!(hub.current(), Some(stamped(1, "youtube.com")));
    assert_eq!(surface.shown(), [stamped(1, "youtube.com")]);
}

#[test]
fn the_latest_block_replaces_the_content_of_the_window() {
    let surface = Arc::new(FakeSurface::default());
    let hub = hub_over(&surface);

    hub.present(screen("youtube.com"), ignore());
    hub.present(screen("reddit.com"), ignore());

    assert_eq!(hub.current(), Some(stamped(2, "reddit.com")));
    assert_eq!(
        surface.shown(),
        [stamped(1, "youtube.com"), stamped(2, "reddit.com")]
    );
}

#[test]
fn dismissing_clears_the_content_and_hides_the_window() {
    let surface = Arc::new(FakeSurface::default());
    let hub = hub_over(&surface);
    hub.present(screen("youtube.com"), ignore());

    hub.dismiss(None);

    assert_eq!(hub.current(), None);
    assert_eq!(surface.hides(), 1);
}

#[test]
fn a_block_after_a_dismissal_shows_again() {
    let surface = Arc::new(FakeSurface::default());
    let hub = hub_over(&surface);
    hub.present(screen("youtube.com"), ignore());
    hub.dismiss(None);

    hub.present(screen("reddit.com"), ignore());

    assert_eq!(hub.current(), Some(stamped(2, "reddit.com")));
    assert_eq!(surface.shown().len(), 2);
}

#[test]
fn a_window_that_cannot_be_shown_runs_the_fallback_once() {
    let hub = hub_over(&Arc::new(FakeSurface::failing()));
    let fallbacks = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&fallbacks);

    hub.present(
        screen("youtube.com"),
        Box::new(move || {
            counted.fetch_add(1, Ordering::SeqCst);
        }),
    );

    assert_eq!(fallbacks.load(Ordering::SeqCst), 1);
}

#[test]
fn a_window_that_opens_never_runs_the_fallback() {
    let hub = hub_over(&Arc::default());
    let fallbacks = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&fallbacks);

    hub.present(
        screen("youtube.com"),
        Box::new(move || {
            counted.fetch_add(1, Ordering::SeqCst);
        }),
    );

    assert_eq!(fallbacks.load(Ordering::SeqCst), 0);
}

#[test]
fn every_show_gets_a_fresh_increasing_id() {
    let surface = Arc::new(FakeSurface::default());
    let hub = hub_over(&surface);

    hub.present(screen("youtube.com"), ignore());
    hub.present(screen("youtube.com"), ignore());
    hub.dismiss(None);
    hub.present(screen("reddit.com"), ignore());

    let ids: Vec<u64> = surface.shown().iter().map(|shown| shown.id).collect();
    assert_eq!(ids, [1, 2, 3]);
    assert_eq!(hub.current().map(|current| current.id), Some(3));
}

#[test]
fn the_new_content_is_stored_before_the_surface_is_asked_to_show_it() {
    let surface = Arc::new(FakeSurface::default());
    let hub = hub_over(&surface);
    hub.present(screen("youtube.com"), ignore());

    hub.present(screen("reddit.com"), ignore());

    assert_eq!(
        hub.current().map(|current| current.id),
        surface.shown().last().map(|shown| shown.id)
    );
}

#[test]
fn a_dismissal_for_an_earlier_block_leaves_the_newer_one_up() {
    let surface = Arc::new(FakeSurface::default());
    let hub = hub_over(&surface);
    hub.present(screen("youtube.com"), ignore());
    let first = hub.current().unwrap().id;
    hub.present(screen("reddit.com"), ignore());

    hub.dismiss(Some(first));

    assert_eq!(hub.current().map(|current| current.id), Some(first + 1));
    assert_eq!(surface.hides(), 0);
}

#[test]
fn a_dismissal_for_the_current_block_hides_the_window() {
    let surface = Arc::new(FakeSurface::default());
    let hub = hub_over(&surface);
    hub.present(screen("youtube.com"), ignore());
    let current = hub.current().unwrap().id;

    hub.dismiss(Some(current));

    assert_eq!(hub.current(), None);
    assert_eq!(surface.hides(), 1);
}

#[test]
fn a_repeated_dismissal_does_not_hide_again() {
    let surface = Arc::new(FakeSurface::default());
    let hub = hub_over(&surface);
    hub.present(screen("youtube.com"), ignore());
    let current = hub.current().unwrap().id;

    hub.dismiss(Some(current));
    hub.dismiss(Some(current));

    assert_eq!(surface.hides(), 1);
}
