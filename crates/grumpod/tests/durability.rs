//! What survives when the daemon is killed without warning, and what stops a second daemon
//! from sharing its state. These spawn the real binary, like `daemon.rs`.
//!
//! Nothing here sleeps for a fixed time to wait for the daemon: every wait polls a condition
//! against a deadline. The only sleeps are the random delays before a kill, which are the
//! point of the crash test, and the short backoff between polls.

#![expect(
    clippy::unwrap_used,
    reason = "test helpers outside #[test] functions are not covered by allow-unwrap-in-tests"
)]

use std::fs;
use std::io::{BufReader, Read, Write};
use std::os::unix::fs::MetadataExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use focus_core::ipc::{
    decode, encode, read_frame, DaemonStatus, IpcError, Request, Response, MAX_RESPONSE_BYTES,
};
use focus_core::test_support::{schedule, site, window};
use focus_core::{Day, Rules, TimeOfDay};
use tempfile::TempDir;

const DEADLINE: Duration = Duration::from_secs(10);
const POLL_BACKOFF: Duration = Duration::from_millis(5);
const CRASH_CYCLES: u32 = 20;
const MIN_KILL_DELAY_MS: u64 = 15;
const KILL_DELAY_SPREAD_MS: u64 = 120;
const UPDATES_PER_CYCLE: u32 = 20;
const REPLY_TIMEOUT: Duration = Duration::from_secs(5);
const LOCALHOST_LINE: &str = "127.0.0.1 localhost\n";
const BLOCK_START: &str = "# >>> grumpo: managed block, do not edit >>>";
const BLOCK_END: &str = "# <<< grumpo: managed block <<<";
const TEMP_SUFFIX: &str = ".tmp";
const SCHEMA_KEY: &str = "schemaVersion";
const STALE_TEMPS: [&str; 2] = [".hosts.424242.7.tmp", ".state.json.424242.7.tmp"];

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

    fn socket(&self) -> PathBuf {
        self.dir.path().join("grumpod.sock")
    }

    fn spawn(&self, stderr: Stdio) -> Child {
        let owner_uid = fs::metadata(self.dir.path()).unwrap().uid().to_string();
        Command::new(env!("CARGO_BIN_EXE_grumpod"))
            .args(["run", "--owner-uid", &owner_uid])
            .env("GRUMPOD_STATE_DIR", self.state_dir())
            .env("GRUMPOD_HOSTS_FILE", self.hosts())
            .env("GRUMPOD_SOCKET", self.socket())
            .env_remove("GRUMPOD_LOG_FILE")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(stderr)
            .spawn()
            .unwrap()
    }

    fn start(&self) -> Daemon {
        let mut daemon = Daemon {
            child: self.spawn(Stdio::null()),
        };
        wait_until(
            &format!("a daemon listening on {}", self.socket().display()),
            || {
                assert!(
                    daemon.child.try_wait().unwrap().is_none(),
                    "the daemon exited while starting"
                );
                UnixStream::connect(self.socket()).is_ok()
            },
        );
        daemon
    }

    fn files_in(directory: &Path) -> Vec<String> {
        fs::read_dir(directory)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect()
    }

    fn temp_files(&self) -> Vec<String> {
        [self.dir.path().to_path_buf(), self.state_dir()]
            .iter()
            .flat_map(|directory| Self::files_in(directory))
            .filter(|name| name.starts_with('.') && name.ends_with(TEMP_SUFFIX))
            .collect()
    }

    fn assert_files_are_whole(&self, context: &str) {
        let hosts = fs::read_to_string(self.hosts()).unwrap();
        assert_hosts_are_whole(&hosts, context);
        match fs::read(self.state_dir().join("state.json")) {
            Ok(bytes) => assert_state_parses(&bytes, context),
            Err(error) => assert_eq!(error.kind(), std::io::ErrorKind::NotFound, "{context}"),
        }
    }

    fn assert_nothing_set_aside(&self, context: &str) {
        let aside: Vec<_> = Self::files_in(&self.state_dir())
            .into_iter()
            .filter(|name| name.contains("corrupt"))
            .collect();
        assert!(
            aside.is_empty(),
            "{context}: a state file was unusable: {aside:?}"
        );
    }
}

struct Daemon {
    child: Child,
}

