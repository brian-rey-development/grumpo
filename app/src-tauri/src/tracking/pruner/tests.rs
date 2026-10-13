use super::*;
use crate::storage::test_support::record;

const NOW: i64 = 1_000 * 86_400;

fn count(store: &Store) -> i64 {
    store
        .with_conn(|c| Ok(c.query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))?))
        .unwrap()
}

#[test]
fn prunes_at_start_then_once_a_day() {
    let store = Arc::new(Store::open_in_memory().unwrap());
    let old = NOW - 100 * 86_400;
    record(&store, "com.a", "A", None, (old, old + 10));
    let mut pruner = Pruner::new(Arc::clone(&store));

    pruner.run_if_due(NOW);
    assert_eq!(count(&store), 0);

    record(&store, "com.a", "A", None, (old, old + 10));
    pruner.run_if_due(NOW + PRUNE_INTERVAL_SECONDS - 1);
    assert_eq!(count(&store), 1);
    pruner.run_if_due(NOW + PRUNE_INTERVAL_SECONDS);
    assert_eq!(count(&store), 0);
}
