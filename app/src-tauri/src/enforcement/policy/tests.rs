use focus_core::ipc::DaemonStatus;

use super::test_support::*;
use super::*;
use crate::enforcement::memory::Outcome;
use crate::enforcement::test_support::{
    app, app_target, blocking, browsing, in_app, site, site_target, with_limit, END, T0,
};
use crate::platform::Browser;

mod background;
mod block_terms;
mod blocking;
mod limits;
mod notice_end;
mod protected;
