#![expect(
    clippy::unwrap_used,
    reason = "test helpers outside #[test] functions are not covered by allow-unwrap-in-tests"
)]

use std::fs;
use std::io::{BufReader, Write};
use std::os::unix::fs::MetadataExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use focus_core::ipc::{
    decode, encode, read_frame, DaemonStatus, ErrorCode, IpcError, Request, Response,
    MAX_REQUEST_BYTES, MAX_RESPONSE_BYTES, PROTOCOL_VERSION,
};
use focus_core::test_support::{schedule, site, window};
use focus_core::{Day, Rules, TimeOfDay};
use tempfile::TempDir;

const STARTUP_TIMEOUT: Duration = Duration::from_secs(10);
const LOCALHOST_LINE: &str = "127.0.0.1 localhost\n";
const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../focus-core/tests/fixtures/focus-hosts.sh"
);

struct Daemon {
    child: Child,
    socket: PathBuf,
}

impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

struct Sandbox {
    dir: TempDir,
}

impl Sandbox {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("hosts"), LOCALHOST_LINE).unwrap();
        Self { dir }
    }

    fn hosts(&self) -> PathBuf {
        self.dir.path().join("hosts")
    }

    fn state_dir(&self) -> PathBuf {
        self.dir.path().join("state")
    }

    fn log(&self) -> PathBuf {
        self.dir.path().join("grumpod.log")
    }

    fn socket(&self) -> PathBuf {
        self.dir.path().join("grumpod.sock")
    }

    fn hosts_content(&self) -> String {
        fs::read_to_string(self.hosts()).unwrap()
    }

    fn owner_uid(&self) -> String {
        fs::metadata(self.dir.path()).unwrap().uid().to_string()
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_grumpod"));
        command
            .args(args)
            .env("GRUMPOD_STATE_DIR", self.state_dir())
            .env("GRUMPOD_HOSTS_FILE", self.hosts())
            .env("GRUMPOD_SOCKET", self.socket())
            .env("GRUMPOD_LOG_FILE", self.log())
            .stdin(Stdio::null());
        command
    }

    fn start(&self) -> Daemon {
        self.start_with_log_filter(None)
    }

    fn start_with_log_filter(&self, filter: Option<&str>) -> Daemon {
        let mut command = self.command(&["run", "--owner-uid", &self.owner_uid()]);
        if let Some(filter) = filter {
            command.env("RUST_LOG", filter);
        }
        let child = command
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let daemon = Daemon {
            child,
            socket: self.socket(),
        };
        daemon.wait_until_listening();
        daemon
    }
}

impl Daemon {
    fn wait_until_listening(&self) {
        let deadline = Instant::now() + STARTUP_TIMEOUT;
        while Instant::now() < deadline {
            if UnixStream::connect(&self.socket).is_ok() {
                return;
            }
            thread::sleep(Duration::from_millis(20));
        }
        panic!("the daemon did not start listening within {STARTUP_TIMEOUT:?}");
    }

    fn stop(mut self) {
        self.child.kill().unwrap();
        self.child.wait().unwrap();
    }

    fn exchange(&self, bytes: &[u8]) -> Response {
        let mut stream = UnixStream::connect(&self.socket).unwrap();
        let _ = stream.write_all(bytes);
        let mut reader = BufReader::new(stream);
        let frame = read_frame(&mut reader, MAX_RESPONSE_BYTES).unwrap();
        decode(&frame).unwrap()
    }

    fn request(&self, request: &Request) -> Result<DaemonStatus, IpcError> {
        self.exchange(&encode(request).unwrap()).into_result()
    }
}

fn always_on_rules(hosts: &[&str]) -> Rules {
    Rules {
        blocked_sites: hosts.iter().map(|host| site(host)).collect(),
        schedule: schedule(vec![window(&Day::ALL, 0, TimeOfDay::END_OF_DAY.minutes())]),
        ..Rules::default()
    }
}

fn update(rules: Rules) -> Request {
    Request::UpdateRules { rules }
}

