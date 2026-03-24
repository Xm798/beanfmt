use beanfmt::options::TimelessPosition;
use beanfmt::sort::{parse_time, sort_input};

#[test]
fn already_sorted_unchanged() {
    let input = "\
2024-01-01 * \"First\"
  Assets:Bank  100 USD

2024-01-02 * \"Second\"
  Assets:Bank  200 USD

2024-01-03 * \"Third\"
  Assets:Bank  300 USD
";
    assert_eq!(sort_input(input, false, TimelessPosition::Begin), input);
}

#[test]
fn out_of_order_sorted_by_date() {
    let input = "\
2024-01-03 * \"Third\"
  Assets:Bank  300 USD

2024-01-01 * \"First\"
  Assets:Bank  100 USD

2024-01-02 * \"Second\"
  Assets:Bank  200 USD
";
    let expected = "\
2024-01-01 * \"First\"
  Assets:Bank  100 USD

2024-01-02 * \"Second\"
  Assets:Bank  200 USD

2024-01-03 * \"Third\"
  Assets:Bank  300 USD
";
    assert_eq!(sort_input(input, false, TimelessPosition::Begin), expected);
}

#[test]
fn barriers_respected() {
    let input = "\
2024-01-03 * \"Late\"
  Assets:Bank  300 USD

2024-01-01 * \"Early\"
  Assets:Bank  100 USD

pushtag #trip

2024-01-05 * \"After barrier late\"
  Assets:Bank  500 USD

2024-01-04 * \"After barrier early\"
  Assets:Bank  400 USD

poptag #trip
";
    let expected = "\
2024-01-01 * \"Early\"
  Assets:Bank  100 USD

2024-01-03 * \"Late\"
  Assets:Bank  300 USD

pushtag #trip

2024-01-04 * \"After barrier early\"
  Assets:Bank  400 USD

2024-01-05 * \"After barrier late\"
  Assets:Bank  500 USD

poptag #trip
";
    assert_eq!(sort_input(input, false, TimelessPosition::Begin), expected);
}

#[test]
fn time_metadata_sorting() {
    let input = "\
2024-01-01 * \"Later\"
  time: \"15:30\"
  Assets:Bank  200 USD

2024-01-01 * \"Earlier\"
  time: \"09:00\"
  Assets:Bank  100 USD
";
    let expected = "\
2024-01-01 * \"Earlier\"
  time: \"09:00\"
  Assets:Bank  100 USD

2024-01-01 * \"Later\"
  time: \"15:30\"
  Assets:Bank  200 USD
";
    assert_eq!(sort_input(input, false, TimelessPosition::Begin), expected);
}

#[test]
fn prudent_sort_minimal_moves() {
    // 1,2,5,3,4 — the LIS is 1,2,3,4 (or 1,2,5) — with prudent sort,
    // 5 is the only out-of-place entry that moves, result: 1,2,3,4,5
    let input = "\
2024-01-01 * \"One\"

2024-01-02 * \"Two\"

2024-01-05 * \"Five\"

2024-01-03 * \"Three\"

2024-01-04 * \"Four\"
";
    let expected = "\
2024-01-01 * \"One\"

2024-01-02 * \"Two\"

2024-01-03 * \"Three\"

2024-01-04 * \"Four\"

2024-01-05 * \"Five\"
";
    assert_eq!(sort_input(input, false, TimelessPosition::Begin), expected);
}

#[test]
fn entries_without_date_stay_in_place() {
    let input = "\
2024-01-02 * \"Second\"
  Assets:Bank  200 USD

; A comment with no date

2024-01-01 * \"First\"
  Assets:Bank  100 USD
";
    let result = sort_input(input, false, TimelessPosition::Begin);
    assert!(result.contains("; A comment with no date"));
    assert_eq!(result, input);
}

#[test]
fn same_date_preserves_relative_order() {
    let input = "\
2024-01-01 * \"Alpha\"
  Assets:Bank  100 USD

2024-01-01 * \"Beta\"
  Assets:Bank  200 USD

2024-01-01 * \"Gamma\"
  Assets:Bank  300 USD
";
    // All same date, no time — should preserve original order
    assert_eq!(sort_input(input, false, TimelessPosition::Begin), input);
}

