//! Finds installed apps without rendering anything.

use objc2::rc::autoreleasepool;
use objc2_app_kit::NSWorkspace;
use objc2_foundation::NSString;

use crate::platform::AppLocator;

#[derive(Debug)]
pub(super) struct MacLocator;

impl AppLocator for MacLocator {
    fn is_installed(&self, bundle_id: &str) -> bool {
        autoreleasepool(|_| {
            NSWorkspace::sharedWorkspace()
                .URLForApplicationWithBundleIdentifier(&NSString::from_str(bundle_id))
                .is_some()
        })
    }
}

#[cfg(test)]
mod tests;
