use super::*;

#[test]
fn converts_seconds_and_clamps_negatives() {
    assert_eq!(to_duration(1.5).unwrap(), Duration::from_millis(1500));
    assert_eq!(to_duration(-0.2).unwrap(), Duration::ZERO);
}

#[test]
fn rejects_non_finite_readings() {
    assert!(to_duration(f64::NAN).is_err());
    assert!(to_duration(f64::INFINITY).is_err());
}
