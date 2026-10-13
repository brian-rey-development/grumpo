use super::*;

fn found(data: &str) -> Icon {
    Icon::Found(data.to_owned())
}

#[test]
fn returns_what_was_stored() {
    let cache = IconCache::new(4);
    assert_eq!(cache.get("com.a"), None);
    cache.insert("com.a", found("a"));
    cache.insert("com.b", Icon::Missing);
    assert_eq!(cache.get("com.a"), Some(found("a")));
    assert_eq!(cache.get("com.b"), Some(Icon::Missing));
}

#[test]
fn evicts_the_least_recently_used_entry() {
    let cache = IconCache::new(2);
    cache.insert("com.a", found("a"));
    cache.insert("com.b", found("b"));
    assert_eq!(cache.get("com.a"), Some(found("a")));
    cache.insert("com.c", found("c"));

    assert_eq!(cache.get("com.b"), None);
    assert_eq!(cache.get("com.a"), Some(found("a")));
    assert_eq!(cache.get("com.c"), Some(found("c")));
}

#[test]
fn replacing_an_entry_does_not_grow_the_cache() {
    let cache = IconCache::new(2);
    cache.insert("com.a", Icon::Missing);
    cache.insert("com.a", found("a"));
    cache.insert("com.b", found("b"));
    assert_eq!(cache.get("com.a"), Some(found("a")));
    assert_eq!(cache.get("com.b"), Some(found("b")));
}

#[test]
fn a_zero_capacity_still_holds_one_entry() {
    let cache = IconCache::new(0);
    cache.insert("com.a", found("a"));
    assert_eq!(cache.get("com.a"), Some(found("a")));
}
