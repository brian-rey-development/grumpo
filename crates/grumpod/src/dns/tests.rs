use std::io::Read;

use super::*;

#[test]
fn a_successful_command_returns_ok() {
    assert!(run_with_timeout("/usr/bin/true", &[], Duration::from_secs(5)).is_ok());
}

#[test]
fn a_failing_command_reports_its_status() {
    let error = run_with_timeout("/usr/bin/false", &[], Duration::from_secs(5)).unwrap_err();
    assert!(error.to_string().contains("exited with"), "{error}");
}

#[test]
fn a_missing_program_is_an_error() {
    let error = run_with_timeout("/nonexistent/program", &[], Duration::from_secs(1));
    assert_eq!(error.unwrap_err().kind(), io::ErrorKind::NotFound);
}

#[test]
fn a_slow_command_is_killed_at_the_timeout() {
    let started = Instant::now();
    let error = run_with_timeout("/bin/sleep", &["30"], Duration::from_millis(150));
    assert_eq!(error.unwrap_err().kind(), io::ErrorKind::TimedOut);
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn nothing_pending_runs_nothing() {
    assert_ne!(PendingFlush::NEEDED, PendingFlush::NONE);
    assert_eq!(PendingFlush::default(), PendingFlush::NONE);
    PendingFlush::NONE.run();
}

#[test]
fn a_failed_poll_kills_the_command_instead_of_leaking_it() {
    let mut child = Command::new("/bin/sleep")
        .arg("30")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut pipe = child.stdout.take().unwrap();
    let started = Instant::now();

    let error = wait_for(child, "sleep", Duration::from_secs(5), |_| {
        Err(io::Error::other("poll failed"))
    });

    assert_eq!(error.unwrap_err().to_string(), "poll failed");
    // The pipe closes only when the process is gone, so reaching the end proves it was killed.
    assert_eq!(pipe.read(&mut [0; 1]).unwrap(), 0);
    assert!(started.elapsed() < Duration::from_secs(10));
}

struct Gate {
    started: mpsc::Receiver<()>,
    release: mpsc::Sender<()>,
    worker: FlushWorker,
}

/// A worker whose runs each announce themselves, then wait until released. Dropping the
/// `release` side lets every later run through.
fn gated_worker() -> Gate {
    let (announce, started) = mpsc::channel();
    let (release, released) = mpsc::channel::<()>();
    let worker = FlushWorker::with_action(move || {
        let _ = announce.send(());
        let _ = released.recv();
    })
    .unwrap();
    Gate {
        started,
        release,
        worker,
    }
}

#[test]
fn a_burst_of_requests_during_a_run_costs_one_more_run() {
    let Gate {
        started,
        release,
        worker,
    } = gated_worker();
    worker.submit(PendingFlush::NEEDED);
    started.recv().unwrap();

    for _ in 0..10 {
        worker.submit(PendingFlush::NEEDED);
    }
    drop(worker);
    drop(release);

    assert_eq!(started.iter().count(), 1);
}

#[test]
fn submitting_does_not_wait_for_a_running_flush() {
    let Gate {
        started,
        release,
        worker,
    } = gated_worker();
    worker.submit(PendingFlush::NEEDED);
    started.recv().unwrap();

    worker.submit(PendingFlush::NEEDED);
    worker.submit(PendingFlush::NEEDED);

    release.send(()).unwrap();
}

#[test]
fn nothing_pending_never_wakes_the_worker() {
    let Gate {
        started,
        release,
        worker,
    } = gated_worker();
    worker.submit(PendingFlush::NONE);
    drop(worker);
    drop(release);

    assert_eq!(started.iter().count(), 0);
}
