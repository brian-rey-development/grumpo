use super::*;

const SECOND: Duration = Duration::from_secs(1);

#[test]
fn nothing_is_due_until_something_is_marked() {
    let mut coalescer = Coalescer::new(SECOND);

    assert!(!coalescer.take_due(Instant::now()));
    assert_eq!(coalescer.wait_hint(Instant::now()), None);
}

#[test]
fn the_first_event_goes_out_immediately() {
    let mut coalescer = Coalescer::new(SECOND);
    coalescer.mark();

    assert!(coalescer.take_due(Instant::now()));
    assert!(!coalescer.take_due(Instant::now()), "taking it clears it");
}

#[test]
fn a_burst_inside_the_interval_collapses_to_one_trailing_event() {
    let start = Instant::now();
    let mut coalescer = Coalescer::new(SECOND);
    coalescer.mark();
    assert!(coalescer.take_due(start));

    for ms in [100, 200, 300] {
        coalescer.mark();
        assert!(!coalescer.take_due(start + Duration::from_millis(ms)));
    }

    assert_eq!(
        coalescer.wait_hint(start + Duration::from_millis(300)),
        Some(Duration::from_millis(700))
    );
    assert!(coalescer.take_due(start + SECOND));
    assert!(!coalescer.take_due(start + SECOND * 2));
}

#[test]
fn an_event_after_a_quiet_interval_goes_out_at_once() {
    let start = Instant::now();
    let mut coalescer = Coalescer::new(SECOND);
    coalescer.mark();
    assert!(coalescer.take_due(start));

    coalescer.mark();

    assert_eq!(
        coalescer.wait_hint(start + SECOND * 5),
        Some(Duration::ZERO)
    );
    assert!(coalescer.take_due(start + SECOND * 5));
}
