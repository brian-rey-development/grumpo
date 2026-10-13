use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use super::*;
use crate::enforcement::test_support::in_app;

const SHORT: Duration = Duration::from_millis(30);

#[test]
fn a_posted_sample_is_delivered_with_its_time() {
    let mailbox = Mailbox::default();
    mailbox.post_sample(Some(&in_app("com.a")), 7);

    let Wake::Batch(batch) = mailbox.wait(Some(SHORT)) else {
        panic!("expected a batch");
    };

    assert_eq!(batch.sample.map(|s| s.at), Some(7));
    assert_eq!(batch.usage_written_at, None);
}

#[test]
fn a_newer_sample_replaces_one_the_worker_never_reached() {
    let mailbox = Mailbox::default();
    mailbox.post_sample(Some(&in_app("com.a")), 1);
    mailbox.post_sample(Some(&in_app("com.b")), 2);
    mailbox.post_sample(None, 3);

    let Wake::Batch(batch) = mailbox.wait(Some(SHORT)) else {
        panic!("expected a batch");
    };

    assert_eq!(
        batch.sample,
        Some(SampleMessage {
            activity: None,
            at: 3
        })
    );
    assert_eq!(mailbox.dropped(), 2);
}

#[test]
fn usage_notices_and_samples_arrive_together_and_only_once() {
    let mailbox = Mailbox::default();
    mailbox.post_usage_written(5);
    mailbox.post_sample(None, 6);

    let first = mailbox.wait(Some(SHORT));

    assert!(matches!(
        first,
        Wake::Batch(Batch {
            sample: Some(_),
            usage_written_at: Some(5)
        })
    ));
    assert_eq!(mailbox.wait(Some(SHORT)), Wake::Idle);
}

#[test]
fn waiting_with_nothing_posted_times_out_as_idle() {
    let mailbox = Mailbox::default();
    let started = Instant::now();

    assert_eq!(mailbox.wait(Some(SHORT)), Wake::Idle);
    assert!(started.elapsed() >= SHORT);
}

#[test]
fn a_post_from_another_thread_wakes_a_waiting_worker() {
    let mailbox = Arc::new(Mailbox::default());
    let poster = {
        let mailbox = Arc::clone(&mailbox);
        thread::spawn(move || {
            thread::sleep(SHORT);
            mailbox.post_usage_written(9);
        })
    };

    let woke = mailbox.wait(None);

    poster.join().unwrap();
    assert!(matches!(
        woke,
        Wake::Batch(Batch {
            usage_written_at: Some(9),
            ..
        })
    ));
}

#[test]
fn closing_wakes_the_worker_and_stays_closed() {
    let mailbox = Arc::new(Mailbox::default());
    let closer = {
        let mailbox = Arc::clone(&mailbox);
        thread::spawn(move || {
            thread::sleep(SHORT);
            mailbox.close();
        })
    };

    assert_eq!(mailbox.wait(None), Wake::Closed);
    closer.join().unwrap();
    assert_eq!(mailbox.wait(Some(SHORT)), Wake::Closed);
}
