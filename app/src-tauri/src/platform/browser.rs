#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Browser {
    Chrome,
    Brave,
    Edge,
    Safari,
    Firefox,
}

impl Browser {
    pub const ALL: [Self; 5] = [
        Self::Chrome,
        Self::Brave,
        Self::Edge,
        Self::Safari,
        Self::Firefox,
    ];

    pub fn from_bundle_id(bundle_id: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|browser| browser.bundle_id() == bundle_id)
    }

    pub fn bundle_id(self) -> &'static str {
        match self {
            Self::Chrome => "com.google.Chrome",
            Self::Brave => "com.brave.Browser",
            Self::Edge => "com.microsoft.edgemac",
            Self::Safari => "com.apple.Safari",
            Self::Firefox => "org.mozilla.firefox",
        }
    }

    pub fn can_close_tabs(self) -> bool {
        !matches!(self, Self::Firefox)
    }
}
