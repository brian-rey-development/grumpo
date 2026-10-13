use super::test_support::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use focus_core::ipc::Request;

use crate::daemon_client::test_support::Reply;
use crate::enforcement::test_support::Rig;
use crate::enforcement::test_support::{
    app, app_block, app_target, blocking, browsing, daemon_status, first_shown, in_app, site_block,
    site_target, with_limit, FakeControl, T0,
};
use crate::platform::Browser;
use crate::storage::test_support::record;

mod background;
mod blocking;
mod limits;
mod private_time;
