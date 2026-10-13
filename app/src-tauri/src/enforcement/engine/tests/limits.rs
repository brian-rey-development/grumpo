use super::*;

#[test]
fn a_reached_limit_is_reported_to_the_daemon_exactly_once() {
    let rig = Rig::new(steam_limited(30));
    spend_on_steam(&rig, 1_800, T0 - 3_600);
    let mut engine = rig.engine();

    engine.sample(None, T0);
    engine.sample(None, T0 + 2);
    engine.sample(None, T0 + 30);

    assert_eq!(reports(&rig), 1);
}

#[test]
fn reporting_a_limit_tells_the_hub_listeners() {
    let rig = Rig::new(steam_limited(30));
    spend_on_steam(&rig, 1_800, T0 - 3_600);
    let heard = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&heard);
    rig.hub.on_change(Box::new(move |_| {
        counter.fetch_add(1, Ordering::SeqCst);
    }));

    rig.engine().sample(None, T0);

    assert_eq!(heard.load(Ordering::SeqCst), 1);
}

#[test]
fn a_report_that_hit_a_dead_daemon_is_retried_once_it_is_back() {
    let rig = Rig::new(steam_limited(30));
    spend_on_steam(&rig, 1_800, T0 - 3_600);
    let mut engine = rig.engine();
    rig.daemon.push(Reply::HangUp);

    engine.sample(None, T0);
    engine.sample(None, T0 + 2);
    rig.hub.refresh();
    engine.sample(None, T0 + 4);
    engine.sample(None, T0 + 6);

    assert_eq!(reports(&rig), 2, "one failed, one retried, then no more");
}

#[test]
fn the_warning_comes_once_five_minutes_before_the_limit() {
    let rig = Rig::new(steam_limited(30));
    spend_on_steam(&rig, 1_560, T0 - 3_600);
    let mut engine = rig.engine();

    engine.sample(None, T0);
    engine.sample(None, T0 + 12);
    engine.sample(None, T0 + 24);

    assert_eq!(rig.titles(), ["4 minutes left on com.valvesoftware.steam"]);
    assert_eq!(reports(&rig), 0);
}

#[test]
fn time_not_yet_written_to_the_store_still_reaches_the_limit_on_time() {
    let rig = Rig::new(steam_limited(30));
    spend_on_steam(&rig, 1_780, T0);
    let mut engine = rig.engine();
    let steam = in_app(STEAM);
    engine.usage_written(T0);

    engine.sample(Some(&steam), T0);
    assert_eq!(reports(&rig), 0, "20 s are still left");
    engine.sample(Some(&steam), T0 + 20);

    assert_eq!(reports(&rig), 1);
}

#[test]
fn site_limits_count_visits_to_subdomains() {
    let status = with_limit(daemon_status(), site_target("reddit.com"), 10);
    let rig = Rig::new(status);
    record(
        &rig.store,
        "com.google.Chrome",
        "Chrome",
        Some("old.reddit.com"),
        (T0 - 1_200, T0 - 600),
    );

    rig.engine().sample(None, T0);

    assert_eq!(reports(&rig), 1);
}

#[test]
fn local_midnight_clears_what_was_already_warned() {
    let rig = Rig::new(steam_limited(30));
    spend_on_steam(&rig, 1_560, T0 - 3_600);
    let mut engine = rig.engine();
    engine.sample(None, T0);

    spend_on_steam(&rig, 1_560, T0 + DAY - 3_600);
    engine.sample(None, T0 + DAY);

    assert_eq!(rig.titles().len(), 2, "the second day gets its own warning");
}

#[test]
fn usage_from_yesterday_does_not_count_today() {
    let rig = Rig::new(steam_limited(30));
    spend_on_steam(&rig, 3_000, T0 - DAY);
    let mut engine = rig.engine();

    engine.sample(None, T0);

    assert_eq!(reports(&rig), 0);
}