// parse_time tests

#[test]
fn parse_time_hm() {
    assert_eq!(parse_time("09:30"), Some("09:30:00".into()));
}

#[test]
fn parse_time_hms() {
    assert_eq!(parse_time("14:30:45"), Some("14:30:45".into()));
}

#[test]
fn parse_time_quoted() {
    assert_eq!(parse_time("\"09:30\""), Some("09:30:00".into()));
}

#[test]
fn parse_time_unix_seconds() {
    // 1704067200 = 2024-01-01 00:00:00 UTC
    assert_eq!(parse_time("1704067200"), Some("00:00:00".into()));
}

#[test]
fn parse_time_unix_millis() {
    assert_eq!(parse_time("1704067200000"), Some("00:00:00".into()));
}

#[test]
fn parse_time_unix_micros() {
    assert_eq!(parse_time("1704067200000000"), Some("00:00:00".into()));
}

#[test]
fn parse_time_15_digit_boundary() {
    // 15-digit number: 100000000000000 is treated as milliseconds (>= 10^12)
    // 100000000000000 / 1000 = 100000000000 seconds
    let result = parse_time("100000000000000");
    assert!(result.is_some(), "15-digit timestamp should be valid");
}

#[test]
fn parse_time_invalid() {
    assert_eq!(parse_time("not-a-time"), None);
    assert_eq!(parse_time(""), None);
}

#[test]
fn parse_time_single_digit_hour() {
    assert_eq!(parse_time("9:30"), Some("09:30:00".into()));
    assert_eq!(parse_time("9:30:45"), Some("09:30:45".into()));
}

#[test]
fn parse_time_iso_datetime() {
    assert_eq!(parse_time("2024-01-01T09:30:45"), Some("09:30:45".into()));
    assert_eq!(parse_time("2024-01-01T09:30"), Some("09:30:00".into()));
    assert_eq!(parse_time("2024-01-01 14:00:00"), Some("14:00:00".into()));
    assert_eq!(parse_time("2024-01-01T9:05"), Some("09:05:00".into()));
    // Lenient: timezone and fractional seconds are ignored
    assert_eq!(parse_time("2024-01-01T09:30:45Z"), Some("09:30:45".into()));
    assert_eq!(
        parse_time("2024-01-01T09:30:45+08:00"),
        Some("09:30:45".into())
    );
    assert_eq!(
        parse_time("2024-01-01T09:30:45.123"),
        Some("09:30:45".into())
    );
}

// LIS sort edge-case tests

#[test]
fn empty_input() {
    assert_eq!(sort_input("", false, TimelessPosition::Begin), "");
}

#[test]
fn single_entry() {
    let input = "\
2024-01-01 * \"Only\"
  Assets:Bank  100 USD
";
    assert_eq!(sort_input(input, false, TimelessPosition::Begin), input);
}

#[test]
fn two_entries_reversed() {
    let input = "\
2024-01-02 * \"B\"
  Assets:Bank  200 USD

2024-01-01 * \"A\"
  Assets:Bank  100 USD
";
    let expected = "\
2024-01-01 * \"A\"
  Assets:Bank  100 USD

2024-01-02 * \"B\"
  Assets:Bank  200 USD
";
    assert_eq!(sort_input(input, false, TimelessPosition::Begin), expected);
}

#[test]
fn fully_reversed_five() {
    let input = "\
2024-01-05 * \"E\"

2024-01-04 * \"D\"

2024-01-03 * \"C\"

2024-01-02 * \"B\"

2024-01-01 * \"A\"
";
    let result = sort_input(input, false, TimelessPosition::Begin);
    let dates: Vec<&str> = result
        .lines()
        .filter(|l| l.starts_with("2024"))
        .map(|l| &l[..10])
        .collect();
    assert_eq!(
        dates,
        vec![
            "2024-01-01",
            "2024-01-02",
            "2024-01-03",
            "2024-01-04",
            "2024-01-05"
        ]
    );
}