impl Daemon {
    fn kill(mut self) {
        self.child.kill().unwrap();
        self.child.wait().unwrap();
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn wait_until(what: &str, mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + DEADLINE;
    while Instant::now() < deadline {
        if condition() {
            return;
        }
        thread::sleep(POLL_BACKOFF);
    }
    panic!("gave up waiting for {what} after {DEADLINE:?}");
}

fn assert_hosts_are_whole(hosts: &str, context: &str) {
    assert!(hosts.starts_with(LOCALHOST_LINE), "{context}: {hosts}");
    let starts = hosts.matches(BLOCK_START).count();
    let ends = hosts.matches(BLOCK_END).count();
    assert_eq!(starts, ends, "{context}: unbalanced block in {hosts}");
    assert!(starts <= 1, "{context}: {starts} blocks in {hosts}");
    if starts == 1 {
        assert!(hosts.find(BLOCK_START) < hosts.find(BLOCK_END), "{context}");
        assert!(hosts.ends_with('\n'), "{context}: cut-off hosts file");
    }
}

fn assert_state_parses(bytes: &[u8], context: &str) {
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .unwrap_or_else(|error| panic!("{context}: state.json does not parse: {error}"));
    assert!(value.get(SCHEMA_KEY).is_some(), "{context}: no schema");
}

fn request(socket: &Path, request: &Request) -> Result<DaemonStatus, IpcError> {
    let mut stream = UnixStream::connect(socket).map_err(|_| IpcError::internal("connect"))?;
    stream.set_read_timeout(Some(REPLY_TIMEOUT)).unwrap();
    stream
        .write_all(&encode(request).unwrap())
        .map_err(|_| IpcError::internal("write"))?;
    let frame = read_frame(&mut BufReader::new(stream), MAX_RESPONSE_BYTES)
        .map_err(|_| IpcError::internal("read"))?;
    decode::<Response>(&frame).unwrap().into_result()
}

fn tightened(current: &Rules, name: &str) -> Rules {
    let mut sites = current.blocked_sites.clone();
    sites.insert(site(name));
    Rules {
        blocked_sites: sites,
        schedule: schedule(vec![window(&Day::ALL, 0, TimeOfDay::END_OF_DAY.minutes())]),
        ..Rules::default()
    }
}

/// Adds one site per request, which is a tightening and so is never refused as a loosening,
/// until the connection breaks or `stop` is raised. Every accepted request rewrites both the
/// hosts block and the state file, which is what a kill should interrupt.
fn churn(socket: &Path, unique: &AtomicU64, stop: &AtomicBool) {
    let mut sent = 0;
    while !stop.load(Ordering::Relaxed) && sent < UPDATES_PER_CYCLE {
        let Ok(status) = request(socket, &Request::Status) else {
            thread::sleep(POLL_BACKOFF);
            continue;
        };
        let name = format!("site{}.example", unique.fetch_add(1, Ordering::Relaxed));
        let rules = tightened(&status.rules, &name);
        let _ = request(socket, &Request::UpdateRules { rules });
        sent += 1;
    }
}

struct Xorshift(u64);

impl Xorshift {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

fn kill_delay(random: &mut Xorshift) -> Duration {
    Duration::from_millis(MIN_KILL_DELAY_MS + random.next() % KILL_DELAY_SPREAD_MS)
}

fn crash_once(sandbox: &Sandbox, unique: &AtomicU64, delay: Duration, context: &str) {
    let daemon = sandbox.start();
    sandbox.assert_nothing_set_aside(context);
    assert_eq!(sandbox.temp_files(), Vec::<String>::new(), "{context}");
    let stop = AtomicBool::new(false);
    thread::scope(|scope| {
        scope.spawn(|| churn(&sandbox.socket(), unique, &stop));
        thread::sleep(delay);
        daemon.kill();
        stop.store(true, Ordering::Relaxed);
    });
    sandbox.assert_files_are_whole(context);
}

#[test]
fn killing_the_daemon_at_random_moments_never_leaves_the_files_unusable() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let seed = u64::try_from(nanos % u128::from(u64::MAX)).unwrap() | 1;
    let mut random = Xorshift(seed);
    let sandbox = Sandbox::new();
    let unique = AtomicU64::new(0);
    for cycle in 0..CRASH_CYCLES {
        let context = format!("cycle {cycle}, seed {seed}");
        crash_once(&sandbox, &unique, kill_delay(&mut random), &context);
    }

    let last = sandbox.start();
    let status = request(&sandbox.socket(), &Request::Status).unwrap();
    assert!(
        !status.rules.blocked_sites.is_empty(),
        "no update survived the whole run, so it proved nothing, seed {seed}"
    );
    sandbox.assert_nothing_set_aside(&format!("final start, seed {seed}"));
    assert_eq!(sandbox.temp_files(), Vec::<String>::new(), "seed {seed}");
    last.kill();
}

#[test]
fn starting_clears_the_temporary_files_a_crash_left_behind() {
    let sandbox = Sandbox::new();
    sandbox.start().kill();
    let planted = [
        sandbox.dir.path().join(STALE_TEMPS[0]),
        sandbox.state_dir().join(STALE_TEMPS[1]),
    ];
    for path in &planted {
        fs::write(path, "half a write").unwrap();
    }

    let daemon = sandbox.start();

    assert!(planted.iter().all(|path| !path.exists()));
    assert!(request(&sandbox.socket(), &Request::Status).is_ok());
    daemon.kill();
}

fn exit_code_within_deadline(child: &mut Child) -> Option<i32> {
    let mut code = None;
    wait_until("the second daemon to exit", || {
        code = child.try_wait().unwrap().map(|status| status.code());
        code.is_some()
    });
    code.flatten()
}

#[test]
fn a_second_daemon_on_the_same_state_dir_exits_at_once_and_the_first_keeps_serving() {
    let sandbox = Sandbox::new();
    let first = sandbox.start();
    let started = Instant::now();

    let mut second = sandbox.spawn(Stdio::piped());
    let code = exit_code_within_deadline(&mut second);

    assert_eq!(code, Some(1));
    assert!(started.elapsed() < DEADLINE);
    let mut stderr = String::new();
    second
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut stderr)
        .unwrap();
    assert!(stderr.contains("already running"), "{stderr}");
    assert!(request(&sandbox.socket(), &Request::Status).is_ok());
    first.kill();
}
