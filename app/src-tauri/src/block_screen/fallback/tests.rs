use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;

fn counting() -> (Arc<AtomicUsize>, Fallback) {
    let count = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&count);
    let fallback = Box::new(move || {
        counted.fetch_add(1, Ordering::SeqCst);
    });
    (count, fallback)
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "it stands in for a queue, whose result type is fixed"
)]
fn run_now(job: Box<dyn FnOnce() + Send>) -> Result<(), String> {
    job();
    Ok(())
}

fn refuse(_job: Box<dyn FnOnce() + Send>) -> Result<(), String> {
    Err("the main thread is gone".to_owned())
}

#[test]
fn a_show_that_works_never_runs_the_fallback() {
    let (count, fallback) = counting();

    queue_with_fallback(run_now, || Ok::<(), String>(()), fallback);

    assert_eq!(count.load(Ordering::SeqCst), 0);
}

#[test]
fn a_show_that_fails_on_the_main_thread_runs_the_fallback_once() {
    let (count, fallback) = counting();

    queue_with_fallback(run_now, || Err("no window".to_owned()), fallback);

    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn work_that_cannot_be_queued_runs_the_fallback_once() {
    let (count, fallback) = counting();

    queue_with_fallback(refuse, || Ok::<(), String>(()), fallback);

    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn work_that_is_queued_but_not_run_yet_does_not_run_the_fallback() {
    let (count, fallback) = counting();
    let mut pending = None;

    queue_with_fallback(
        |job| {
            pending = Some(job);
            Ok::<(), String>(())
        },
        || Ok::<(), String>(()),
        fallback,
    );

    assert_eq!(count.load(Ordering::SeqCst), 0);
    assert!(pending.is_some());
}
