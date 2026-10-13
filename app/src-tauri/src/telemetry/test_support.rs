use super::*;
use std::sync::{Arc, Mutex};
use tracing::Level;
use tracing_subscriber::fmt;

#[derive(Clone, Default)]
pub struct Captured(Arc<Mutex<Vec<u8>>>);

impl io::Write for Captured {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn logged_while(work: impl FnOnce()) -> String {
    let captured = Captured::default();
    let writer = captured.clone();
    let subscriber = fmt()
        .with_ansi(false)
        .with_max_level(Level::DEBUG)
        .with_writer(move || writer.clone())
        .finish();
    tracing::subscriber::with_default(subscriber, work);
    let bytes = captured.0.lock().unwrap().clone();
    String::from_utf8(bytes).unwrap()
}