fn error_code(result: Result<DaemonStatus, IpcError>) -> ErrorCode {
    result.unwrap_err().code
}

#[test]
fn status_works_on_a_fresh_install() {
    let sandbox = Sandbox::new();
    let daemon = sandbox.start();

    let status = daemon.request(&Request::Status).unwrap();

    assert!(!status.enforcement.active);
    assert!(status.rules.blocked_sites.is_empty());
    assert_eq!(sandbox.hosts_content(), LOCALHOST_LINE);
}

#[test]
fn tightening_writes_the_managed_block_and_persists_privately() {
    let sandbox = Sandbox::new();
    let daemon = sandbox.start();

    let status = daemon
        .request(&update(always_on_rules(&["youtube.com"])))
        .unwrap();

    assert!(status.enforcement.active);
    let hosts = sandbox.hosts_content();
    assert!(hosts.starts_with(LOCALHOST_LINE), "{hosts}");
    assert!(hosts.contains("0.0.0.0 youtube.com\n"), "{hosts}");
    assert!(hosts.contains("0.0.0.0 www.youtube.com\n"), "{hosts}");
    let state = sandbox.state_dir().join("state.json");
    assert_eq!(fs::metadata(&state).unwrap().mode() & 0o777, 0o600);
    assert_eq!(
        fs::metadata(sandbox.state_dir()).unwrap().mode() & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(sandbox.socket()).unwrap().mode() & 0o777,
        0o600
    );
}

#[test]
fn loosening_is_rejected_while_enforcing() {
    let sandbox = Sandbox::new();
    let daemon = sandbox.start();
    daemon
        .request(&update(always_on_rules(&["youtube.com", "reddit.com"])))
        .unwrap();
    let before = sandbox.hosts_content();

    let result = daemon.request(&update(always_on_rules(&["youtube.com"])));

    assert_eq!(error_code(result), ErrorCode::LooseningLocked);
    assert_eq!(sandbox.hosts_content(), before);
}

#[test]
fn a_version_mismatched_envelope_yields_version_mismatch() {
    let sandbox = Sandbox::new();
    let daemon = sandbox.start();

    let response = daemon.exchange(b"{\"protocolVersion\":99,\"body\":{\"kind\":\"status\"}}\n");

    assert_eq!(
        error_code(response.into_result()),
        ErrorCode::VersionMismatch
    );
}

#[test]
fn an_oversized_frame_is_rejected_and_the_daemon_keeps_serving() {
    let sandbox = Sandbox::new();
    let daemon = sandbox.start();

    let response = daemon.exchange(&vec![b'x'; MAX_REQUEST_BYTES + 1024]);

    assert_eq!(error_code(response.into_result()), ErrorCode::Protocol);
    assert!(daemon.request(&Request::Status).is_ok());
}

#[test]
fn a_client_that_sends_nothing_does_not_wedge_the_daemon() {
    let sandbox = Sandbox::new();
    let daemon = sandbox.start();
    let silent = UnixStream::connect(&daemon.socket).unwrap();

    assert!(daemon.request(&Request::Status).is_ok());

    drop(silent);
    assert!(daemon.request(&Request::Status).is_ok());
}

#[test]
fn a_restart_reapplies_the_block_before_any_client_connects() {
    let sandbox = Sandbox::new();
    let daemon = sandbox.start();
    daemon
        .request(&update(always_on_rules(&["youtube.com"])))
        .unwrap();
    daemon.stop();
    fs::write(sandbox.hosts(), LOCALHOST_LINE).unwrap();

    let restarted = sandbox.start();

    assert!(sandbox.hosts_content().contains("0.0.0.0 youtube.com\n"));
    let status = restarted.request(&Request::Status).unwrap();
    assert_eq!(status.rules.blocked_sites.len(), 1);
}

