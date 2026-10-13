mod codec;
mod envelope;
mod error;
mod request;
mod socket;
mod status;
#[cfg(unix)]
pub mod transport;
mod views;

pub use codec::{
    decode, encode, read_frame, IpcCodecError, Message, MAX_REQUEST_BYTES, MAX_RESPONSE_BYTES,
};
pub use error::{ErrorCode, IpcError};
pub use request::{Request, Response};
pub use socket::{socket_path, DEFAULT_SOCKET_PATH, SOCKET_ENV};
pub use status::DaemonStatus;
pub use views::{EnforcementReason, EnforcementView, UnixSeconds, UnlockPhaseView, UnlockView};

use envelope::Envelope;

pub const PROTOCOL_VERSION: u32 = 3;