#[test]
fn unix_timestamp_time_sorting() {
    let input = "\
2024-01-01 * \"Later\"
  time: \"1704110400\"
  Assets:Bank  200 USD

2024-01-01 * \"Earlier\"
  time: \"1704067200\"
  Assets:Bank  100 USD
";
    let result = sort_input(input, false, TimelessPosition::Begin);
    assert!(
        result.find("Earlier").unwrap() < result.find("Later").unwrap(),
        "Earlier timestamp should come first"
    );
}

// Undated comment preservation test

#[test]
fn undated_comment_between_transactions_preserved() {
    let input = "\
2024-01-03 * \"Third\"
  Assets:Bank  300 USD

; Section divider

2024-01-01 * \"First\"
  Assets:Bank  100 USD
";
    let result = sort_input(input, false, TimelessPosition::Begin);
    assert!(result.contains("; Section divider"));
}

// Descending sort tests

#[test]
fn descending_sort_reverses_order() {
    let input = "\
2024-01-01 * \"First\"
  Assets:Bank  100 USD

2024-01-02 * \"Second\"
  Assets:Bank  200 USD

2024-01-03 * \"Third\"
  Assets:Bank  300 USD
";
    let expected = "\
2024-01-03 * \"Third\"
  Assets:Bank  300 USD

2024-01-02 * \"Second\"
  Assets:Bank  200 USD

2024-01-01 * \"First\"
  Assets:Bank  100 USD
";
    assert_eq!(sort_input(input, true, TimelessPosition::Begin), expected);
}

#[test]
fn descending_sort_out_of_order() {
    let input = "\
2024-01-02 * \"Second\"
  Assets:Bank  200 USD

2024-01-03 * \"Third\"
  Assets:Bank  300 USD

2024-01-01 * \"First\"
  Assets:Bank  100 USD
";
    let expected = "\
2024-01-03 * \"Third\"
  Assets:Bank  300 USD

2024-01-02 * \"Second\"
  Assets:Bank  200 USD

2024-01-01 * \"First\"
  Assets:Bank  100 USD
";
    assert_eq!(sort_input(input, true, TimelessPosition::Begin), expected);
}

#[test]
fn descending_barriers_respected() {
    let input = "\
2024-01-01 * \"Early\"
  Assets:Bank  100 USD

2024-01-03 * \"Late\"
  Assets:Bank  300 USD

pushtag #trip

2024-01-04 * \"After barrier early\"
  Assets:Bank  400 USD

2024-01-05 * \"After barrier late\"
  Assets:Bank  500 USD

poptag #trip
";
    let expected = "\
2024-01-03 * \"Late\"
  Assets:Bank  300 USD

2024-01-01 * \"Early\"
  Assets:Bank  100 USD

pushtag #trip

2024-01-05 * \"After barrier late\"
  Assets:Bank  500 USD

2024-01-04 * \"After barrier early\"
  Assets:Bank  400 USD

poptag #trip
";
    assert_eq!(sort_input(input, true, TimelessPosition::Begin), expected);
}

// TimelessPosition tests

#[test]
fn timeless_begin_places_before_timed() {
    let input = "\
2024-01-01 * \"Has time\"
  time: \"15:00\"
  Assets:Bank  200 USD

2024-01-01 * \"No time\"
  Assets:Bank  100 USD
";
    let expected = "\
2024-01-01 * \"No time\"
  Assets:Bank  100 USD

2024-01-01 * \"Has time\"
  time: \"15:00\"
  Assets:Bank  200 USD
";
    assert_eq!(sort_input(input, false, TimelessPosition::Begin), expected);
}

#[test]
fn timeless_end_places_after_timed() {
    let input = "\
2024-01-01 * \"No time\"
  Assets:Bank  100 USD

2024-01-01 * \"Has time\"
  time: \"09:00\"
  Assets:Bank  200 USD
";
    let expected = "\
2024-01-01 * \"Has time\"
  time: \"09:00\"
  Assets:Bank  200 USD

2024-01-01 * \"No time\"
  Assets:Bank  100 USD
";
    assert_eq!(sort_input(input, false, TimelessPosition::End), expected);
}