#[test]
fn a_corrupt_state_file_is_set_aside_and_the_daemon_still_starts() {
    let sandbox = Sandbox::new();
    fs::create_dir_all(sandbox.state_dir()).unwrap();
    fs::write(sandbox.state_dir().join("state.json"), "{ broken").unwrap();

    let daemon = sandbox.start();

    assert!(daemon.request(&Request::Status).is_ok());
    let aside: Vec<_> = fs::read_dir(sandbox.state_dir())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with("state.json.corrupt-"))
        .collect();
    assert_eq!(aside.len(), 1);
}

#[test]
fn a_damaged_state_file_does_not_lift_the_block_that_was_in_force() {
    let sandbox = Sandbox::new();
    let first = sandbox.start();
    first
        .request(&update(always_on_rules(&["youtube.com"])))
        .unwrap();
    first.stop();
    fs::write(sandbox.state_dir().join("state.json"), "{ broken").unwrap();

    let restarted = sandbox.start();

    assert!(sandbox.hosts_content().contains("0.0.0.0 youtube.com\n"));
    let status = restarted.request(&Request::Status).unwrap();
    assert!(status.rules.blocked_sites.is_empty());
    assert_eq!(status.held_sites.len(), 1);
}

#[test]
fn a_stale_socket_file_does_not_prevent_startup() {
    let sandbox = Sandbox::new();
    let first = sandbox.start();
    first.stop();
    assert!(Path::new(&sandbox.socket()).exists());

    let second = sandbox.start();

    assert!(second.request(&Request::Status).is_ok());
}

#[test]
fn the_daemon_writes_structured_logs_to_the_configured_file_without_rule_contents() {
    let sandbox = Sandbox::new();
    let daemon = sandbox.start();
    daemon
        .request(&update(always_on_rules(&["secret-site.com"])))
        .unwrap();

    let log = fs::read_to_string(sandbox.log()).unwrap();

    assert!(log.contains("listening"), "{log}");
    assert!(log.contains("hosts file updated"), "{log}");
    assert!(!log.contains("secret-site.com"), "{log}");
}

#[test]
fn a_second_daemon_is_refused_and_leaves_the_first_one_serving() {
    let sandbox = Sandbox::new();
    let daemon = sandbox.start();

    let second = sandbox
        .command(&["run", "--owner-uid", &sandbox.owner_uid()])
        .env_remove("GRUMPOD_LOG_FILE")
        .output()
        .unwrap();

    assert_eq!(second.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&second.stderr);
    assert!(stderr.contains("already running"), "{stderr}");
    assert!(daemon.request(&Request::Status).is_ok());
}

#[test]
fn state_and_hosts_commands_refuse_to_run_beside_the_daemon() {
    let sandbox = Sandbox::new();
    let daemon = sandbox.start();
    let state_before = fs::read_dir(sandbox.state_dir()).unwrap().count();

    for args in [&["import-legacy", FIXTURE][..], &["strip-hosts"]] {
        let output = sandbox.command(args).output().unwrap();
        assert_eq!(output.status.code(), Some(1), "args {args:?}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Stop the daemon first"), "{stderr}");
    }

    assert_eq!(
        fs::read_dir(sandbox.state_dir()).unwrap().count(),
        state_before
    );
    assert!(daemon.request(&Request::Status).is_ok());
}

#[test]
fn the_commands_run_again_once_the_daemon_has_stopped() {
    let sandbox = Sandbox::new();
    sandbox.start().stop();

    let strip = sandbox.command(&["strip-hosts"]).output().unwrap();
    assert_eq!(strip.status.code(), Some(0), "{strip:?}");
    let import = sandbox
        .command(&["import-legacy", FIXTURE])
        .output()
        .unwrap();
    assert_eq!(import.status.code(), Some(0), "{import:?}");
}

#[test]
fn ping_succeeds_against_a_running_daemon_and_fails_without_one() {
    let sandbox = Sandbox::new();
    let failed = sandbox.command(&["ping"]).output().unwrap();
    assert_eq!(failed.status.code(), Some(1));

    let daemon = sandbox.start();
    let answered = sandbox.command(&["ping"]).output().unwrap();
    assert_eq!(answered.status.code(), Some(0), "{answered:?}");

    daemon.stop();
    let stale = sandbox.command(&["ping"]).output().unwrap();
    assert_eq!(
        stale.status.code(),
        Some(1),
        "a leftover socket file is not a live daemon"
    );
}

