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
