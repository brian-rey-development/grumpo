//! Turns raw platform probes into stored usage sessions. Data flows one way, from the
//! observer through the session builder into the store, and the sampler drives that loop.

pub mod activity;
pub mod health;
pub mod observer;
mod probe_errors;
mod pruner;
pub mod sampler;
pub mod session_builder;
mod session_writer;
mod tab_memory;
#[cfg(test)]
mod test_support;
pub mod timing;
