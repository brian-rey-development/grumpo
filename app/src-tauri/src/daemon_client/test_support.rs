//! A scripted `focusd` on a Unix socket in a temp directory, shared by the tests of every
//! module that talks to the daemon.

use std::collections::{BTreeSet, VecDeque};
use std::io::{BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use focus_core::ipc::{
    decode, encode, read_frame, DaemonStatus, EnforcementReason, EnforcementView, Request,
    Response, UnlockPhaseView, UnlockView, MAX_REQUEST_BYTES,
};
use focus_core::Rules;

use super::DaemonClient;

use crate::concurrency::LockExt;

const REQUEST_READ_LIMIT: Duration = Duration::from_secs(2);

pub fn daemon_status() -> DaemonStatus {
    DaemonStatus {
        rules: Rules::default(),
        enforcement: EnforcementView {
            active: false,
            reason: EnforcementReason::None,
            active_until: None,
            blocked_sites: BTreeSet::new(),
            blocked_apps: BTreeSet::new(),
        },
        exhausted: Vec::new(),
        unlock: UnlockView {
            phase: UnlockPhaseView::Locked,
            next_cooldown_minutes: 5,
            sentence: String::new(),
            durations: Vec::new(),
        },
    }
}

#[derive(Debug, Clone)]
pub enum Reply {
    Respond(Box<Response>),
    Raw(Vec<u8>),
    HangUp,
    /// Accepts the connection and never answers.
    Silent,
}

#[derive(Debug)]
struct Script {
    queued: VecDeque<Reply>,
    fallback: Reply,
    requests: Vec<Request>,
}

#[derive(Debug)]
pub struct FakeDaemon {
    pub socket: PathBuf,
    script: Arc<Mutex<Script>>,
    stopping: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
    _dir: tempfile::TempDir,
}

impl FakeDaemon {
    pub fn start() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join("focusd.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        let script = Arc::new(Mutex::new(Script {
            queued: VecDeque::new(),
            fallback: Reply::Respond(Box::new(Response::Ok {
                status: daemon_status(),
            })),
            requests: Vec::new(),
        }));
        let stopping = Arc::new(AtomicBool::new(false));
        let join = thread::spawn({
            let (script, stopping) = (Arc::clone(&script), Arc::clone(&stopping));
            move || serve(&listener, &script, &stopping)
        });
        Self {
            socket,
            script,
            stopping,
            join: Some(join),
            _dir: dir,
        }
    }

    pub fn client(&self) -> DaemonClient {
        DaemonClient::new(self.socket.clone())
    }

    pub fn push(&self, reply: Reply) {
        self.edit(|script| script.queued.push_back(reply));
    }

    pub fn respond_with(&self, status: DaemonStatus) {
        self.edit(|script| script.fallback = Reply::Respond(Box::new(Response::Ok { status })));
    }

    pub fn requests(&self) -> Vec<Request> {
        self.edit(|script| script.requests.clone())
    }

    fn edit<T>(&self, change: impl FnOnce(&mut Script) -> T) -> T {
        change(&mut self.script.lock_or_recover())
    }
}

impl Drop for FakeDaemon {
    fn drop(&mut self) {
        self.stopping.store(true, Ordering::SeqCst);
        drop(UnixStream::connect(&self.socket));
        if let Some(join) = self.join.take() {
            join.join().unwrap();
        }
    }
}

fn serve(listener: &UnixListener, script: &Mutex<Script>, stopping: &AtomicBool) {
    let mut held = Vec::new();
    for stream in listener.incoming().flatten() {
        if stopping.load(Ordering::SeqCst) {
            return;
        }
        let reply = next_reply(&stream, script);
        match reply {
            Reply::Respond(response) => deliver(&stream, &encode(&*response).unwrap()),
            Reply::Raw(bytes) => deliver(&stream, &bytes),
            Reply::HangUp => {}
            Reply::Silent => held.push(stream),
        }
    }
}

fn next_reply(stream: &UnixStream, script: &Mutex<Script>) -> Reply {
    stream.set_read_timeout(Some(REQUEST_READ_LIMIT)).unwrap();
    let frame = read_frame(&mut BufReader::new(stream), MAX_REQUEST_BYTES).unwrap();
    let request = decode::<Request>(&frame).unwrap();
    let mut script = script.lock_or_recover();
    script.requests.push(request);
    let fallback = script.fallback.clone();
    script.queued.pop_front().unwrap_or(fallback)
}

fn deliver(mut stream: &UnixStream, bytes: &[u8]) {
    drop(stream.write_all(bytes));
}
