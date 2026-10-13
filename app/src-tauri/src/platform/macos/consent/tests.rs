use super::*;

const fn empty_cache() -> Mutex<Cache> {
    Mutex::new(Cache {
        entries: BTreeMap::new(),
    })
}

#[test]
fn statuses_map_to_consent() {
    assert_eq!(consent_of(0), Consent::Granted);
    assert_eq!(consent_of(-1743), Consent::Denied);
    assert_eq!(consent_of(-1744), Consent::Unasked);
    assert_eq!(consent_of(-600), Consent::Granted);
    assert_eq!(consent_of(-50), Consent::Granted);
}

#[test]
fn missing_consent_blocks_with_the_matching_error() {
    assert!(gate(Browser::Chrome, Consent::Granted).is_ok());
    assert!(matches!(
        gate(Browser::Chrome, Consent::Unasked),
        Err(PlatformError::NeedsConsent(Permission::Automation(
            Browser::Chrome
        )))
    ));
    assert!(matches!(
        gate(Browser::Safari, Consent::Denied),
        Err(PlatformError::PermissionDenied(Permission::Automation(
            Browser::Safari
        )))
    ));
}

#[test]
fn the_status_is_read_once_per_ttl_per_browser() {
    let cache = empty_cache();
    let start = Instant::now();
    let mut reads = 0;
    let mut ask = |browser, at| {
        check(&cache, browser, at, || {
            reads += 1;
            ERR_WOULD_REQUIRE_CONSENT
        })
    };
    assert_eq!(ask(Browser::Chrome, start), Consent::Unasked);
    assert_eq!(
        ask(Browser::Chrome, start + CACHE_TTL / 2),
        Consent::Unasked
    );
    assert_eq!(ask(Browser::Safari, start), Consent::Unasked);
    assert_eq!(ask(Browser::Chrome, start + CACHE_TTL), Consent::Unasked);
    assert_eq!(reads, 3);
}

#[test]
fn a_grant_is_picked_up_after_the_cache_expires() {
    let cache = empty_cache();
    let start = Instant::now();
    assert_eq!(
        check(&cache, Browser::Edge, start, || ERR_NOT_PERMITTED),
        Consent::Denied
    );
    let later = start + CACHE_TTL + Duration::from_secs(1);
    assert_eq!(
        check(&cache, Browser::Edge, later, || NO_ERR),
        Consent::Granted
    );
}
