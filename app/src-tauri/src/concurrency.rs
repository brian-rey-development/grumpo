//! The small tools the app's worker threads share: locks that survive a poisoned mutex, a
//! thread that stops on request, and a throttle for bursts of events.

mod coalescer;
mod lock;
mod stoppable;

pub use coalescer::Coalescer;
pub use lock::{CondvarExt, LockExt};
pub use stoppable::{StopSignal, StoppableThread};