#[test]
fn timeless_end_multiple_preserve_order() {
    let input = "\
2024-01-01 * \"No time A\"
  Assets:Bank  100 USD

2024-01-01 * \"Has time\"
  time: \"10:00\"
  Assets:Bank  200 USD

2024-01-01 * \"No time B\"
  Assets:Bank  300 USD
";
    let expected = "\
2024-01-01 * \"Has time\"
  time: \"10:00\"
  Assets:Bank  200 USD

2024-01-01 * \"No time A\"
  Assets:Bank  100 USD

2024-01-01 * \"No time B\"
  Assets:Bank  300 USD
";
    assert_eq!(sort_input(input, false, TimelessPosition::End), expected);
}

#[test]
fn timeless_end_with_descending() {
    let input = "\
2024-01-01 * \"No time\"
  Assets:Bank  100 USD

2024-01-01 * \"Has time\"
  time: \"09:00\"
  Assets:Bank  200 USD

2024-01-02 * \"Day 2\"
  Assets:Bank  300 USD
";
    let result = sort_input(input, true, TimelessPosition::End);
    // Day 2 should come first (descending), then within Jan 1: timed before timeless
    assert!(result.find("Day 2").unwrap() < result.find("Has time").unwrap());
    assert!(result.find("Has time").unwrap() < result.find("No time").unwrap());
}

#[test]
fn timeless_begin_multiple_preserve_order() {
    let input = "\
2024-01-01 * \"Has time\"
  time: \"10:00\"
  Assets:Bank  200 USD

2024-01-01 * \"No time A\"
  Assets:Bank  100 USD

2024-01-01 * \"No time B\"
  Assets:Bank  300 USD
";
    let expected = "\
2024-01-01 * \"No time A\"
  Assets:Bank  100 USD

2024-01-01 * \"No time B\"
  Assets:Bank  300 USD

2024-01-01 * \"Has time\"
  time: \"10:00\"
  Assets:Bank  200 USD
";
    assert_eq!(sort_input(input, false, TimelessPosition::Begin), expected);
}

#[test]
fn timeless_begin_with_descending() {
    let input = "\
2024-01-01 * \"No time\"
  Assets:Bank  100 USD

2024-01-01 * \"Has time\"
  time: \"09:00\"
  Assets:Bank  200 USD

2024-01-02 * \"Day 2\"
  Assets:Bank  300 USD
";
    let result = sort_input(input, true, TimelessPosition::Begin);
    // Descending: Day 2 first, then within Jan 1: timeless before timed (Begin)
    assert!(result.find("Day 2").unwrap() < result.find("No time").unwrap());
    assert!(result.find("No time").unwrap() < result.find("Has time").unwrap());
}

#[test]
fn all_timed_entries_unaffected_by_timeless_position() {
    let input = "\
2024-01-01 * \"Later\"
  time: \"15:30\"
  Assets:Bank  200 USD

2024-01-01 * \"Earlier\"
  time: \"09:00\"
  Assets:Bank  100 USD
";
    let result_begin = sort_input(input, false, TimelessPosition::Begin);
    let result_end = sort_input(input, false, TimelessPosition::End);
    assert_eq!(result_begin, result_end);
}

#[test]
fn timeless_position_from_str_valid() {
    assert_eq!(
        "begin".parse::<TimelessPosition>(),
        Ok(TimelessPosition::Begin)
    );
    assert_eq!("end".parse::<TimelessPosition>(), Ok(TimelessPosition::End));
    assert_eq!(
        "BEGIN".parse::<TimelessPosition>(),
        Ok(TimelessPosition::Begin)
    );
    assert_eq!("End".parse::<TimelessPosition>(), Ok(TimelessPosition::End));
}

#[test]
fn timeless_position_from_str_invalid() {
    assert!("start".parse::<TimelessPosition>().is_err());
    assert!("".parse::<TimelessPosition>().is_err());
    assert!("middle".parse::<TimelessPosition>().is_err());
}
