//! A modal alert for failures that stop the app from starting.

use objc2_app_kit::NSAlert;
use objc2_foundation::{ns_string, MainThreadMarker, NSString};

use crate::platform::Dialogs;

#[derive(Debug)]
pub(super) struct MacDialogs;

impl Dialogs for MacDialogs {
    fn alert(&self, title: &str, message: &str) {
        let Some(main) = MainThreadMarker::new() else {
            tracing::error!(title, "could not show an alert off the main thread");
            return;
        };
        let alert = NSAlert::new(main);
        alert.setMessageText(&NSString::from_str(title));
        alert.setInformativeText(&NSString::from_str(message));
        alert.addButtonWithTitle(ns_string!("Quit"));
        alert.runModal();
    }
}
