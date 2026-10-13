use super::*;

#[test]
fn returns_output_of_a_fast_command() {
    let mut command = Command::new("/bin/echo");
    command.arg("hi");
    let output = run_bounded(&mut command, Duration::from_secs(5)).unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "hi");
}

#[test]
fn kills_a_command_that_outlives_the_deadline() {
    let mut command = Command::new("/bin/sleep");
    command.arg("30");
    let started = Instant::now();
    let result = run_bounded(&mut command, Duration::from_millis(100));
    assert!(matches!(result, Err(RunError::Timeout)));
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn a_flood_of_output_neither_deadlocks_nor_exhausts_memory() {
    let mut command = Command::new("/usr/bin/head");
    command.args(["-c", "2000000", "/dev/zero"]);
    let started = Instant::now();
    let output = run_bounded(&mut command, Duration::from_secs(10)).unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout.len(), MAX_OUTPUT_BYTES);
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn a_flood_on_stderr_is_drained_too() {
    let mut command = Command::new("/bin/sh");
    command.args(["-c", "/usr/bin/head -c 1000000 /dev/zero >&2; echo done"]);
    let output = run_bounded(&mut command, Duration::from_secs(10)).unwrap();
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "done");
    assert_eq!(output.stderr.len(), MAX_OUTPUT_BYTES);
}

#[test]
fn reports_a_missing_binary_as_a_spawn_error() {
    let mut command = Command::new("/nonexistent/binary");
    let result = run_bounded(&mut command, Duration::from_secs(1));
    assert!(matches!(result, Err(RunError::Spawn(_))));
}