fn secret_kind_request() -> String {
    format!(
        "{{\"protocolVersion\":{PROTOCOL_VERSION},\"body\":{{\"kind\":\"secret-kind.com\"}}}}\n"
    )
}

#[test]
fn request_text_is_logged_only_at_debug() {
    let quiet = Sandbox::new();
    let daemon = quiet.start();
    daemon.exchange(secret_kind_request().as_bytes());
    daemon.stop();
    let log = fs::read_to_string(quiet.log()).unwrap();
    assert!(log.contains("rejected a malformed request"), "{log}");
    assert!(!log.contains("secret-kind.com"), "{log}");

    let verbose = Sandbox::new();
    let daemon = verbose.start_with_log_filter(Some("debug"));
    daemon.exchange(secret_kind_request().as_bytes());
    daemon.stop();
    let log = fs::read_to_string(verbose.log()).unwrap();
    assert!(log.contains("secret-kind.com"), "{log}");
}

#[test]
fn a_hosts_file_that_is_not_utf8_still_gets_blocked() {
    let sandbox = Sandbox::new();
    let original = b"127.0.0.1 localhost\n# caf\xE9\n";
    fs::write(sandbox.hosts(), original).unwrap();
    let daemon = sandbox.start();

    daemon
        .request(&update(always_on_rules(&["youtube.com"])))
        .unwrap();

    let hosts = fs::read(sandbox.hosts()).unwrap();
    assert!(hosts.starts_with(original));
    assert!(String::from_utf8_lossy(&hosts).contains("0.0.0.0 youtube.com\n"));
}

fn exits_without_serving(sandbox: &Sandbox) -> std::process::ExitStatus {
    let child = sandbox
        .command(&["run", "--owner-uid", &sandbox.owner_uid()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut daemon = Daemon {
        child,
        socket: sandbox.socket(),
    };
    let deadline = Instant::now() + STARTUP_TIMEOUT;
    while Instant::now() < deadline {
        if let Some(status) = daemon.child.try_wait().unwrap() {
            return status;
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("the daemon kept running with a state it cannot use");
}

#[test]
fn a_state_from_a_newer_daemon_stops_startup_and_stays_untouched() {
    let sandbox = Sandbox::new();
    fs::create_dir_all(sandbox.state_dir()).unwrap();
    let newer = r#"{"schemaVersion":99}"#;
    fs::write(sandbox.state_dir().join("state.json"), newer).unwrap();
    let hosts_before = sandbox.hosts_content();

    let status = exits_without_serving(&sandbox);

    assert_eq!(status.code(), Some(1));
    assert_eq!(
        fs::read_to_string(sandbox.state_dir().join("state.json")).unwrap(),
        newer
    );
    assert!(!sandbox.socket().exists());
    assert_eq!(sandbox.hosts_content(), hosts_before);
    let names: Vec<String> = fs::read_dir(sandbox.state_dir())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        names.iter().all(|name| !name.contains("corrupt")),
        "{names:?}"
    );
}

#[test]
fn a_damaged_state_is_set_aside_and_the_daemon_serves_defaults() {
    let sandbox = Sandbox::new();
    fs::create_dir_all(sandbox.state_dir()).unwrap();
    fs::write(sandbox.state_dir().join("state.json"), "{ not json").unwrap();

    let daemon = sandbox.start();

    assert!(daemon.request(&Request::Status).is_ok());
    let kept = fs::read_dir(sandbox.state_dir())
        .unwrap()
        .filter(|entry| {
            entry
                .as_ref()
                .is_ok_and(|entry| entry.file_name().to_string_lossy().contains(".corrupt-"))
        })
        .count();
    assert_eq!(kept, 1);
}
