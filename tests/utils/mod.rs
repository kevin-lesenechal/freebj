pub mod bin_runner;

pub fn assert_f64_eq(actual: f64, expected: f64, within: f64) {
    assert!((expected - actual).abs() < within,
            "Expected: {}\nActual:   {}\nWithin:   {}",
            expected, actual, within);
}
